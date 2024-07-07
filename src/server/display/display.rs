use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ChangeWindowAttributesAux, ConnectionExt as XprotoConnectionExt, EventMask, ImageFormat, KeyPressEvent, Rectangle, Screen, Window};
use x11rb::rust_connection::RustConnection;
use std::rc::Rc;
use std::sync::Arc;
use image::ImageBuffer;
use x11rb::errors::ConnectError;
use x11rb::protocol::composite::{ConnectionExt, Redirect};
use x11rb::protocol::damage::{ConnectionExt as DamageConnectionExt, NotifyEvent, ReportLevel};
use x11rb::protocol::{Event, xproto};
use x11rb::protocol::xtest::ConnectionExt as XConnectionExt;
use crate::errors::connection_errors::ConnectionError;
pub type ScreenDimensions = (i32, i32, i32);

pub struct RrdpDisplay {
    connection: Arc<RustConnection>,
    screen_num: usize,
    damage_notify: Option<NotifyEvent>,
    damage_id: Option<u32>
}

impl RrdpDisplay {
    pub fn new(display: Option<&str>) -> Result<RrdpDisplay, ConnectionError> {
        let (connection, screen_num) = match x11rb::connect(display){
            Ok(res) => res,
            Err(_) => {return Err(ConnectionError::CreateConnectionError("".to_string()))}
        };

        Ok(RrdpDisplay {
            connection: Arc::new(connection),
            screen_num,
            damage_notify: None,
            damage_id: None
        })
    }

    pub fn set_event_mask(&self){

        match self.connection.change_window_attributes(
            self.get_screen().root,
            &ChangeWindowAttributesAux::new().event_mask(
                EventMask::BUTTON_PRESS | EventMask::BUTTON_RELEASE | EventMask::POINTER_MOTION |
                    EventMask::SUBSTRUCTURE_NOTIFY | EventMask::SUBSTRUCTURE_REDIRECT
            )
        ){
            Ok(_) => {}
            Err(e) => {println!("{}", e)}       // TODO: gestione errori
        };
    }

    pub fn setup_damage(&mut self)->u32{
        let damage_id = self.connection.generate_id().unwrap();
        // TODO: errore se damage non esiste
        let damage_version = &self.connection.damage_query_version(1, 1).unwrap().reply().unwrap();
        self.connection.damage_create(damage_id, self.get_screen().root, ReportLevel::RAW_RECTANGLES).unwrap();
        self.damage_id = Some(damage_id);
        damage_id
    }

    pub fn setup_xcomposite(&self){
        // Inizializzare l'estensione XComposite
        self.connection.composite_query_version(0, 4).unwrap().reply().unwrap();

        // Reindirizzare la root window per includere tutte le finestre sovrapposte
        self.connection.composite_redirect_subwindows(self.get_screen().root, x11rb::protocol::composite::Redirect::AUTOMATIC).unwrap();

    }
    pub fn get_screen(&self) -> &Screen {
        &self.connection.setup().roots[self.screen_num]
    }

    pub fn get_root(&self) -> &Screen {
        &self.connection.setup().roots[self.screen_num]
    }

    pub async fn get_image(&self, start_x: i16, start_y: i16, width: u16, height: u16) -> Result<image::RgbaImage, String>{

        let image = self.connection.get_image(
            ImageFormat::Z_PIXMAP,
            self.get_screen().root,
            start_x,
            start_y,
            width,
            height,
            !0,
        ).unwrap().reply().unwrap();

        let buffer = image.data;
        let img: ImageBuffer<image::Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(width as u32, height as u32, buffer).unwrap();
        let img = image::RgbaImage::from(img);

        Ok(img)
    }

    pub async fn get_full_image(&self) -> Result<image::RgbaImage, String>{
        Self::get_image(&self,
                        0,
                        0,
                        self.get_screen().width_in_pixels,
                        self.get_screen().height_in_pixels ).await
    }

