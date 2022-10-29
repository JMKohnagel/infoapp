use eframe::egui::{Context, CentralPanel, Separator};


#[derive(Default)]
pub struct Weather {
    _forecast: ForeCast
}

#[derive(Default)]
struct ForeCast {

}

impl Weather {
    pub fn new() -> Self {
        Weather { _forecast: ForeCast::default() }
    }

    pub fn render(&self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            ui.label("Weather");
            ui.add_space(1.0);
            ui.add(Separator::default().horizontal());
        });
    }
}
