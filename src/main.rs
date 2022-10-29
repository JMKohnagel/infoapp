mod infoapp;

use eframe::{
    egui::{self, CentralPanel},
    App,
};
use infoapp::InfoApp;

const TITLE: &str = "InfoApp";

fn main() {
    let options = eframe::NativeOptions {
        min_window_size: Some(egui::vec2(320.0, 100.0)),
        ..Default::default()
    };
    eframe::run_native(TITLE, options, Box::new(|cc| Box::new(InfoApp::new(cc))));
}

impl App for InfoApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |_ui| {
            self.render_top_panel(ctx);
            self.render_current_page(ctx);
        });
    }
} 
