use eframe::egui::{CentralPanel, Context, RichText, ScrollArea, Separator};
use std::io::Write;
use std::{fs, thread};

const STORE_PATH: &str = "storage";
const LIST_NAME: &str = "test";

pub struct Todo {
    next_id: usize,
    items: Vec<TodoItem>,
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
        let items = load_items();
        Todo {
            next_id: items.len(),
            items,
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
            let self_item = {
                let this = get_item_by_id(&mut self.items, &id);
                match this {
                    Some(val) => val,
                    None => {
                        eprintln!("Item with id {} not found", id);
                        continue;
                    }
                }
            };
            ui.horizontal(|ui| {
                let checkbox = ui.checkbox(&mut self_item.done, "");
                let input = ui.text_edit_singleline(&mut self_item.text);
                if input.changed() || checkbox.clicked() {
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

fn get_item_by_id<'a>(items: &'a mut [TodoItem], id: &'a usize) -> Option<&'a mut TodoItem> {
    return match items.iter_mut().find(|item| &item.id == id) {
        Some(item) => Some(item),
        None => None,
    };
}

fn filter_items(items: &[TodoItem], filter_value: &str) -> Vec<usize> {
    return items
        .iter()
        .filter(|item| {
            item.text
                .to_lowercase()
                .contains(filter_value.to_lowercase().as_str())
        })
        .map(|item| item.id)
        .collect();
}

// load items from file
fn load_items() -> Vec<TodoItem> {
    let mut items: Vec<TodoItem> = vec![];
    let file = match fs::read_to_string(format!("{}/{}.todo", STORE_PATH, LIST_NAME)) {
        Ok(file) => file,
        Err(_) => return items,
    };
    let mut last = "0::f";
    file.lines().enumerate().for_each(|(i, line)| {
        let mut split = line.split("::");
        split.next();
        let id = i;
        let done = split
            .next()
            .expect(&format!(
                "Todo-File corrupted, missing 2nd value for item {}",
                i
            ))
            .parse::<bool>()
            .expect("Todo-File corrupted, 2nd value is not bool");
        let text = split
            .next()
            .unwrap_or_else(|| {
                eprintln!(
                    "Todo-File corrupted, missing 3rd value for item {}, using empty string",
                    i
                );
                return "";
            })
            .to_string();
        items.push(TodoItem::new(id, text, done));
        last = line;
    });
    return items;
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
            if item.text.is_empty() {
                continue;
            }
            writeln!(file, "{}::{}::{}", item.id, item.done, item.text)
                .expect("Unable to write file");
        }
    });
}
