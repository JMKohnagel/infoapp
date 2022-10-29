use eframe::egui::{Align, CentralPanel, Context, Layout, ScrollArea, Ui, Separator};

enum SettingsType {
    General,
    Config,
}

impl SettingsType {}

pub struct Settings {
    current_setting: SettingsType,
}

impl Settings {
    pub fn new() -> Self {
        Settings {
            current_setting: SettingsType::General,
        }
    }

    pub fn render(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            ui.label("Settings");
            ui.add_space(1.0);
            ui.add(Separator::default().horizontal());
            ui.with_layout(Layout::left_to_right(Align::TOP), |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    ui.with_layout(Layout::top_down(Align::TOP), |ui| {
                        ui.add_space(9.0);
                        let general_button = ui.button("General");
                        let config_button = ui.button("Config");
                        if general_button.clicked() {
                            self.current_setting = SettingsType::General;
                        }
                        if config_button.clicked() {
                            self.current_setting = SettingsType::Config;
                        }
                    });
                });
                ui.add_space(10.0);
                ui.add(Separator::default().vertical());
                match self.current_setting {
                    SettingsType::General => self.render_general(ui),
                    SettingsType::Config => self.render_config(ui),
                }
            });
        });
    }

    fn render_general(&self, ui: &mut Ui) {
        CentralPanel::default().show_inside(ui, |ui| {
            eframe::egui::widgets::global_dark_light_mode_buttons(ui);
        });
    }

    fn render_config(&self, ui: &mut Ui) {
        CentralPanel::default().show_inside(ui, |ui| {
            ui.label("Config");
        });
    }
}
