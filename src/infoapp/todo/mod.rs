use eframe::egui::{CentralPanel, Context, RichText, Separator, ScrollArea};
use std::{fs, thread};
use std::io::Write;

const STORE_PATH: &str = "storage";
const LIST_NAME: &str = "test";

pub struct Todo {
    items: Vec<TodoItem>,
    next_id: usize,
}

#[derive(Clone)]
struct TodoItem {
    id: usize,
    text: String,
    done: bool,
}

impl TodoItem {
    pub fn new(id: usize, text: String, done: bool) -> Self {
        Self { id, text, done }
    }

    pub fn empty(id: usize) -> Self {
        Self {
            id,
            text: String::new(),
            done: false,
        }
    }
}

impl Todo {
    pub fn new() -> Self {
        let (items, last_id) = load_items();
        Todo {
            items,
            next_id: last_id + 1,
        }
    }

    pub fn render(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading(RichText::new("Todo").strong());
            ui.add_space(1.0);
            ui.add(Separator::default().horizontal());
            ScrollArea::vertical().show(ui, |ui| {
                self.render_items(ui);
                if ui.button("+").on_hover_text("Add Item").clicked() {
                    self.items.push(TodoItem::empty(self.next_id));
                    self.next_id += 1;
                }
            });
            ui.add(Separator::default().horizontal());
        });
    }

    fn render_items(&mut self, ui: &mut eframe::egui::Ui) {
        let mut items_to_delete: Vec<usize> = vec![];
        let mut save = false;
        for item in self.items.iter_mut() {
            ui.horizontal(|ui| {
                let checkbox = ui.checkbox(&mut item.done, "");
                let input = ui.text_edit_singleline(&mut item.text);
                if input.lost_focus() && !item.text.is_empty() {
                    save = true;
                }
                if checkbox.clicked() && !item.text.is_empty() {
                    save = true;
                }
                if ui.button("-").on_hover_text("Delete").clicked() {
                    items_to_delete.push(item.id);
                };
            });
        }
        if save {
            save_items(&self.items);
        }
        if !items_to_delete.is_empty() {
            self.delete(items_to_delete);
        }
    }

    fn delete(&mut self, ids: Vec<usize>) {
        self.items.retain(|item| !ids.contains(&item.id));
        save_items(&self.items)
    }
}

// load items from file
fn load_items() -> (Vec<TodoItem>, usize) {
    let mut items: Vec<TodoItem> = vec![];
    let file = fs::read_to_string(format!("{}/{}.todo", STORE_PATH, LIST_NAME))
        .expect("Unable to open file");
    let mut last = "0::f";
    for line in file.lines() {
        last = line;
        let parts: Vec<&str> = line.split("::").collect();
        let id = parts[0].parse::<usize>().unwrap();
        let done = parts[1].parse::<bool>().unwrap();
        let text = parts[2].to_string();
        items.push(TodoItem::new(id, text, done));
    }
    return (
        items,
        last.split("::").collect::<Vec<&str>>()[0]
            .parse::<usize>()
            .unwrap(),
    );
}

// save all items to file
fn save_items(items: &Vec<TodoItem>) {
    let itemss = items.clone();
    thread::spawn(move || {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(format!("{}/{}.todo", STORE_PATH, LIST_NAME))
            .expect("Unable to open file");
        for item in itemss {
            writeln!(file, "{}::{}::{}", item.id, item.done, item.text).expect("Unable to write file");
        }
    });
}