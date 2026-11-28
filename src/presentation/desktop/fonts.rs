use eframe::egui;

pub(super) fn configure(ctx: &egui::Context) {
    ctx.set_fonts(egui::FontDefinitions::default());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_font_definitions_cover_proportional_and_monospace_families() {
        let definitions = egui::FontDefinitions::default();

        assert!(definitions.font_data.contains_key("Ubuntu-Light"));
        assert!(definitions.font_data.contains_key("Hack"));

        let proportional = definitions
            .families
            .get(&egui::FontFamily::Proportional)
            .expect("familia proporcional");
        let monospace = definitions
            .families
            .get(&egui::FontFamily::Monospace)
            .expect("familia monoespaciada");

        assert_eq!(
            proportional.first().map(String::as_str),
            Some("Ubuntu-Light")
        );
        assert_eq!(monospace.first().map(String::as_str), Some("Hack"));
    }
}
