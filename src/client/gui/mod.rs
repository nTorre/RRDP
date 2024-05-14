#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use eframe::egui;
use image::RgbaImage;
use tokio::sync::mpsc::Receiver;

pub async fn start_gui(mut image_rx: Receiver<RgbaImage>) -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([600.0, 800.0]),
        ..Default::default()
    };


    eframe::run_native(
        "Image Viewer",
        options,
        Box::new(move |cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::new(MyApp {
                texture: None,
                image_rx,
            })
        }),
    )
}

struct MyApp {
    texture: Option<egui::TextureHandle>,
    image_rx: Receiver<RgbaImage>,
}

impl MyApp{
    fn new(){

    }
}


impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                if let Ok(image) = self.image_rx.try_recv() {
                    let (h, w) = image.dimensions();
                    let texture = egui::ColorImage::from_rgba_unmultiplied([h as usize, w as usize], &image);
                    let texture = ctx.load_texture("loaded_image", texture, egui::TextureOptions::LINEAR);
                    self.texture = Some(texture);
                }

                if let Some(texture) = &self.texture {
                    ui.image(texture);
                    ctx.request_repaint();  // Richiede un nuovo frame, aggiornando l'UI
                } else {
                    ui.label("No image loaded");
                }
            });
        });
    }

}