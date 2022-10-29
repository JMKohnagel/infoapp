use eframe::egui::{Context, CentralPanel, Separator};


pub struct Todo {

}


impl Todo {
    pub fn new() -> Self {
        Todo {  }
    }

    pub fn render(&self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            ui.label("Todo");
            ui.add_space(1.0);
            ui.add(Separator::default().horizontal());
        });
    }
}
