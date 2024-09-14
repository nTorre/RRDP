use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use image::EncodableLayout;
use quinn::{Connection, ReadExactError};
use tokio::task;
use tokio::time::interval;
use x11rb::protocol::damage::{ConnectionExt as XConnectionExt};
use x11rb::protocol::xproto::ConnectionExt;
use crate::display::RrdpDisplay;
use crate::errors::connection_errors::ConnectionError;
use crate::rd::serialize_image;

pub trait Handle{
    async fn handle_connection(&self, connection: Connection)
        -> Pin<Box<dyn Future<Output = Result<(), ConnectionError>> + Send>>;
}


pub struct RrdpHandler{

}

impl RrdpHandler {
    pub fn new() -> Self {
        Self {}
    }

    fn _login(self){

    }
}

impl Handle for RrdpHandler{
    async fn handle_connection(&self, connection: Connection)
        -> Pin<Box<dyn Future<Output = Result<(), ConnectionError>> + Send>> {
        Box::pin(async move {
            let (mut send, mut recv) = match connection.open_bi().await{
                Ok((send, recv)) => (send, recv),
                Err(err) => {
                    return Err(ConnectionError::OpenBiError(err.to_string()))
                }
            };

            // TODO: unify display

            // Gestione input
            tokio::spawn(async move {

                let display = match RrdpDisplay::new(None){
                    Ok(display) => display,
                    Err(err) => {println!("Error"); panic!()}       // TODO: gestione errori
                };


                loop {

                    let mut type_byte = [0u8; 1];
                    let n = recv.read(&mut type_byte).await;

                    let packet_type = u8::from_le_bytes(type_byte);


                    if packet_type == 0{
                        let mut bytes = [0u8; 8];
                        let n = recv.read(&mut bytes).await;
                        // TODO modificare lettura (primo byte per capire il tipo di pacchetto e quanti altri bytes leggere. Vedi lettura immagini)
                        let x = i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                        let y = i32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);

                        if x!=0 && y!=0{
                            display.warp_pointer(
                                x as i16, y as i16
                            );
                        }
                    } else if packet_type==1 {
                        let mut button = [0u8; 1];
                        let n = recv.read(&mut button).await;
                        let button_num = u8::from_le_bytes(button);
                        display.press_button(button_num);
                        // display.press_keyboard();
                    }

                    else if packet_type==2 {
                        let mut button = [0u8; 1];
                        let n = recv.read(&mut button).await;
                        let button_num = u8::from_le_bytes(button);
                        display.release_button(button_num);
                    }

                    else if packet_type==3 {
                        let mut buf = [0u8; 1];
                        let n = recv.read(&mut buf).await;
                        let len = u8::from_le_bytes(buf);

                        let mut buf = vec![0u8; len as usize];

                        match recv.read_exact(&mut buf).await{
                            Ok(_) => {
                                for byte in &buf {
                                    display.press_keyboard(*byte);
                                }

                                for byte in &buf {
                                    display.release_keyboard(*byte);
                                }
                            }
                            Err(_) => {}
                        }

                        // press tastiera
                    }

                }


            });



            // Gestione output

            let mut display = match RrdpDisplay::new(None){
                Ok(display) => display,
                Err(_) => {panic!()}            // TODO gestione errore
            };

            let damage_id = display.setup_damage();
            display.set_event_mask();

            // Acquisisci l'intera schermata e inviala subito dopo la connessione
            let full_image = &display.get_full_image().await.unwrap();
            let binding = serialize_image(&full_image);
            let bytes = binding.as_bytes();
            send.write_all(&bytes).await.unwrap();

            let mut interval = interval(Duration::from_millis(10));



            loop {
                interval.tick().await;
                let damage_events = display.fetch_damage_events().unwrap();
                if !damage_events.is_empty() {
                    let bounding_box = display.calculate_bounding_box(&damage_events);
                    let mut width = bounding_box.width;
                    let mut height = bounding_box.height;
                    let mut start_x = bounding_box.x;
                    let mut start_y = bounding_box.y;

                    // Processa qui l'ultimo damage notify
                    let img = display.get_image(
                        start_x, start_y,
                        width, height
                    ).await.unwrap();       // TODO: gestione errori

                    let bytes = serialize_image(&img);

                    send.write_all(bytes.as_bytes()).await.unwrap();

                    // Riparare il danno
                    let _ = display.damage_subtract();      // TODO: gestione errori

                }
            }


            Ok(())
        })
    }
}