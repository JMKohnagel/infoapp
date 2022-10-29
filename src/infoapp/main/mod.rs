use eframe::egui::{Context, CentralPanel, Separator};


pub struct Main {

}


impl Main {
    pub fn new() -> Self {
        Main { }
    }

    pub fn render(&self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            ui.label("Main");
            ui.add_space(1.0);
            ui.add(Separator::default().horizontal());
        });
    }
}
