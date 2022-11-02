use eframe::egui::{
    Align, CentralPanel, ColorImage, Context, Hyperlink, Layout, RichText, ScrollArea, Separator,
    Ui,
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
    pub image_updated: bool,
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
    item_rx: Receiver<NewsItem>,
    item_tx: Sender<NewsItem>,
    image_rx: Receiver<Option<ImgPos>>,
    image_tx: Sender<Option<ImgPos>>,
}

impl News {
    pub fn new() -> Self {
        let (image_tx, image_rx) = std::sync::mpsc::channel();
        let (item_tx, item_rx) = std::sync::mpsc::channel();
        News {
            item_count: 0,
            image_load_counter: 0,
            rss_url: RSS_URL.to_string(),
            rendering: false,
            items: Vec::new(),
            item_rx,
            item_tx,
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
                ui.label("Loading...");
                if let Ok(item) = self.item_rx.try_recv() {
                    self.insert_item(item);
                    self.item_count += 1;
                }
                if self.image_load_counter < self.item_count {
                    if let Ok(img_pos) = self.image_rx.try_recv() {
                        if let Some(img_pos) = img_pos {
                            self.try_update_image(img_pos);
                        } else {
                            self.image_load_counter += 1;
                        }
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
                ui.heading(RichText::new("News").strong());
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let refresh_button = ui.button("Refresh");
                if refresh_button.clicked() {
                    if self.rss_url.is_empty() {
                        ui.label("RSS url is not specified");
                    } else {
                        self.refresh();
                    }
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

    pub fn refresh(&mut self) {
        self.items = Vec::new();
        self.image_load_counter = 0;
        self.rendering = true;
        let url = self.rss_url.clone();
        let item_tx = self.item_tx.clone();
        let image_tx = self.image_tx.clone();
        std::thread::spawn(move || {
            get_items(url, item_tx, image_tx);
        });
    }

    // function to sorted insert item in list
    fn insert_item(&mut self, item: NewsItem) {
        let pos = self.items.binary_search_by(|i| i.position.cmp(&item.position));
        match pos {
            Ok(_) => (),
            Err(pos) => self.items.insert(pos, item),
        }
    }

    fn try_update_image(&mut self, img_pos: ImgPos) {
        if self.is_item_in_list(img_pos.pos) {
            self.update_image(img_pos);
        } else {
            self.image_tx.send(Some(img_pos)).unwrap();
        }
    }

    fn is_item_in_list(&self, pos: usize) -> bool {
        let mut result = false;
        self.items.iter().for_each(|item| {
            if item.position == pos {
                result = true;
                return;
            }
        });
        result
    }

    fn update_image(&mut self, img_pos: ImgPos) {
        let mut index = 0;
        self.items.iter().enumerate().for_each(|(i, item)| {
            if item.position == img_pos.pos {
                index = i;
                return;
            }
        });
        self.items[index].image = img_pos.image;
        self.items[index].image_updated = true;
        self.image_load_counter += 1;
    }

}

fn get_channel(rss_url: String) -> Result<Channel, Box<dyn Error>> {
    let content = reqwest::blocking::get(rss_url).unwrap().bytes().unwrap();
    let channel = Channel::read_from(BufReader::new(&content[..]))?;
    Ok(channel)
}

fn get_items(rss_url: String, item_tx: Sender<NewsItem>, image_tx: Sender<Option<ImgPos>>) {
    let channel = get_channel(rss_url);
    let items = match channel {
        Ok(c) => c.into_items().to_vec(),
        Err(_) => Vec::new(),
    };
    items.iter().enumerate().for_each(|(i, item)| {
        let tx = item_tx.clone();
        let img_tx = image_tx.clone();
        let it = item.clone();
        std::thread::spawn(move || {
            let news_item = NewsItem {
                position: i,
                title: it.title().unwrap_or("").to_string(),
                link: it.link().unwrap_or("").to_string(),
                description: it.description().unwrap_or("").to_string(),
                image: loading_image(),
                image_updated: false,
            };
            tx.send(news_item).unwrap();
            get_image(get_image_url(it.content()), i, img_tx);
        });
    });
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