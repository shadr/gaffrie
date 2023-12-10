#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod string_finder;

use std::sync::Arc;

use eframe::egui;
use egui::mutex::RwLock;
use string_finder::StringFinder;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Gaffrie",
        options,
        Box::new(|_cc| {
            #[cfg_attr(not(feature = "serde"), allow(unused_mut))]
            let mut app = MyApp::default();
            #[cfg(feature = "serde")]
            if let Some(storage) = _cc.storage {
                if let Some(state) = eframe::get_value(storage, eframe::APP_KEY) {
                    app = state;
                }
            }
            Box::new(app)
        }),
    )
}

pub enum Event {
    FileChanged,
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
struct MyApp {
    current_file: Arc<RwLock<Vec<u8>>>,
    string_finder: StringFinder,
}

impl Default for MyApp {
    fn default() -> Self {
        let current_file = Arc::new(RwLock::new(Vec::new()));
        Self {
            string_finder: StringFinder::new(Arc::clone(&current_file)),
            current_file,
        }
    }
}

impl MyApp {
    fn notify_tools(&mut self, event: Event) {
        self.string_finder.notify_event(event);
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("tree").show(ctx, |ui| {
            if ui.button("Select file").clicked() {
                if let Some(file) = rfd::FileDialog::new().pick_file() {
                    dbg!(&file);
                    let file_content = std::fs::read(file).unwrap();
                    let mut lock = self.current_file.write();
                    *lock = file_content;
                    drop(lock);
                    self.notify_tools(Event::FileChanged);
                }
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.string_finder.ui(ui);
        });
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        #[cfg(feature = "serde")]
        eframe::set_value(_storage, eframe::APP_KEY, &self);
    }
}
