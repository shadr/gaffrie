#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod tools;

use std::{
    fs::File,
    sync::{
        mpsc::{Receiver, Sender},
        Arc,
    },
};

use eframe::egui;
use egui::mutex::RwLock;
use egui_tiles::SimplificationOptions;
use memmap2::{MmapMut, MmapOptions};
use std::future::Future;
use tools::{string_finder::StringFinder, GaffrieTool};

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_drag_and_drop(true),
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

#[cfg(target_arch = "wasm32")]
fn main() {
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id",
                web_options,
                Box::new(|cc| Box::new(MyApp::default())),
            )
            .await
            .expect("failed to start eframe")
    })
}

type ActionsVec = Vec<(
    String,
    Box<dyn Fn(&Arc<RwLock<MmapMut>>, &mut egui_tiles::Tree<Pane>)>,
)>;

#[derive(Clone, Copy)]
pub enum Event {
    FileChanged,
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
struct MyApp {
    current_file: Arc<RwLock<MmapMut>>,
    tree: egui_tiles::Tree<Pane>,

    file_channel: (Sender<MmapMut>, Receiver<MmapMut>),

    action_popup_opened: bool,
    action_popup_text: String,
    actions: ActionsVec,
    current_action: usize,
}

impl Default for MyApp {
    fn default() -> Self {
        let mmap = MmapOptions::new().map_anon().unwrap();
        let current_file = Arc::new(RwLock::new(mmap));
        let mut tiles = egui_tiles::Tiles::default();
        let tabs = vec![];
        let root = tiles.insert_tab_tile(tabs);
        let tree = egui_tiles::Tree::new("tools_tree", root, tiles);
        let actions: ActionsVec = vec![
            (
                "String Finder".to_string(),
                Box::new(|file, tree| {
                    let tool = StringFinder::new(file.clone());
                    let boxed_tool = Box::new(tool);
                    MyApp::add_tool(tree, boxed_tool);
                }),
            ),
            (
                "Entropy Plot".to_string(),
                Box::new(|file, tree| {
                    let tool = tools::entropy_plot::EntropyPlot::new(file.clone());
                    let boxed_tool = Box::new(tool);
                    MyApp::add_tool(tree, boxed_tool);
                }),
            ),
            (
                "Frequency Image".to_string(),
                Box::new(|file, tree| {
                    let tool = tools::frequency_image::FrequencyImage::new(file.clone());
                    let boxed_tool = Box::new(tool);
                    MyApp::add_tool(tree, boxed_tool);
                }),
            ),
            (
                "Hex Viewer".to_string(),
                Box::new(|file, tree| {
                    let tool = tools::hex_viewer::HexViewer::new(file.clone());
                    let boxed_tool = Box::new(tool);
                    MyApp::add_tool(tree, boxed_tool);
                }),
            ),
            (
                "Format Explorer".to_string(),
                Box::new(|file, tree| {
                    let tool = tools::format_explorer::FormatExplorer::new(file.clone());
                    let boxed_tool = Box::new(tool);
                    MyApp::add_tool(tree, boxed_tool);
                }),
            ),
        ];

        let file_channel = std::sync::mpsc::channel();

        Self {
            current_file,
            tree,
            action_popup_opened: false,
            action_popup_text: String::new(),
            actions,
            current_action: 0,
            file_channel,
        }
    }
}

impl MyApp {
    fn notify_tools(&mut self, event: Event) {
        for (_, tile) in self.tree.tiles.iter_mut() {
            match tile {
                egui_tiles::Tile::Pane(pane) => pane.tool.notify(event),
                egui_tiles::Tile::Container(_) => {}
            }
        }
    }

