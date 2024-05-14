mod cryptography;
mod gui;

use std::{error::Error, net::SocketAddr, sync::Arc};
use image::{ImageBuffer, Rgba, RgbaImage};
use quinn::{ConnectionError, Endpoint, RecvStream, SendStream};
use tokio::runtime::Runtime;
use tokio::sync::{mpsc, Mutex};

use gui::{start_gui};
use cryptography::configure_client;


struct SharedState {
    pub image_tx: mpsc::Sender<RgbaImage>,
}

fn main() -> Result<(), Box<dyn Error>> {


    let (tx, mut rx) = mpsc::channel(32);
    let shared_state = Arc::new(Mutex::new(SharedState { image_tx: tx }));

    // Clonare lo stato condiviso per il thread client
    let client_state = Arc::clone(&shared_state);

    let rt = Runtime::new()?;

    rt.block_on(async {

        // Avvia il client in un altro task.
        let client_task = tokio::spawn(async {
            let res = run_client("127.0.0.1:4433".parse().unwrap(), client_state).await;
            println!("{:?}", res);
        });


        let res = start_gui(rx).await;

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


async fn run_client(server_addr: SocketAddr, state: Arc<Mutex<SharedState>>) -> Result<(), Box<dyn Error>> {
    let mut endpoint = Endpoint::client("127.0.0.1:0".parse().unwrap())?;
    endpoint.set_default_client_config(configure_client());

    // connect to server
    let connection = endpoint
        .connect(server_addr, "localhost")
        .unwrap()
        .await
        .unwrap();

    println!("[client] Connected: addr={}", connection.remote_address());


    let (send, mut recv) = match connection.accept_bi().await{
        Ok((send, recv)) => (send, recv),
        Err(err) => {
            println!("{}", err.to_string());
            panic!()
        }
    };
    let mut img_vec: Vec<u8> = vec![];

    // loop{
    //     let mut buf = [0u8; 100_000];
    //     let n = recv.read(&mut buf).await;
    //     match n.unwrap() {
    //         Some(num) => {
    //             //println!("{:?}", String::from_utf8_lossy(&buf[..num]).len());
    //             for &item in &buf[..num] {
    //                 img_vec.push(item);
    //             }
    //             println!("{:?}", img_vec.len());
    //
    //             if img_vec.len() >= 3145728 {
    //                 println!("New image");
    //                 let img_res: Option<ImageBuffer<Rgba<u8>, &[u8]>> = ImageBuffer::from_raw(1024, 768, &*img_vec);
    //                 match img_res {
    //                     None => {
    //
    //                     }
    //                     Some(img) => {
    //                         let rgba_img = RgbaImage::from_vec(img.width(), img.height(), img.to_vec());
    //                         let state = state.lock().await;
    //                         state.image_tx.send(rgba_img.unwrap()).await.unwrap();
    //                     }
    //                 }
    //
    //                 img_vec = vec![];
    //             }
    //
    //         },
    //         None =>{}
    //     }
    //
    // }

    let mut i=0;
    loop{
        let mut buf = [0u8; 100_000];
        let n = recv.read(&mut buf).await;
        match n.unwrap() {
            Some(num) => {
                //println!("{:?}", String::from_utf8_lossy(&buf[..num]).len());
                for &item in &buf[..num] {
                    img_vec.push(item);
                    if img_vec.len() == 3145728{
                        println!("Img: {}", i);
                        i+=1;
                        let img_res: Option<ImageBuffer<Rgba<u8>, &[u8]>> = ImageBuffer::from_raw(1024, 768, &*img_vec);
                        match img_res {
                            None => {

                            }
                            Some(img) => {
                                let rgba_img = RgbaImage::from_vec(img.width(), img.height(), img.to_vec());
                                let state = state.lock().await;
                                state.image_tx.send(rgba_img.unwrap()).await.unwrap();
                            }
                        }

                        img_vec = vec![];
                    }
                }
            },
            None =>{}
        }

    }

    Ok(())
}
