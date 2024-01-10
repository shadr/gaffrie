use std::sync::Arc;

use egui::{mutex::RwLock, Color32, Vec2b};
use egui_extras::Column;

use super::GaffrieTool;
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
    string_min_length: usize,
}

impl GaffrieTool for StringFinder {
    fn new(file_lock: Arc<RwLock<Vec<u8>>>) -> Self {
        let mut this = Self {
            file: file_lock,
            strings: Vec::new(),
            current_sorting: StringsSorting::default(),
            string_min_length: 5,
        };
        this.find_strings();
        this
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("Strings found: {}", self.strings.len()));
        if ui
            .add(egui::DragValue::new(&mut self.string_min_length))
            .changed()
        {
            self.find_strings();
        }
        let table = egui_extras::TableBuilder::new(ui)
            .striped(true)
            .resizable(false)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto().at_least(64.0))
            .column(Column::auto().at_least(32.0))
            .column(Column::remainder().clip(true))
            .vscroll(true)
            .auto_shrink(Vec2b::new(false, true));
        let mut address_button_label = "Address".to_string();
        match self.current_sorting {
            StringsSorting::AddressAsc => {
                address_button_label.push_str(" 🔼");
            }
            StringsSorting::AddressDesc => {
                address_button_label.push_str(" 🔽");
            }
            _ => (),
        }
        let mut length_button_label = "Length".to_string();
        match self.current_sorting {
            StringsSorting::LengthAsc => {
                length_button_label.push_str(" 🔼");
            }
            StringsSorting::LengthDesc => {
                length_button_label.push_str(" 🔽");
            }
            _ => (),
        }
        table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    if ui
                        .add(egui::Button::new(address_button_label).fill(Color32::TRANSPARENT))
                        .clicked()
                    {
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
                    if ui
                        .add(egui::Button::new(length_button_label).fill(Color32::TRANSPARENT))
                        .clicked()
                    {
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
                    ui.label("String");
                });
            })
            .body(|body| {
                body.rows(20.0, self.strings.len(), |mut row| {
                    let string = &self.strings[row.index()];
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

    fn title(&self) -> String {
        "Strings".to_string()
    }

    fn notify(&mut self, event: Event) {
        match event {
            Event::FileChanged => {
                self.find_strings();
            }
        }
    }
}

impl StringFinder {
    pub fn find_strings(&mut self) {
        self.strings.clear();
        let file = self.file.read();
        let mut new_string = String::new();
        for (index, byte) in file.iter().enumerate() {
            let byte_is_readable_ascii = byte.is_ascii_graphic() || byte.is_ascii_whitespace();
            if byte_is_readable_ascii {
                new_string.push(*byte as char);
            } else if !new_string.is_empty() {
                if new_string.len() >= self.string_min_length {
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
