use eframe::egui;

pub(super) fn configure(ctx: &egui::Context) {
    ctx.set_fonts(egui::FontDefinitions::default());
}

#[cfg(test)]
mod tests {
    use super::*;

    const REQUIRED_GLYPHS: &str = "áéíóúñÑ¿¡%+-/0123456789";

    fn configured_context() -> egui::Context {
        let context = egui::Context::default();
        configure(&context);
        context.run_ui(Default::default(), |_| {});
        context
    }

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

    #[test]
    fn configured_context_contains_required_spanish_and_market_glyphs() {
        let context = configured_context();

        let proportional = egui::FontId::proportional(14.0);
        let monospace = egui::FontId::monospace(14.0);
        context.fonts_mut(|fonts| {
            assert!(
                fonts.has_glyphs(&proportional, REQUIRED_GLYPHS),
                "la familia proporcional debe cubrir todos los glifos requeridos"
            );
            assert!(
                fonts.has_glyphs(&monospace, REQUIRED_GLYPHS),
                "la familia monoespaciada debe cubrir todos los glifos requeridos"
            );
        });
    }

    #[test]
    fn configure_installs_the_expected_font_definitions_on_the_context() {
        let context = configured_context();
        let expected = egui::FontDefinitions::default();

        context.fonts(|fonts| {
            assert_eq!(fonts.definitions(), &expected);
        });
    }
}
