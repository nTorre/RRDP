mod cryptography;
mod gui;

use std::{error::Error, net::SocketAddr, sync::Arc};
use std::time::Instant;
use image::{EncodableLayout, ImageBuffer, Rgb, Rgba, RgbaImage, RgbImage};
use quinn::Endpoint;
use tokio::runtime::Runtime;
use tokio::sync::{mpsc, Mutex};
use x11rb::x11_utils::Serialize;

use gui::{start_gui};
use cryptography::configure_client;


struct SharedState {
    pub image_tx: mpsc::Sender<(RgbaImage, (u16, u16))>,
}

struct MouseSharedState{
    pub muse_rx: mpsc::Receiver<(i32, i32)>
}

struct MouseClickSharedState{
    pub muse_press_rx: mpsc::Receiver<(u8, u8)>
}

struct KeyboardSharedState{
    pub keyboard_rx: mpsc::Receiver<Vec<u8>>
}

fn main() -> Result<(), Box<dyn Error>> {

    // mouse control
    let (mtx, mut mrx) = mpsc::channel(32);
    let mouse_shared_state = Arc::new(Mutex::new(MouseSharedState { muse_rx: mrx }));
    let mouse_client_state = Arc::clone(&mouse_shared_state);

    let (mtx_press, mut mrx_press) = mpsc::channel(32);
    let mouse_click_shared_state = Arc::new(Mutex::new(MouseClickSharedState { muse_press_rx: mrx_press }));
    let mouse_press_client_state = Arc::clone(&mouse_click_shared_state);

    let (tx, mut rx) = mpsc::channel(32);
    let shared_state = Arc::new(Mutex::new(SharedState { image_tx: tx }));
    let client_state = Arc::clone(&shared_state);

    let (keyboard_tx, mut keyboard_rx) = mpsc::channel(32);
    let keyboard_shared_state = Arc::new(Mutex::new(KeyboardSharedState { keyboard_rx }));
    let keyboard_client_state = Arc::clone(&keyboard_shared_state);

    let rt = Runtime::new()?;

    rt.block_on(async {

        // Avvia il client in un altro task.
        let client_task = tokio::spawn(async {
            let res = run_client("38.60.249.227:4433".parse().unwrap(), client_state, mouse_client_state, mouse_press_client_state, keyboard_client_state).await;
            println!("{:?}", res);
        });


        let res = start_gui(rx, mtx, mtx_press, keyboard_tx).await;

        match res {
            Ok(_) => {
                client_task.abort();
            }
            Err(_) => {}
        }

        client_task.await; // Nota: Doppio '?' per gestire Result di Result.

    });


    Ok(())
}


