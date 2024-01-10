pub mod entropy_plot;
pub mod format_explorer;
pub mod frequency_image;
pub mod hex_viewer;
pub mod string_finder;

use egui::mutex::RwLock;
use memmap2::MmapMut;

use crate::Event;

pub trait GaffrieTool {
    fn new(file_lock: std::sync::Arc<RwLock<MmapMut>>) -> Self
    where
        Self: Sized;
    fn ui(&mut self, ui: &mut egui::Ui);
    fn title(&self) -> String;
    fn notify(&mut self, event: Event);
}
