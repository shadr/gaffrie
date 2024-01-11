use std::sync::Arc;

use egui::{mutex::RwLock, FontSelection, TextStyle};
use memmap2::MmapMut;

use super::GaffrieTool;

pub struct HexViewer {
    file: Arc<RwLock<MmapMut>>,
    text: String,
    ascii_text: String,
}

impl GaffrieTool for HexViewer {
    fn new(file_lock: Arc<RwLock<MmapMut>>) -> Self
    where
        Self: Sized,
    {
        Self {
            file: file_lock,
            text: String::new(),
            ascii_text: String::new(),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        let text_style = TextStyle::Monospace;
        let row_height_sans_spacing = ui.text_style_height(&text_style) - 4.0;
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
                self.ascii_text.clear();
                let lock = self.file.read();
                let file_range_start = row.start * bytes_per_row;
                let file_range_end = (row.end * bytes_per_row).min(lock.len());
                let visible_file_range = &lock[file_range_start..file_range_end];
                for chunk in visible_file_range.chunks(bytes_per_row) {
                    let chunk_len = chunk.len();
                    for (index, byte) in chunk.iter().enumerate() {
                        self.text.push_str(&format!("{:02x}", byte));
                        if !byte.is_ascii_control() {
                            self.ascii_text.push(*byte as char);
                        } else {
                            self.ascii_text.push('.');
                        }
                        if index < chunk_len - 1 {
                            self.text.push(' ');
                        }
                    }
                    self.text.push('\n');
                    self.ascii_text.push('\n');
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
                            .font(FontSelection::Style(text_style.clone()))
                            .desired_width(width * (bytes_per_row * 3 - 1) as f32),
                    );
                    ui.add(
                        egui::TextEdit::multiline(&mut self.ascii_text.as_str())
                            .font(FontSelection::Style(text_style))
                            .desired_width(width * (bytes_per_row + 1) as f32 + 4.0),
                    )
                });
            },
        );
    }

    fn title(&self) -> String {
        "Hex Viewer".to_string()
    }

    fn notify(&mut self, _event: crate::Event) {}
}
