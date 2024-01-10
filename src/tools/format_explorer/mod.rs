use std::sync::Arc;

use egui::mutex::RwLock;
use memmap2::MmapMut;

use super::GaffrieTool;

mod formats;

pub trait FileFormatUi {
    fn ui(&mut self, ui: &mut egui::Ui, name: &str);
}

impl FileFormatUi for () {
    fn ui(&mut self, _ui: &mut egui::Ui, _name: &str) {}
}

impl FileFormatUi for u8 {
    fn ui(&mut self, ui: &mut egui::Ui, name: &str) {
        ui.horizontal(|ui| {
            ui.label(name);
            ui.label(": ");
            ui.label(self.to_string());
        });
    }
}

impl FileFormatUi for u16 {
    fn ui(&mut self, ui: &mut egui::Ui, name: &str) {
        ui.horizontal(|ui| {
            ui.label(name);
            ui.label(": ");
            ui.label(self.to_string());
        });
    }
}

impl FileFormatUi for u32 {
    fn ui(&mut self, ui: &mut egui::Ui, name: &str) {
        ui.horizontal(|ui| {
            ui.label(name);
            ui.label(": ");
            ui.label(self.to_string());
        });
    }
}

impl FileFormatUi for u64 {
    fn ui(&mut self, ui: &mut egui::Ui, name: &str) {
        ui.horizontal(|ui| {
            ui.label(name);
            ui.label(": ");
            ui.label(self.to_string());
        });
    }
}

pub struct FormatExplorer {
    file: Arc<RwLock<MmapMut>>,
    parsed: Box<dyn FileFormatUi>,
}

impl GaffrieTool for FormatExplorer {
    fn new(file_lock: Arc<RwLock<MmapMut>>) -> Self
    where
        Self: Sized,
    {
        let mut this = Self {
            file: file_lock,
            parsed: Box::new(()),
        };
        this.file_changed();
        this
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        self.parsed.ui(ui, "file");
    }

    fn title(&self) -> String {
        "Format Explorer".to_string()
    }

    fn notify(&mut self, event: crate::Event) {
        match event {
            crate::Event::FileChanged => self.file_changed(),
        }
    }
}

impl FormatExplorer {
    fn file_changed(&mut self) {
        let lock = self.file.read();
        let elf = formats::elf::ElfFormat::new(&*lock);
        self.parsed = Box::new(elf);
    }
}