    fn add_tool(tree: &mut egui_tiles::Tree<Pane>, tool: Box<dyn GaffrieTool>) {
        let pane = tree.tiles.insert_pane(Pane { tool });
        match tree.root {
            Some(root_tileid) => {
                let root_tile = tree.tiles.get_mut(root_tileid).unwrap();
                match root_tile {
                    egui_tiles::Tile::Container(container) => {
                        container.add_child(pane);
                    }
                    egui_tiles::Tile::Pane(_) => unimplemented!(),
                }
            }
            None => {
                tree.root = Some(pane);
            }
        }

        tree.make_active(|tileid, _| tileid == pane);
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(file) = self.file_channel.1.try_recv() {
            let mut lock = self.current_file.write();
            *lock = file;
            drop(lock);
            self.notify_tools(Event::FileChanged);
        }
        egui::SidePanel::left("tree").show(ctx, |ui| {
            if ui.button("Select file").clicked() {
                let sender = self.file_channel.0.clone();
                let task = rfd::AsyncFileDialog::new().pick_file();
                execute(async move {
                    let file = task.await;
                    if let Some(file) = file {
                        let path = file.path();

                        let file = File::options().write(true).read(true).open(path).unwrap();
                        let memfile =
                            unsafe { memmap2::MmapOptions::new().map_mut(&file).unwrap() };
                        let _ = sender.send(memfile);
                    }
                });
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut behavior = TreeBehavior {};
            self.tree.ui(&mut behavior, ui);
        });

        ctx.input(|i| {
            if i.key_pressed(egui::Key::P) && !self.action_popup_opened {
                self.action_popup_opened = true;
                self.current_action = 0;
                self.action_popup_text.clear();
            }
            if self.action_popup_opened {
                if i.key_pressed(egui::Key::ArrowDown) {
                    self.current_action = (self.current_action + 1).min(self.actions.len() - 1);
                } else if i.key_pressed(egui::Key::ArrowUp) {
                    self.current_action = self.current_action.saturating_sub(1);
                }
            }
        });
        if self.action_popup_opened {
            egui::Window::new("action_popup")
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .collapsible(false)
                .resizable(false)
                .title_bar(false)
                .show(ctx, |ui| {
                    let text_edit = egui::TextEdit::singleline(&mut self.action_popup_text)
                        .hint_text("Enter the name of a tool");
                    let response = ui.add(text_edit);
                    if response.lost_focus() {
                        self.action_popup_opened = false;
                    }
                    response.request_focus();
                    // TODO: Find a fuzzy search library to use here instead of string similarity
                    let mut filtered_actions = self
                        .actions
                        .iter()
                        .map(|a| {
                            (
                                a,
                                strsim::jaro_winkler(&a.0, self.action_popup_text.as_str()),
                            )
                        })
                        // .filter(|a| a.1 > 0.8)
                        .collect::<Vec<_>>();
                    filtered_actions
                        .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                    for (index, ((action_name, _), similarity_score)) in
                        filtered_actions.iter().enumerate()
                    {
                        if index == self.current_action {
                            let rect = ui
                                .label(format!("{} : {}", action_name, similarity_score))
                                .rect;
                            ui.painter().rect_stroke(
                                rect,
                                0.5,
                                egui::Stroke::new(1.0, egui::Color32::GREEN),
                            );
                        } else {
                            ui.label(format!("{} : {}", action_name, similarity_score));
                        }
                    }
                    ui.input(|i| {
                        if i.key_pressed(egui::Key::Enter) {
                            self.action_popup_opened = false;
                            let action = &filtered_actions[self.current_action].0 .1;
                            (action)(&self.current_file, &mut self.tree);
                        }
                    });
                });
        }

        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                let dropped_file = i.raw.dropped_files.first().unwrap();
                if let Some(path) = &dropped_file.path {
                    let file = File::options().write(true).read(true).open(path).unwrap();
                    let memfile = unsafe { memmap2::MmapOptions::new().map_mut(&file).unwrap() };
                    let mut lock = self.current_file.write();
                    *lock = memfile;
                    drop(lock);
                    self.notify_tools(Event::FileChanged);
                }
            }
        })
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        #[cfg(feature = "serde")]
        eframe::set_value(_storage, eframe::APP_KEY, &self);
    }
}

struct Pane {
    tool: Box<dyn GaffrieTool>,
}

struct TreeBehavior {}

impl egui_tiles::Behavior<Pane> for TreeBehavior {
    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        pane: &mut Pane,
    ) -> egui_tiles::UiResponse {
        pane.tool.ui(ui);
        egui_tiles::UiResponse::None
    }

    fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
        pane.tool.title().into()
    }

    fn simplification_options(&self) -> SimplificationOptions {
        let mut options = SimplificationOptions::default();
        options.all_panes_must_have_tabs = true;
        options
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn execute<F: Future<Output = ()> + Send + 'static>(f: F) {
    std::thread::spawn(move || futures::executor::block_on(f));
}

#[cfg(target_arch = "wasm32")]
fn execute<F: Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}