    pub fn warp_pointer(&self, x: i16, y: i16){
        let _ = self.connection.warp_pointer(
            x11rb::NONE, // src_window
            self.get_screen().root,  // dst_window
            x, y, // src_x, src_y
            self.get_screen().width_in_pixels, self.get_screen().height_in_pixels, // src_width, src_height
            x, y // dst_x, dst_y
        );

        match self.connection.flush(){
            Ok(_) => {}
            Err(err) => {
                println!("Errore"); //TODO: gestire errori
            }
        }
    }

    pub fn fetch_damage_events(&self) -> Result<Vec<Rectangle>, ConnectionError> {
        let mut damage_events = Vec::new();
        loop {
            match self.connection.poll_for_event() {
                Ok(Some(Event::DamageNotify(damage_notify))) => {
                    damage_events.push(damage_notify.area);
                },
                Ok(Some(_)) => continue, // Ignora altri eventi
                Ok(None) => break,
                Err(e) => return Err(ConnectionError::CreateConnectionError("Failed to poll event".to_string())),
            }
        }
        Ok(damage_events)
    }



    pub fn calculate_bounding_box(&self, damage_areas: &[Rectangle]) -> Rectangle {
        let mut bounding_box = Rectangle {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        };

        for area in damage_areas {
            let top_left_x = bounding_box.x.min(area.x);
            let top_left_y = bounding_box.y.min(area.y);
            let bottom_right_x = (bounding_box.x as i32 + bounding_box.width as i32).max((area.x as i32 + area.width as i32)) as i16;
            let bottom_right_y = (bounding_box.y as i32 + bounding_box.height as i32).max((area.y as i32 + area.height as i32)) as i16;

            bounding_box = Rectangle {
                x: top_left_x,
                y: top_left_y,
                width: (bottom_right_x - top_left_x) as u16,
                height: (bottom_right_y - top_left_y) as u16,
            };
        }

        bounding_box
    }

    pub fn damage_subtract(&self)->Result<(), &str>{
        let damage_id = match self.damage_id{
            None => {return Err("Errore")}          // TODO: gestione errori
            Some(damage_id) => damage_id
        };

        &self.connection.damage_subtract(damage_id, x11rb::NONE, x11rb::NONE).unwrap();
        Ok(())

    }

    // TODO: create enum of buttons
    pub fn press_button(&self, button: u8){

        let pos = self.get_pointer_position();

        self.connection.xtest_fake_input(4, button, 0, self.get_screen().root, pos.0, pos.1, 0);

        // Simula il rilascio del pulsante
        // self.connection.xtest_fake_input(5, button, 0, self.get_screen().root, 0, 0, 0);

        // Assicurati che i comandi siano stati inviati
        self.connection.flush();

    }

    pub fn release_button(&self, button: u8){

        let pos = self.get_pointer_position();

        // Simula il rilascio del pulsante
        self.connection.xtest_fake_input(5, button, 0, self.get_screen().root, pos.0, pos.1, 0);

        // Assicurati che i comandi siano stati inviati
        self.connection.flush();

    }

    // TODO: gestione errori
    pub fn get_pointer_position(&self) -> (i16, i16) {
        let reply = self.connection.query_pointer(self.get_screen().root).unwrap().reply().unwrap();

        // Estrarre le coordinate del puntatore
        let (x, y) = (reply.root_x, reply.root_y);

        return (x, y);
    }


    /// TEST
    pub fn press_keyboard(&self, key: u8){

        &self.connection.xtest_fake_input(2, key, 0, self.get_screen().root, 0, 0, 0)
            .unwrap();
        &self.connection.flush().unwrap();

    }

    pub fn release_keyboard(&self, key: u8){
        // Simula il rilascio del tasto
        &self.connection.xtest_fake_input(3, key, 0, self.get_screen().root, 0, 0, 0)
            .unwrap();
        &self.connection.flush().unwrap();
    }
}

