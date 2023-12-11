#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod string_finder;

use std::sync::Arc;

use eframe::egui;
use egui::mutex::RwLock;
use egui_tiles::SimplificationOptions;
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

pub trait GaffrieTool {
    fn new(file_lock: Arc<RwLock<Vec<u8>>>) -> Self
    where
        Self: Sized;
    fn ui(&mut self, ui: &mut egui::Ui);
    fn title(&self) -> String;
    fn notify(&mut self, event: Event);
}

#[derive(Clone, Copy)]
pub enum Event {
    FileChanged,
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
struct MyApp {
    current_file: Arc<RwLock<Vec<u8>>>,
    tree: egui_tiles::Tree<Pane>,
    tools: Vec<Box<dyn GaffrieTool>>,
}

impl Default for MyApp {
    fn default() -> Self {
        let current_file = Arc::new(RwLock::new(Vec::new()));
        let mut tiles = egui_tiles::Tiles::default();
        let mut tabs = vec![];
        tabs.push(tiles.insert_pane(Pane { nr: 0 }));
        tabs.push(tiles.insert_pane(Pane { nr: 1 }));
        let root = tiles.insert_tab_tile(tabs);
        let tree = egui_tiles::Tree::new("tools_tree", root, tiles);
        Self {
            tools: vec![Box::new(StringFinder::new(Arc::clone(&current_file)))],
            current_file,
            tree,
        }
    }
}

impl MyApp {
    fn notify_tools(&mut self, event: Event) {
        for tool in &mut self.tools {
            tool.notify(event);
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("tree").show(ctx, |ui| {
            if ui.button("Select file").clicked() {
                if let Some(file) = rfd::FileDialog::new().pick_file() {
                    let file_content = std::fs::read(file).unwrap();
                    let mut lock = self.current_file.write();
                    *lock = file_content;
                    drop(lock);
                    self.notify_tools(Event::FileChanged);
                }
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut behavior = TreeBehavior {
                tools: &mut self.tools,
            };
            self.tree.ui(&mut behavior, ui);
        });
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        #[cfg(feature = "serde")]
        eframe::set_value(_storage, eframe::APP_KEY, &self);
    }
}

struct Pane {
    nr: usize,
}

struct TreeBehavior<'a> {
    tools: &'a mut Vec<Box<dyn GaffrieTool>>,
}

impl<'a> egui_tiles::Behavior<Pane> for TreeBehavior<'a> {
    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        pane: &mut Pane,
    ) -> egui_tiles::UiResponse {
        self.tools[pane.nr].ui(ui);
        egui_tiles::UiResponse::None
    }

    fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
        format!("Pane {}", pane.nr).into()
    }

    fn simplification_options(&self) -> SimplificationOptions {
        let mut options = SimplificationOptions::default();
        options.all_panes_must_have_tabs = true;
        options
    }
}
