use std::sync::Arc;

use egui::{mutex::RwLock, FontSelection, TextStyle};

use super::GaffrieTool;

pub struct HexViewer {
    file: Arc<RwLock<Vec<u8>>>,
    rows: Vec<String>,
}

impl GaffrieTool for HexViewer {
    fn new(file_lock: Arc<RwLock<Vec<u8>>>) -> Self
    where
        Self: Sized,
    {
        let mut this = Self {
            file: file_lock,
            rows: Vec::new(),
        };
        this.file_changed();
        this
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        let text_style = TextStyle::Monospace;
        let row_height_sans_spacing = ui.text_style_height(&text_style);
        let fontid = &ui.style().text_styles[&text_style];
        let width = ui.fonts(|f| f.glyph_width(fontid, 'a'));
        egui::ScrollArea::vertical().show_rows(
            ui,
            row_height_sans_spacing,
            self.rows.len(),
            |ui, row| {
                let mut offsets = Vec::new();
                let offset_length = 8;
                for r in row.clone() {
                    offsets.push(format!("{:08x}", r * 16));
                }
                let offsets = offsets.join("\n");
                let text = &self.rows[row].join("\n");
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut offsets.as_str())
                            .font(FontSelection::Style(text_style.clone()))
                            .desired_width(width * (offset_length + 1) as f32),
                    );
                    ui.add(
                        egui::TextEdit::multiline(&mut text.as_str())
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

    fn notify(&mut self, event: crate::Event) {
        match event {
            crate::Event::FileChanged => {
                self.file_changed();
            }
        }
    }
}

impl HexViewer {
    pub fn file_changed(&mut self) {
        let lock = self.file.write();
        let mut rows = Vec::new();
        for chunk in lock.chunks(16) {
            let row = chunk
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<_>>()
                .join(" ");
            rows.push(row);
        }
        self.rows = rows;
    }
}
