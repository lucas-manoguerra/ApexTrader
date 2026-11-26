use std::mem::size_of;
use std::sync::Arc;

use bytemuck_derive::{Pod, Zeroable};
use eframe::egui;
use eframe::egui_wgpu::{Callback, CallbackResources, CallbackTrait, ScreenDescriptor};
use eframe::wgpu;

use crate::renderer::RenderGeometry;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct GpuVertex {
    position: [f32; 2],
    color: [u8; 4],
}

pub struct ChartGpuCallback {
    vertices: Arc<[GpuVertex]>,
    target_format: wgpu::TextureFormat,
}

struct ChartGpuResources {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    vertex_capacity: u64,
    vertex_count: u32,
    last_vertices: Option<Arc<[GpuVertex]>>,
}

impl ChartGpuResources {
    fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("apex-trader-chart-shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("chart.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("apex-trader-chart-pipeline-layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![0 => Float32x2, 1 => Unorm8x4];
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("apex-trader-chart-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: size_of::<GpuVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &ATTRIBUTES,
                }],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let vertex_capacity = 4;
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("apex-trader-chart-vertices"),
            size: vertex_capacity,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            vertex_buffer,
            vertex_capacity,
            vertex_count: 0,
            last_vertices: None,
        }
    }
}

impl ChartGpuCallback {
    pub fn vertices_from_geometry(
        geometry: &RenderGeometry,
        viewport_size: [f32; 2],
    ) -> Arc<[GpuVertex]> {
        let [width, height] = viewport_size;
        if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
            return Arc::from([]);
        }

        let vertices = geometry
            .vertices
            .iter()
            .filter_map(|vertex| {
                let x = vertex.position[0] / width * 2.0 - 1.0;
                let y = 1.0 - vertex.position[1] / height * 2.0;
                if !x.is_finite() || !y.is_finite() {
                    return None;
                }

                Some(GpuVertex {
                    position: [x, y],
                    color: [
                        vertex.color.red,
                        vertex.color.green,
                        vertex.color.blue,
                        vertex.color.alpha,
                    ],
                })
            })
            .collect::<Vec<_>>();

        Arc::from(vertices)
    }

    pub fn shape(
        rect: egui::Rect,
        vertices: Arc<[GpuVertex]>,
        target_format: wgpu::TextureFormat,
    ) -> egui::Shape {
        egui::Shape::Callback(Callback::new_paint_callback(
            rect,
            Self {
                vertices,
                target_format,
            },
        ))
    }
}

impl CallbackTrait for ChartGpuCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        if !callback_resources.contains::<ChartGpuResources>() {
            callback_resources.insert(ChartGpuResources::new(device, self.target_format));
        }

        let Some(resources) = callback_resources.get_mut::<ChartGpuResources>() else {
            return Vec::new();
        };

        if resources
            .last_vertices
            .as_ref()
            .is_some_and(|previous| Arc::ptr_eq(previous, &self.vertices))
        {
            return Vec::new();
        }

        if self.vertices.is_empty() {
            resources.vertex_count = 0;
            resources.last_vertices = Some(self.vertices.clone());
            return Vec::new();
        }

        let Some(vertex_count) = u32::try_from(self.vertices.len()).ok() else {
            return Vec::new();
        };
        let Some(required_bytes) = u64::try_from(self.vertices.len())
            .ok()
            .and_then(|count| count.checked_mul(size_of::<GpuVertex>() as u64))
        else {
            return Vec::new();
        };
        let Some(required_capacity) = required_bytes.checked_next_power_of_two() else {
            return Vec::new();
        };
        if required_capacity > device.limits().max_buffer_size {
            return Vec::new();
        }

        if required_capacity > resources.vertex_capacity {
            resources.vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("apex-trader-chart-vertices"),
                size: required_capacity,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            resources.vertex_capacity = required_capacity;
        }

        queue.write_buffer(
            &resources.vertex_buffer,
            0,
            bytemuck::cast_slice(&self.vertices),
        );
        resources.vertex_count = vertex_count;
        resources.last_vertices = Some(self.vertices.clone());
        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::epaint::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        callback_resources: &CallbackResources,
    ) {
        let Some(resources) = callback_resources.get::<ChartGpuResources>() else {
            return;
        };
        if resources.vertex_count == 0 {
            return;
        }

        render_pass.set_pipeline(&resources.pipeline);
        render_pass.set_vertex_buffer(0, resources.vertex_buffer.slice(..));
        render_pass.draw(0..resources.vertex_count, 0..1);
    }
}
