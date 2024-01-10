use std::sync::Arc;

use egui::{mutex::RwLock, Color32};
use memmap2::MmapMut;

use super::GaffrieTool;

pub struct FrequencyImage {
    file: Arc<RwLock<MmapMut>>,
    texture: Option<egui::TextureHandle>,
}

impl GaffrieTool for FrequencyImage {
    fn new(file_lock: Arc<RwLock<MmapMut>>) -> Self
    where
        Self: Sized,
    {
        let mut this = Self {
            file: file_lock,
            texture: None,
        };
        this.reload_image();
        this
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        let texture = match &self.texture {
            Some(texture) => texture,
            None => {
                self.texture = Some(ui.ctx().load_texture(
                    "freq_image",
                    egui::ColorImage::example(),
                    Default::default(),
                ));
                self.reload_image();
                self.texture.as_ref().unwrap()
            }
        };
        let image = egui::Image::new(texture).shrink_to_fit();
        ui.add(image);
    }

    fn title(&self) -> String {
        "Byte pairs frequency".to_string()
    }

    fn notify(&mut self, event: crate::Event) {
        match event {
            crate::Event::FileChanged => {
                self.reload_image();
            }
        }
    }
}

impl FrequencyImage {
    pub fn reload_image(&mut self) {
        if let Some(texture) = &mut self.texture {
            let file = self.file.read();
            let mut image = egui::ColorImage::new([256, 256], Color32::BLACK);
            let mut counts = vec![0; 256 * 256];
            for byte_pair in file.windows(2) {
                counts[byte_pair[0] as usize * 256 + byte_pair[1] as usize] += 1;
            }
            let max_count = *counts.iter().max().unwrap();
            let pixels = counts
                .into_iter()
                .map(|count| (((count as f32).ln() / (max_count as f32).ln()) * 255.) as u8)
                .map(|c| Color32::from_rgb(c, c, c))
                .collect::<Vec<_>>();
            image.pixels = pixels;

            texture.set(image, egui::TextureOptions::NEAREST);
        }
    }
}
