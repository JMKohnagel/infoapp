mod main;
mod news;
mod settings;
mod todo;
mod weather;

use main::Main;
use news::News;
use settings::Settings;
use todo::Todo;
use weather::Weather;

use eframe::{
    egui::{self, menu::bar, Align, Context, Layout, RichText, TextStyle, TopBottomPanel},
    epaint::FontId,
};

pub enum PageType {
    Main,
    Weather,
    News,
    ToDo,
    Settings,
}

pub struct InfoApp {
    pub current_page: PageType,
    pub main: Main,
    pub weather: Weather,
    pub news: News,
    pub todo: Todo,
    pub settings: Settings,
}

impl InfoApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::configure_textstyles(&cc.egui_ctx);
        InfoApp {
            current_page: PageType::Main,
            main: Main::new(),
            weather: Weather::new(),
            news: News::new(),
            todo: Todo::new(),
            settings: Settings::new(),
        }
    }

    pub fn render_top_panel(&mut self, ctx: &egui::Context) {
        use TextStyle::Button;
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(3.0);
            bar(ui, |ui| {
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    let main_button = ui.button(RichText::new("Main").text_style(Button));
                    let weather_button = ui.button(RichText::new("Weather").text_style(Button));
                    let news_button = ui.button(RichText::new("News").text_style(Button));
                    let todo_button = ui.button(RichText::new("Todo").text_style(Button));
                    if main_button.clicked() {
                        self.current_page = PageType::Main;
                    }
                    if weather_button.clicked() {
                        self.current_page = PageType::Weather;
                    }
                    if news_button.clicked() {
                        self.current_page = PageType::News;
                    }
                    if todo_button.clicked() {
                        self.current_page = PageType::ToDo;
                    }
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let settings_button = ui.button(RichText::new("Settings").text_style(Button));
                    if settings_button.clicked() {
                        self.current_page = PageType::Settings;
                    }
                });
            });
            ui.add_space(3.0);
        });
    }

    fn configure_textstyles(egui_ctx: &Context) {
        use eframe::epaint::FontFamily::Proportional;

        let mut style = (*egui_ctx.style()).clone();
        style.text_styles = [
            (TextStyle::Heading, FontId::new(35.0, Proportional)),
            (TextStyle::Body, FontId::new(20.0, Proportional)),
            (TextStyle::Button, FontId::new(15.0, Proportional)),
        ]
        .into();
        egui_ctx.set_style(style);
    }

    pub(crate) fn render_current_page(&mut self, ctx: &Context) {
        match self.current_page {
            PageType::Main => self.render_main(ctx),
            PageType::Weather => self.render_weather(ctx),
            PageType::News => self.render_news(ctx),
            PageType::ToDo => self.render_todo(ctx),
            PageType::Settings => self.render_settings(ctx),
        }
    }

    fn render_main(&self, ctx: &Context) {
        self.main.render(ctx);
    }

    fn render_weather(&self, ctx: &Context) {
        self.weather.render(ctx);
    }

    fn render_news(&self, ctx: &Context) {
        self.news.render(ctx);
    }

    fn render_todo(&self, ctx: &Context) {
        self.todo.render(ctx);
    }

    fn render_settings(&mut self, ctx: &Context) {
        self.settings.render(ctx);
    }
}
