use eframe::egui::{
    Align, CentralPanel, ColorImage, Context, Hyperlink, Layout, ScrollArea, Separator, Ui, RichText,
};
use egui_extras::RetainedImage;
use rss::Channel;
use std::{
    error::Error,
    fmt::Debug,
    io::BufReader,
    path::Path,
    sync::mpsc::{Receiver, Sender},
};

const RSS_URL: &str = "https://www.tagesschau.de/xml/rss2/";
const LOADING_IMAGE_PATH: &str = "assets/loading.jpg";

struct ImgPos {
    pos: usize,
    image: RetainedImage,
}

struct NewsItem {
    pub position: usize,
    pub title: String,
    pub link: String,
    pub description: String,
    pub image: RetainedImage,
    pub image_url: Option<String>,
}

impl Ord for NewsItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.position.cmp(&other.position)
    }
}

impl PartialOrd for NewsItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for NewsItem {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
    }
}

impl Eq for NewsItem {}

impl Debug for NewsItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NewsItem")
            .field("position", &self.position)
            .field("title", &self.title)
            .field("link", &self.link)
            .field("description", &self.description)
            .finish()
    }
}

pub struct News {
    item_count: usize,
    image_load_counter: usize,
    rss_url: String,
    rendering: bool,
    items: Vec<NewsItem>,
    news_rx: Receiver<Vec<NewsItem>>,
    news_tx: Sender<Vec<NewsItem>>,
    image_rx: Receiver<Option<ImgPos>>,
    image_tx: Sender<Option<ImgPos>>,
}

impl News {
    pub fn new() -> Self {
        let (news_tx, news_rx) = std::sync::mpsc::channel();
        let (image_tx, image_rx) = std::sync::mpsc::channel();
        News {
            item_count: 0,
            image_load_counter: 0,
            rss_url: RSS_URL.to_string(),
            rendering: false,
            items: Vec::new(),
            news_rx,
            news_tx,
            image_rx,
            image_tx,
        }
    }

    pub fn render(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            self.render_top_bar(ui);
            ui.add_space(1.0);
            ui.add(Separator::default().horizontal());
            self.render_items(ui, ctx);
            if self.rendering {
                if self.items.is_empty() {
                    ui.label("Loading...");
                    if let Ok(items) = self.news_rx.try_recv() {
                        self.items = items;
                        self.item_count = self.items.len();
                        self.update_images();
                    }
                }
                if self.image_load_counter < self.item_count || self.image_load_counter == 0 {
                    if let Ok(img_pos) = self.image_rx.try_recv() {
                        if let Some(img_pos) = img_pos {
                            self.items[img_pos.pos].image = img_pos.image;
                        }
                        self.image_load_counter += 1;
                    }
                } else {
                    self.rendering = false;
                }
            }
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
                    self.refresh(ui);
                }
            });
        });
    }

    fn render_items(&mut self, ui: &mut Ui, ctx: &Context) {
        ScrollArea::vertical().show(ui, |ui| {
            for item in &self.items {
                ui.horizontal(|ui| {
                    ui.image(item.image.texture_id(ctx), item.image.size_vec2());
                    ui.vertical(|ui| {
                        ui.label(RichText::new(&item.title).size(25.0).strong());
                        ui.add_space(2.0);
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

    fn refresh(&mut self, ui: &mut Ui) {
        if self.rss_url.is_empty() {
            ui.label("RSS url is not specified");
        } else {
            self.items = Vec::new();
            self.image_load_counter = 0;
            self.rendering = true;
            let url = self.rss_url.clone();
            let tx = self.news_tx.clone();
            std::thread::spawn(move || {
                get_items(url, tx);
            });
            
        }
    }

    // spawn a thread for each item to download the image and send it back to the main thread
    fn update_images(&self) {
        for item in &self.items {
            let tx = self.image_tx.clone();
            let image_url = item.image_url.clone();
            let item_pos = item.position.clone();
            std::thread::spawn(move || {
                get_image(image_url, item_pos, tx);
            });
        }
    }
}

fn get_channel(rss_url: String) -> Result<Channel, Box<dyn Error>> {
    let content = reqwest::blocking::get(rss_url).unwrap().bytes().unwrap();
    let channel = Channel::read_from(BufReader::new(&content[..]))?;
    Ok(channel)
}

fn get_items(rss_url: String, news_tx: Sender<Vec<NewsItem>>) {
    let channel = get_channel(rss_url);
    let items = match channel {
        Ok(c) => c.into_items().to_vec(),
        Err(_) => Vec::new(),
    };
    let mut news_items = Vec::new();
    items.iter().enumerate().for_each(|(i, item)| {
        let item = NewsItem {
            position: i,
            title: item.title().unwrap_or("").to_string(),
            link: item.link().unwrap_or("").to_string(),
            description: item.description().unwrap_or("").to_string(),
            image: loading_image(),
            image_url: get_image_url(item.content()),
        };
        news_items.push(item);
    });
    news_tx.send(news_items).unwrap();
}

fn get_image(url: Option<String>, pos: usize, image_tx: Sender<Option<ImgPos>>) {
    match url {
        Some(url) => {
            let image_bytes = reqwest::blocking::get(url).unwrap().bytes().unwrap();
            let image = RetainedImage::from_image_bytes("image", &image_bytes).unwrap();
            let image_pos = ImgPos { pos, image };
            image_tx.send(Some(image_pos)).unwrap();
        }
        None => image_tx.send(None).unwrap(),
    };
}

fn get_image_url<'a>(content: Option<&'a str>) -> Option<String> {
    let context_string = match content {
        Some(c) => c,
        None => "",
    };
    let re = regex::Regex::new(r#"<img src="([^"]+)""#).unwrap();
    match re.captures(context_string) {
        Some(caps) => Some(caps.get(1).map_or("", |m| m.as_str()).to_string()),
        None => None,
    }
}

fn loading_image() -> RetainedImage {
    let img = color_image(LOADING_IMAGE_PATH).unwrap();
    RetainedImage::from_color_image("loading_image", img)
}

fn color_image(path: &str) -> Result<ColorImage, image::ImageError> {
    let image = image::io::Reader::open(Path::new(path))?.decode()?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    Ok(ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()))
}
