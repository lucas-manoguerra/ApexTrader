# Testing, plataformas y CI

## Niveles de prueba

### Unit tests sin display

Prueba la lógica que no requiere crear una ventana:

- Conversión de eventos a comandos internos.
- Máquina de estados de ventanas.
- Política de redraw/dirty state.
- Conversión de DPI y coordenadas.
- Normalización de entrada.
- Decisiones de cierre y recuperación.

Mantén esta lógica fuera del event loop siempre que sea posible.

### Compile checks

Compila con cada target y feature set soportado por el producto. No uses `--all-features` si habilita combinaciones incompatibles o features no previstas.

### Pruebas de integración con display

En un entorno gráfico real o controlado verifica:

- Creación y cierre de una ventana.
- Cambios de tamaño y escala.
- Minimizar/restaurar y exposición.
- Foco y entrada de teclado/mouse.
- Ventanas múltiples y `WindowId`.
- GTK widgets, contenedores, ventana transitoria y callbacks.
- Transparencia solo si el producto la utiliza.
- Worker -> user event -> UI thread.
- Creación, resize y recreación del renderer si se integra GPU.

## Linux y GTK

Comprueba que la máquina CI tiene toolchain Rust, bibliotecas de desarrollo GTK correspondientes, dependencias nativas de los targets y un servidor de display disponible cuando la prueba lo necesite.

No confundas:

- El backend de ventana elegido por el fork.
- El servidor de display que ejecuta la sesión.
- La versión de GTK instalada.
- La disponibilidad de headers/libs de desarrollo.

El README y el manifiesto del paquete pueden no describir todos los detalles de plataforma con el mismo nivel de actualización. Usa el manifiesto publicado y las pruebas del target como evidencia.

## CI recomendada

- `cargo fmt --check`
- `cargo check` por target/features relevantes
- `cargo test` para lógica headless
- `cargo clippy` con el feature set real
- `cargo tree` para comprobar duplicaciones y compatibilidad
- smoke test con display en Linux cuando la aplicación dependa de GTK
- build/test en cada sistema operativo prometido por el proyecto

## Informes reproducibles

Por cada bug de ventana, adjunta:

- Sistema operativo, versión GTK y sesión/display server.
- Target triple y `rustc -Vv`.
- Versiones Cargo y features activas.
- GPU/driver si hay renderer.
- Logs mínimos del ciclo de vida.
- Pasos exactos para reproducir.

No marques una plataforma como soportada solo porque compila en otra plataforma.
