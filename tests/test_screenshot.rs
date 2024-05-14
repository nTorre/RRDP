#[cfg(test)]
mod tests {
    use std::time::Instant;
    use tokio::runtime::Runtime;
    use torre_rdp::screenshot::{capture_screen, capture_screen_fram, split_image_into_chunks};
    #[test]
    fn take_screenshot() {
        let rt = Runtime::new().unwrap(); // Crea un nuovo Tokio runtime

        rt.block_on(async {
            let start = Instant::now();
            match capture_screen_fram().await {
                Ok(image) => {
                    println!("Tempo per catturare: {:?}", start.elapsed());
                    println!("Dimensione immagine (height, width): {:?}px, {:?}px", image.height(), image.width());

                    // Nota l'uso di to_bytes() invece di into_bytes() qui
                    //println!("Dimensione immagine: {:?}", image.into_bytes().len());

                    // Puoi usare direttamente image qui senza riferirlo
                    match image.save("tmp/test.png") {
                        Ok(_) => println!("Immagine salvata con successo"),
                        Err(e) => eprintln!("Errore nel salvataggio dell'immagine: {:?}", e),
                    }
                },
                Err(e) => eprintln!("Errore nella cattura dello schermo: {:?}", e),
            }
            println!("Tempo totale: {:?}", start.elapsed());
        });
    }


    // #[test]
    // fn crop_screenshot() {
    //     let start = Instant::now();
    //     let image = capture_screen();
    //     println!("Tempo per catturare: {:?}", start.elapsed());
    //     println!("Dimensione immagine (height, width): {:?}px {:?}px", image.height(), image.width());
    //     println!("Dimensione immagine: {:?}", image.as_bytes().len());
    //
    //     let screen = split_image_into_chunks(4, 4, image.clone());
    //
    //     println!("Tempo per suddividere: {:?}", start.elapsed());
    //
    //     let mut i = 0;
    //     for chunk in screen.frames{
    //         let _ = chunk.frame.save(format!("tmp/{:?}.png", i));
    //         i+=1;
    //     }
    //     println!("Tempo per salvare: {:?}", start.elapsed());
    // }
    //
    // #[test]
    // fn compare_chunk(){
    //     // provare a prendere due frame uguali, ma con qualche pixel diverso
    //     let image = capture_screen();
    //     let screen = split_image_into_chunks(4, 4, image.clone());
    //     let chunk0 = &screen.frames[0];
    //     let chunk1 = &screen.frames[1];
    //
    //     let start = Instant::now();
    //     println!("{:?}", chunk0.compare(chunk0));
    //     println!("Tempo per comparare fino alla fine: {:?}", start.elapsed());
    //     println!("{:?}", chunk0.compare(chunk1));
    //     println!("Tempo per comparare: {:?}", start.elapsed());
    // }
}