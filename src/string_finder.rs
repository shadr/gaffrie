use std::sync::Arc;

use egui::{mutex::RwLock, Vec2b};
use egui_extras::Column;

use crate::Event;

pub struct FoundString {
    address: usize,
    length: usize,
    string: String,
}

#[derive(Default)]
pub enum StringsSorting {
    #[default]
    AddressAsc,
    AddressDesc,
    LengthAsc,
    LengthDesc,
}

pub struct StringFinder {
    file: Arc<RwLock<Vec<u8>>>,
    strings: Vec<FoundString>,
    current_sorting: StringsSorting,
}

impl StringFinder {
    pub fn new(file: Arc<RwLock<Vec<u8>>>) -> Self {
        Self {
            file,
            strings: Vec::new(),
            current_sorting: StringsSorting::default(),
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("Strings found: {}", self.strings.len()));
        let table = egui_extras::TableBuilder::new(ui)
            .striped(true)
            .resizable(false)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto().at_least(64.0))
            .column(Column::auto().at_least(32.0))
            .column(Column::remainder().clip(true))
            .vscroll(true)
            .auto_shrink(Vec2b::new(false, true));
        table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    if ui.button("Address").clicked() {
                        match self.current_sorting {
                            StringsSorting::AddressAsc => {
                                self.current_sorting = StringsSorting::AddressDesc
                            }
                            _ => self.current_sorting = StringsSorting::AddressAsc,
                        }
                        self.sort_strings();
                    }
                });
                header.col(|ui| {
                    if ui.button("Length").clicked() {
                        match self.current_sorting {
                            StringsSorting::LengthAsc => {
                                self.current_sorting = StringsSorting::LengthDesc
                            }
                            _ => self.current_sorting = StringsSorting::LengthAsc,
                        }
                        self.sort_strings();
                    }
                });
                header.col(|ui| {
                    ui.heading("String");
                });
            })
            .body(|body| {
                body.rows(20.0, self.strings.len(), |row_index, mut row| {
                    let string = &self.strings[row_index];
                    row.col(|ui| {
                        ui.label(string.address.to_string());
                    });
                    row.col(|ui| {
                        ui.label(string.length.to_string());
                    });
                    row.col(|ui| {
                        ui.add(egui::Label::new(&string.string).wrap(false).truncate(true));
                    });
                })
            });
    }

    pub fn notify_event(&mut self, event: Event) {
        match event {
            Event::FileChanged => {
                self.find_strings();
            }
        }
    }

    pub fn find_strings(&mut self) {
        log::info!("Finding strings");
        self.strings.clear();
        let file = self.file.read();
        let mut new_string = String::new();
        for (index, byte) in file.iter().enumerate() {
            let byte_is_readable_ascii = byte.is_ascii_graphic() || byte.is_ascii_whitespace();
            if byte_is_readable_ascii {
                new_string.push(*byte as char);
            } else if !new_string.is_empty() {
                if new_string.len() >= 5 {
                    self.strings.push(FoundString {
                        address: index - new_string.len(),
                        length: new_string.len(),
                        string: new_string,
                    });
                    new_string = String::new();
                } else {
                    new_string.clear();
                }
            }
        }
        drop(file);
        self.sort_strings();
    }

    fn sort_strings(&mut self) {
        match self.current_sorting {
            StringsSorting::AddressAsc => {
                self.strings.sort_by(|a, b| a.address.cmp(&b.address));
            }
            StringsSorting::AddressDesc => {
                self.strings.sort_by(|a, b| b.address.cmp(&a.address));
            }
            StringsSorting::LengthAsc => {
                self.strings.sort_by(|a, b| a.length.cmp(&b.length));
            }
            StringsSorting::LengthDesc => {
                self.strings.sort_by(|a, b| b.length.cmp(&a.length));
            }
        }
    }
}
