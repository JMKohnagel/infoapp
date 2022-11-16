use eframe::egui::{CentralPanel, Context, RichText, Separator, ScrollArea};
use std::{fs, thread};
use std::io::Write;

const STORE_PATH: &str = "storage";
const LIST_NAME: &str = "test";

pub struct Todo {
    items: Vec<TodoItem>,
    next_id: usize,
    filter_value: String,
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
            filter_value: String::new(),
        }
    }

    pub fn render(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading(RichText::new("Todo").strong());
            ui.add_space(1.0);
            ui.add(Separator::default().horizontal());
            ui.text_edit_singleline(&mut self.filter_value);
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
        let filtered_ids = filter_items(&self.items, &self.filter_value);
        for id in filtered_ids.iter() {
            let self_item = get_item_from_id(&mut self.items, id.clone()).unwrap();
            ui.horizontal(|ui| {
                let checkbox = ui.checkbox(&mut self_item.done, "");
                let input = ui.text_edit_singleline(&mut self_item.text);
                if input.lost_focus() && !self_item.text.is_empty() {
                    save = true;
                }
                if checkbox.clicked() && !self_item.text.is_empty() {
                    save = true;
                }
                if ui.button("x").on_hover_text("Delete").clicked() {
                    items_to_delete.push(self_item.id);
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

fn get_item_from_id(items: &mut [TodoItem], id: usize) -> Option<&mut TodoItem> {
    for item in items.iter_mut() {
        if item.id == id {
            return Some(item);
        }
    }
    return None;
}

fn filter_items(items: &[TodoItem], filter_value: &str) -> Vec<usize> {
    return items
        .iter()
        .filter(|item| item.text.to_lowercase().contains(filter_value.to_lowercase().as_str()))
        .map(|item| item.id)
        .collect();
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