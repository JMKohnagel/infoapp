use eframe::egui::{Align, CentralPanel, Context, Layout, ScrollArea, Separator, Ui};
use rss::{Channel,Item};
use std::{
    error::Error, io::BufReader,
};

const RSS_URL: &str = "https://www.tagesschau.de/xml/rss2/";

#[derive(Debug)]
pub struct News {
    rss_url: String,
    items: Vec<Item>,
}

impl News {
    pub fn new() -> Self {
        News {
            rss_url: RSS_URL.to_string(),
            items: Vec::new(),
        }
    }

    pub fn render(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            self.render_top_bar(ui);
            ui.add_space(1.0);
            ui.add(Separator::default().horizontal());
            self.render_items(ui)
        });
    }

    fn render_top_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                ui.heading("News");
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let refresh_button = ui.button("Refresh");
                if refresh_button.clicked() {
                    if self.rss_url.is_empty() {
                        ui.label("RSS url is not specified");
                    } else {
                        self.get_items();
                    }
                }
            });
        });
    }

    fn render_items(&self, ui: &mut Ui) {
        ScrollArea::vertical().show(ui, |ui| {
            for item in &self.items {
                ui.label(item.title().unwrap_or("No title"));
                ui.add_space(1.0);
                ui.add(Separator::default().horizontal());
            }
        });
    }

    fn get_channel(&self) -> Result<Channel, Box<dyn Error>> {
        let content = ureq::get(&self.rss_url).call()?.into_string()?;
        let channel = Channel::read_from(BufReader::new(content.as_bytes()))?;
        Ok(channel)
    }

    fn get_items(&mut self) {
        let channel = self.get_channel();
        let items = match channel {
            Ok(c) => c.into_items().to_vec(),
            Err(_) => Vec::new(),
        };
        self.items = items;
    }
}
