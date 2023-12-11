use std::sync::Arc;

use egui::{mutex::RwLock, Vec2b};
use egui_plot::{Line, PlotPoints};

use super::GaffrieTool;

pub struct EntropyPlot {
    file: Arc<RwLock<Vec<u8>>>,
    points: Vec<[f64; 2]>,
}

impl GaffrieTool for EntropyPlot {
    fn new(file_lock: Arc<RwLock<Vec<u8>>>) -> Self
    where
        Self: Sized,
    {
        let mut this = Self {
            file: file_lock,
            points: Vec::new(),
        };
        this.regenerate_plot();
        this
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        let plot = egui_plot::Plot::new("entropy_plot")
            .auto_bounds_x()
            .auto_bounds_y()
            .show_grid(Vec2b::new(false, false))
            .allow_drag(false)
            .allow_zoom(false)
            .allow_scroll(false)
            .allow_boxed_zoom(false);
        plot.show(ui, |plot_ui| {
            let plot_points: PlotPoints = PlotPoints::new(self.points.clone());
            plot_ui.line(Line::new(plot_points));
        });
    }

    fn title(&self) -> String {
        "Entropy Plot".to_string()
    }

    fn notify(&mut self, event: crate::Event) {
        match event {
            crate::Event::FileChanged => self.regenerate_plot(),
        }
    }
}

impl EntropyPlot {
    fn regenerate_plot(&mut self) {
        let file = self.file.read();
        let number_of_points = 1000;
        let chunk_size = file.len() / number_of_points;
        let points = file
            .chunks(chunk_size)
            .map(Self::entropy_of_slice)
            .enumerate()
            .map(|(i, b)| [i as f64, b])
            .collect::<Vec<_>>();
        self.points = points;
    }

    fn entropy_of_slice(slice: &[u8]) -> f64 {
        let mut entropy = 0.0;
        let mut counts = [0; 256];
        for b in slice {
            counts[*b as usize] += 1;
        }
        for count in counts {
            if count > 0 {
                let p = count as f64 / slice.len() as f64;
                entropy -= p * p.log2();
            }
        }
        entropy
    }
}
