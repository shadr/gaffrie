use std::sync::Arc;

use egui::mutex::RwLock;

use super::GaffrieTool;

mod formats;

pub trait FileFormatUi {
    fn ui(&mut self, ui: &mut egui::Ui);
}

impl FileFormatUi for () {
    fn ui(&mut self, _ui: &mut egui::Ui) {}
}

pub struct FormatExplorer {
    file: Arc<RwLock<Vec<u8>>>,
    parsed: Box<dyn FileFormatUi>,
}

impl GaffrieTool for FormatExplorer {
    fn new(file_lock: std::sync::Arc<egui::mutex::RwLock<Vec<u8>>>) -> Self
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
        self.parsed.ui(ui);
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
