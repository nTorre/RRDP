#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use eframe::egui;
use egui::Modifiers;
use crate::gui::egui::Key;
use image::{RgbaImage, RgbImage};
use tokio::sync::mpsc::{Receiver, Sender};

pub async fn start_gui(mut image_rx: Receiver<(RgbaImage, (u16, u16))>,
                       sender: Sender<(i32, i32)>,
                       mouse_press_sender: Sender<(u8, u8)>,
                       key_sender: Sender<Vec<u8>>) -> Result<(), eframe::Error> {
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
                image_rx: image_rx,
                sender,
                mouse_press_sender,
                last_pos: (0, 0),
                full_image: RgbaImage::new(1024, 768),
                key_sender
            })
        }),
    )
}

struct MyApp {
    texture: Option<egui::TextureHandle>,
    image_rx: Receiver<(RgbaImage, (u16, u16))>,
    sender: Sender<(i32, i32)>,
    mouse_press_sender: Sender<(u8, u8)>,
    // keyboard_press_sender: Sender<u8>,
    last_pos: (i32, i32),
    full_image: RgbaImage,
    key_sender: Sender<Vec<u8>>
}

impl MyApp{
    fn new(){

    }
}


impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let pos = ctx.input((|i| i.pointer.interact_pos()));
        // let primary_down = ctx.input((|i| i.pointer.primary_down()));
        let primary_pressed = ctx.input(|i| i.pointer.primary_pressed());
        let primary_released = ctx.input(|i| i.pointer.primary_released());
        let scroll = ctx.input(|i| i.smooth_scroll_delta);

        if scroll.length()!=0.{
            // send scroll
            println!("{:?}", scroll);
        }

        let mut pressed_keys: Vec<u8> = vec![];
        let mut released_keys: Vec<u8> = vec![];
        let mut modifiers: Vec<u8> = vec![];

        // add modifiers
        if ctx.input(|i| i.modifiers.shift){
            modifiers.push(50);
            // println!("Shift")
        }

        if ctx.input(|i| i.modifiers.ctrl){
            modifiers.push(37);
            // println!("Ctrl")
        }

        if ctx.input(|i| i.modifiers.command){
            modifiers.push(37);
            // println!("Command")
        }

        for key in Key::ALL.iter() {
            if ctx.input(|i| i.key_pressed(*key)) {
                // println!("{:?} premuto", key);
                pressed_keys.push(to_x11_keycode(key))
                // add pressed
            }
            if ctx.input(|i| i.key_released(*key)) {
                // println!("{:?} rilasciato", key);
                released_keys.push(to_x11_keycode(key))
                // add released
            }
        }

        if !pressed_keys.is_empty(){
            let mut to_send: Vec<u8> = vec![];
            to_send.push(3);
            to_send.append(modifiers.as_mut());
            to_send.append(pressed_keys.as_mut());
            self.key_sender.try_send(to_send);
        }

        if !released_keys.is_empty(){
            let mut to_send: Vec<u8> = vec![];
            to_send.push(4);
            to_send.append(modifiers.as_mut());
            to_send.append(released_keys.as_mut());
            self.key_sender.try_send(to_send);
        }


        if primary_pressed {
            // send press
            self.mouse_press_sender.try_send((1,1));
        }

        if primary_released {
            // send release
            self.mouse_press_sender.try_send((2,1));
        }

        match pos {
            Some(pos)=> {
                let new_pos = (pos.x as i32, pos.y as i32);
                if self.last_pos != new_pos{
                    self.sender.try_send((pos.x as i32, pos.y as i32));
                    self.last_pos = new_pos;
                }
            }
            _ => {}
        }


        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                if let Ok(image) = self.image_rx.try_recv() {
                    let (w, h) = image.0.dimensions();
                    let (start_x, start_y) = image.1;

                    for i in 0..w {
                        for j in 0..h {
                            let px = image.0.get_pixel(i, j);
                            self.full_image.put_pixel(start_x as u32 + i, start_y as u32 + j, *px);
                        }
                    }

                    let texture = egui::ColorImage::from_rgba_premultiplied([1024, 768], &self.full_image);
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

pub fn to_x11_keycode(key: &Key) -> u8 {
    match key {
        Key::ArrowDown => 116,
        Key::ArrowLeft => 113,
        Key::ArrowRight => 114,
        Key::ArrowUp => 111,
        Key::Escape => 9,
        Key::Tab => 23,
        Key::Backspace => 22,
        Key::Enter => 36,
        Key::Space => 65,
        Key::Insert => 118,
        Key::Delete => 119,
        Key::Home => 110,
        Key::End => 115,
        Key::PageUp => 112,
        Key::PageDown => 117,
        Key::Copy => 0, // Implement as needed (no direct keycode)
        Key::Cut => 0, // Implement as needed (no direct keycode)
        Key::Paste => 0, // Implement as needed (no direct keycode)
        Key::Colon => 58,
        Key::Comma => 59,
        Key::Backslash => 51,
        Key::Slash => 61,
        Key::Pipe => 0, // Specific keycode or combination needed
        Key::Questionmark => 0, // Specific keycode or combination needed
        Key::OpenBracket => 34,
        Key::CloseBracket => 35,
        Key::Backtick => 49,
        Key::Minus => 20,
        Key::Period => 60,
        Key::Plus => 21,
        Key::Equals => 21,
        Key::Semicolon => 47,
        Key::Num0 => 19,
        Key::Num1 => 10,
        Key::Num2 => 11,
        Key::Num3 => 12,
        Key::Num4 => 13,
        Key::Num5 => 14,
        Key::Num6 => 15,
        Key::Num7 => 16,
        Key::Num8 => 17,
        Key::Num9 => 18,
        Key::A => 38,
        Key::B => 56,
        Key::C => 54,
        Key::D => 40,
        Key::E => 26,
        Key::F => 41,
        Key::G => 42,
        Key::H => 43,
        Key::I => 31,
        Key::J => 44,
        Key::K => 45,
        Key::L => 46,
        Key::M => 58,
        Key::N => 57,
        Key::O => 32,
        Key::P => 33,
        Key::Q => 24,
        Key::R => 27,
        Key::S => 39,
        Key::T => 28,
        Key::U => 30,
        Key::V => 55,
        Key::W => 25,
        Key::X => 53,
        Key::Y => 29,
        Key::Z => 52,
        Key::F1 => 67,
        Key::F2 => 68,
        Key::F3 => 69,
        Key::F4 => 70,
        Key::F5 => 71,
        Key::F6 => 72,
        Key::F7 => 73,
        Key::F8 => 74,
        Key::F9 => 75,
        Key::F10 => 76,
        Key::F11 => 95,
        Key::F12 => 96,
        Key::F13 => 191,
        Key::F14 => 192,
        Key::F15 => 193,
        Key::F16 => 194,
        Key::F17 => 195,
        Key::F18 => 196,
        Key::F19 => 197,
        Key::F20 => 198,
        Key::F21 => 199,
        Key::F22 => 200,
        Key::F23 => 201,
        Key::F24 => 202,
        Key::F25 => 203,
        Key::F26 => 204,
        Key::F27 => 205,
        Key::F28 => 206,
        Key::F29 => 207,
        Key::F30 => 208,
        Key::F31 => 209,
        Key::F32 => 210,
        Key::F33 => 211,
        Key::F34 => 212,
        Key::F35 => 213,
        _ => 0, // Default for unspecified keys
    }
}