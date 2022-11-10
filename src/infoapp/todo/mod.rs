use eframe::egui::{CentralPanel, Context, RichText, Separator, Ui};
use std::fs;
use std::io::Write;


const STORE_PATH: &str = "storage";

pub struct Todo {
    items: Vec<TodoItem>,
    next_id: usize,
}

struct TodoItem {
    id: usize,
    text: String,
    done: bool,
}

impl TodoItem {
    pub fn new(id: usize, text: String, done: bool) -> Self {
        Self { id, text, done}
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
            items: load_items(),
            next_id: items.len(),
        }
    }

    pub fn render(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading(RichText::new("Todo").strong());
            ui.add_space(1.0);
            ui.add(Separator::default().horizontal());
            self.render_items(ui);
            if ui.button("+").on_hover_text("Add Item").clicked() {
                self.items.push(TodoItem::empty(self.next_id));
                self.next_id += 1;
            }
            ui.add(Separator::default().horizontal());
        });
    }

    fn render_items(&mut self, ui: &mut eframe::egui::Ui) {
        let mut items_to_delete: Vec<usize> = vec![];
        for item in self.items.iter_mut() {
            ui.horizontal(|ui| {
                ui.checkbox(&mut item.done, "");
                let input = ui.text_edit_singleline(&mut item.text);
                if input.lost_focus() {
                    persist(item);
                }
                if ui.button("--").on_hover_text("Delete").clicked() {
                    items_to_delete.push(item.id);
                };
            });
        }
        self.delete(items_to_delete);
    }

    fn delete(&mut self, ids: Vec<usize>) {
        self.items.retain(|item| !ids.contains(&item.id));
        // delete items from file
    }

}

// load items from file
fn load_items() -> Vec<TodoItem> {
    let mut items: Vec<TodoItem> = vec![];
    let file = fs::read_to_string(format!("{}/test.todo", STORE_PATH)).expect("Unable to open file");
    for line in file.lines() {
        let parts: Vec<&str> = line.split("::").collect();
        let id = parts[0].parse::<usize>().unwrap();
        let done = parts[1].parse::<bool>().unwrap();
        let text = parts[2].to_string();
        items.push(TodoItem::new(id, text, done));
    }
    return items;
}

// function to persist one item to file
fn persist(arg: &mut TodoItem) {
    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(format!("{}/test.todo", STORE_PATH))
        .expect("Unable to open file");
    if let Err(e) = writeln!(file, "{}::{}::{}", arg.id, arg.done, arg.text) {
        eprintln!("Couldn't write to file: {}", e);
    }
}
