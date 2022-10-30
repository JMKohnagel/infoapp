use eframe::egui::{Align, CentralPanel, Context, Hyperlink, Layout, ScrollArea, Separator, Ui};
use egui_extras::RetainedImage;
use rss::Channel;
use std::{error::Error, fmt::Debug, io::BufReader};

const RSS_URL: &str = "https://www.tagesschau.de/xml/rss2/";

struct NewsItem {
    pub title: String,
    pub link: String,
    pub description: String,
    pub image: RetainedImage,
}

impl Debug for NewsItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NewsItem")
            .field("title", &self.title)
            .field("link", &self.link)
            .field("description", &self.description)
            .finish()
    }
}

#[derive(Debug)]
pub struct News {
    rss_url: String,
    items: Vec<NewsItem>,
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
            self.render_items(ui, ctx);
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
                        self.items = Vec::new();
                        self.get_items();
                    }
                }
            });
        });
    }

    fn render_items(&self, ui: &mut Ui, ctx: &Context) {
        ScrollArea::vertical().show(ui, |ui| {
            for item in &self.items {
                ui.horizontal(|ui| {
                    ui.image(item.image.texture_id(ctx), item.image.size_vec2());
                    ui.vertical(|ui| {
                        ui.label(&item.title);
                        ui.label(&item.description);
                        ui.with_layout(Layout::right_to_left(Align::BOTTOM), |ui| {
                            ui.add(Hyperlink::from_label_and_url("read more ⤴", &item.link));
                        });
                        ui.add_space(1.0);
                    });
                });
                ui.add(Separator::default().horizontal());
            }
        });
    }

    fn get_channel(&self) -> Result<Channel, Box<dyn Error>> {
        let content = reqwest::blocking::get(&self.rss_url).unwrap().bytes().unwrap();
        let channel = Channel::read_from(BufReader::new(&content[..]))?;
        Ok(channel)
    }

    fn get_items(&mut self) {
        let channel = self.get_channel();
        let items = match channel {
            Ok(c) => c.into_items().to_vec(),
            Err(_) => Vec::new(),
        };
        items.iter().for_each(|item| {              // threading
            let image = self.get_image(self.get_image_url(item.content()));
            let news_item = NewsItem {
                title: item.title().unwrap_or("No title").to_string(),
                link: item.link().unwrap_or("No link").to_string(),
                description: item.description().unwrap_or("No description").to_string(),
                image,
            };
            self.items.push(news_item);
        });
    }

    fn get_image(&self, url: &str) -> RetainedImage {       // TODO error handling
        let image_bytes = reqwest::blocking::get(url).unwrap().bytes().unwrap();
        RetainedImage::from_image_bytes("image", &image_bytes).unwrap()
    }

    fn get_image_url<'a>(&'a self, content: Option<&'a str>) -> &str {
        let context_string = match content {
            Some(c) => c,
            None => todo!("default image"),
        };
        let re = regex::Regex::new(r#"<img src="([^"]+)""#).unwrap();
        match re.captures(context_string) {
            Some(caps) => caps.get(1).map_or("", |m| m.as_str()),
            None => todo!("default image"),
        }
    }
}
