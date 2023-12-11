pub mod entropy_plot;
pub mod frequency_image;
pub mod string_finder;

use egui::mutex::RwLock;

use crate::Event;

pub trait GaffrieTool {
    fn new(file_lock: std::sync::Arc<RwLock<Vec<u8>>>) -> Self
    where
        Self: Sized;
    fn ui(&mut self, ui: &mut egui::Ui);
    fn title(&self) -> String;
    fn notify(&mut self, event: Event);
}
