use std::sync::Arc;

use egui::{mutex::RwLock, FontSelection, TextStyle};

use super::GaffrieTool;

pub struct HexViewer {
    file: Arc<RwLock<Vec<u8>>>,
    text: String,
}

impl GaffrieTool for HexViewer {
    fn new(file_lock: Arc<RwLock<Vec<u8>>>) -> Self
    where
        Self: Sized,
    {
        Self {
            file: file_lock,
            text: String::new(),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        let text_style = TextStyle::Monospace;
        let row_height_sans_spacing = ui.text_style_height(&text_style);
        let fontid = &ui.style().text_styles[&text_style];
        let width = ui.fonts(|f| f.glyph_width(fontid, 'a'));
        let bytes_per_row = 16;
        let total_rows = self.file.read().len().div_ceil(bytes_per_row);
        egui::ScrollArea::vertical().show_rows(
            ui,
            row_height_sans_spacing,
            total_rows,
            |ui, row| {
                let mut offsets = Vec::new();
                let offset_length = 8;
                for r in row.clone() {
                    offsets.push(format!("{:08x}", r * bytes_per_row));
                }
                let offsets = offsets.join("\n");
                self.text.clear();
                let lock = self.file.read();
                for chunk in
                    lock[row.start * bytes_per_row..row.end * bytes_per_row].chunks(bytes_per_row)
                {
                    let chunk_len = chunk.len();
                    for (index, byte) in chunk.iter().enumerate() {
                        self.text.push_str(&format!("{:02x}", byte));
                        if index < chunk_len - 1 {
                            self.text.push(' ');
                        }
                    }
                    self.text.push('\n');
                }
                drop(lock);
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut offsets.as_str())
                            .font(FontSelection::Style(text_style.clone()))
                            .desired_width(width * (offset_length + 1) as f32),
                    );
                    ui.add(
                        egui::TextEdit::multiline(&mut self.text.as_str())
                            .font(FontSelection::Style(text_style))
                            .desired_width(f32::INFINITY),
                    );
                });
            },
        );
    }

    fn title(&self) -> String {
        "Hex Viewer".to_string()
    }

    fn notify(&mut self, _event: crate::Event) {}
}