async fn run_client(server_addr: SocketAddr, state: Arc<Mutex<SharedState>>, mouse: Arc<Mutex<MouseSharedState>>, mouse_press: Arc<Mutex<MouseClickSharedState>>, keyboard_shared_state: Arc<Mutex<KeyboardSharedState>>) -> Result<(), Box<dyn Error>> {
    let mut endpoint = Endpoint::client("0.0.0.0:0".parse().unwrap())?;
    endpoint.set_default_client_config(configure_client());

    // connect to server
    let connection = endpoint
        .connect(server_addr, "localhost")
        .unwrap()
        .await
        .unwrap();

    println!("[client] Connected: addr={}", connection.remote_address());


    let (mut send, mut recv) = match connection.accept_bi().await{
        Ok((send, recv)) => (send, recv),
        Err(err) => {
            println!("{}", err.to_string());
            panic!()
        }
    };


    let send = Arc::new(Mutex::new(send));

    // Primo task
    let send_clone = Arc::clone(&send);

    tokio::spawn(async move{
        let mut step = 0;
        while let coord = mouse.lock().await.muse_rx.recv().await {
            match coord {
                None => {}
                Some(coord) => {
                    if step == 0 {
                        let custom_type: [u8; 1] = [0x0];
                        let bind = coord.serialize();

                        let mut new_data = Vec::new();
                        new_data.extend_from_slice(&custom_type);
                        new_data.extend(bind);

                        let mut send = send_clone.lock().await;
                        send.write_all(new_data.as_bytes()).await;

                        step = 0;
                    }

                    else {
                        step+=1;
                    }

                }
           }
       }

    });

    let send_clone = Arc::clone(&send);
    tokio::spawn(async move{
        while let coord = mouse_press.lock().await.muse_press_rx.recv().await {
            match coord {
                None => {}
                Some(res) => {
                    // send click
                    let custom_type: [u8; 2] = [res.0, res.1];
                    // let custom_type: [u8; 2] = [1, res.1];

                    let mut new_data = Vec::new();
                    new_data.extend_from_slice(&custom_type);

                    let mut send = send_clone.lock().await;
                    send.write_all(new_data.as_bytes()).await;
                    println!("{:?}", res)
                }
            }
        }
    });

    let send_clone = Arc::clone(&send);
    tokio::spawn(async move{
        while let keys = keyboard_shared_state.lock().await.keyboard_rx.recv().await {
            match keys {
                None => {}
                Some(mut res) => {
                    let mut to_send = vec![];
                    let len = res.len() - 1;
                    let type_byte = res.remove(0);
                    let other_bytes = res.as_ref();
                    let head = [type_byte, len as u8];
                    to_send.extend_from_slice(&head);
                    to_send.extend_from_slice(&other_bytes);
                    println!("{:?}", to_send);
                    let mut send = send_clone.lock().await;
                    send.write_all(to_send.as_bytes()).await;
                }
            }
        }
    });

    let mut global_buffer = Vec::new();
    let mut cursor = 0;
    let mut img_number = 0;
    let mut fps_time = Instant::now();

    loop {

        // Assicurati di avere abbastanza dati per leggere gli header
        while global_buffer.len() - cursor < 9 {
            let mut buf = [0u8; 1000];
            let n = recv.read(&mut buf).await.unwrap().unwrap();
            global_buffer.extend_from_slice(&buf[..n]);
        }

        // Estrai i dati degli header
        let buf_type = global_buffer[cursor];
        let width = u16::from_be_bytes([global_buffer[cursor + 1], global_buffer[cursor + 2]]);
        let height = u16::from_be_bytes([global_buffer[cursor + 3], global_buffer[cursor + 4]]);
        let start_x = u16::from_be_bytes([global_buffer[cursor + 5], global_buffer[cursor + 6]]);
        let start_y = u16::from_be_bytes([global_buffer[cursor + 7], global_buffer[cursor + 8]]);
        cursor += 9;


        let dim = (width as u64) * (height as u64) * 4;


        // Assicurati di avere abbastanza dati per l'immagine completa
        while global_buffer.len() - cursor < dim as usize {
            let mut buf = [0u8; 1000];
            let n = recv.read(&mut buf).await.unwrap().unwrap();
            global_buffer.extend_from_slice(&buf[..n]);
        }

        // println!("Received dim bytes");

        // Elabora l'immagine
        let img_res: Option<ImageBuffer<Rgba<u8>, &[u8]>> = ImageBuffer::from_raw(width as u32, height as u32, &global_buffer[cursor..cursor + dim as usize]);
        match img_res {
            None => {
                println!("errore");
            }
            Some(img) => {
                let rgba_img = RgbaImage::from_vec(img.width(), img.height(), img.to_vec());
                let state = state.lock().await;
                state.image_tx.send((rgba_img.unwrap(), (start_x, start_y))).await;
                img_number+=1;
                if img_number % 10 == 0 {
                    let secs = fps_time.elapsed().as_millis() as f64 / 1000.0;
                    println!("fps: {}", img_number as f64/secs);
                    img_number=0;
                    fps_time = Instant::now();
                }
                // println!("Created new image");
            }
        }
        cursor += dim as usize;

        // println!("Ciao");

        // Rimuovi i dati processati dal buffer, se necessario
        if cursor >= global_buffer.len() {
            global_buffer.clear();
            cursor = 0;
        } else {
            global_buffer.drain(..cursor);
            cursor = 0;
        }

    }


    // let mut i=0;
    // let mut fps_time = Instant::now();
    //
    // let mut buf_type = [0u8; 1];
    // let mut buf_width = [0u8; 2];
    // let mut buf_height = [0u8; 2];
    // let mut buf_start_x = [0u8; 2];
    // let mut buf_start_y = [0u8; 2];
    //
    //
    // loop {
    //     // Leggi gli header per la nuova immagine
    //     recv.read_exact(&mut buf_type).await?;
    //     recv.read_exact(&mut buf_width).await?;
    //     recv.read_exact(&mut buf_height).await?;
    //     recv.read_exact(&mut buf_start_x).await?;
    //     recv.read_exact(&mut buf_start_y).await?;
    //
    //     // Converti i buffer in valori utili
    //     let width = u16::from_be_bytes([buf_width[0], buf_width[1]]);
    //     let height = u16::from_be_bytes([buf_height[0], buf_height[1]]);
    //
    //     println!("WIDTH: {}", width);
    //     println!("HEIGHT: {}", height);
    //
    //     // Calcola la dimensione dell'immagine
    //     let dim = (width as u64) * (height as u64) * 4;
    //
    //     // Prepara il vettore per i dati dell'immagine
    //     let mut img_vec: Vec<u8> = Vec::with_capacity(dim as usize);
    //
    //     // Continua a leggere finché non hai ricevuto tutti i dati dell'immagine
    //     while (img_vec.len() as u64) < dim {
    //         let mut buf = vec![0u8; 100_000];
    //         let n = recv.read(&mut buf).await.unwrap().unwrap();
    //         img_vec.extend_from_slice(&buf[..n]);
    //     }
    //
    //     // Elabora l'immagine ricevuta
    //     let img_res: Option<ImageBuffer<Rgba<u8>, &[u8]>> = ImageBuffer::from_raw(width as u32, height as u32, &*img_vec);
    //     match img_res {
    //         None => {
    //             println!("errore");
    //         }
    //         Some(img) => {
    //             println!("{}", img.len());
    //             let rgba_img = RgbaImage::from_vec(img.width(), img.height(), img.to_vec());
    //             let state = state.lock().await;
    //             state.image_tx.send(rgba_img.unwrap()).await.unwrap();
    //         }
    //     }
    //
    //     // Resetta il buffer dell'immagine per la prossima iterazione
    //     img_vec.clear();
    // }


    Ok(())
}
