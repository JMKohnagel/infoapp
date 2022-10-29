use eframe::egui::{Context, CentralPanel, Separator};


pub struct News {

}


impl News {
    pub fn new() -> Self {
        News {  }
    }

    pub fn render(&self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            ui.label("News");
            ui.add_space(1.0);
            ui.add(Separator::default().horizontal());
        });
    }
}
