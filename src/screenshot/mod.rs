use image::{DynamicImage, GenericImageView, ImageBuffer, Pixel};
use scrap::{Capturer, Display};
use tokio::task;

pub struct Screen{
    pub frames: Vec<ScreenFrame>
}

pub struct ScreenFrame{
    pub frame: DynamicImage
}

impl ScreenFrame{

    fn new(frame: DynamicImage)->ScreenFrame{
        ScreenFrame{
            frame
        }
    }
    pub fn compare(&self, other: &ScreenFrame) ->bool{
        let (width1, height1) = self.frame.dimensions();
        let (width2, height2) = other.frame.dimensions();

        // Se le dimensioni delle immagini sono diverse, le immagini sono sicuramente diverse.
        if width1 != width2 || height1 != height2 {
            return false;
        }

        // Controlla ogni secondo pixel di ogni seconda riga per confrontare le immagini.
        // Ogni pixel 11s, ogni 2 3s, ogni 3 1,7s
        for y in (0..height1).step_by(2) {
            for x in (0..width1).step_by(2) {
                let pixel1 = self.frame.get_pixel(x, y).to_rgba();
                let pixel2 = other.frame.get_pixel(x, y).to_rgba();

                if pixel1 != pixel2 {
                    return false; // Trovata una differenza, le immagini non sono uguali.
                }
            }
        }

        true
    }
}

pub async fn capture_screen() -> Result<DynamicImage, Box<dyn std::error::Error + Send + Sync>> {
    let image = task::spawn_blocking(|| {
        let display = Display::primary().expect("Couldn't find primary display.");
        let mut capturer = Capturer::new(display).expect("Couldn't begin capture.");
        let (width, height) = (capturer.width() as u32, capturer.height() as u32);

        let frame = loop {
            match capturer.frame() {
                Ok(frame) => break frame,
                Err(error) => {
                    if error.kind() != std::io::ErrorKind::WouldBlock {
                        panic!("Errore durante la cattura dello schermo: {:?}", error);
                    }
                }
            }

            // Attenzione: sleep in un contesto bloccante; considera l'uso di un approccio diverso se necessario.
            std::thread::sleep(std::time::Duration::from_millis(10));
        };

        let mut rgba_frame = Vec::with_capacity(frame.len());
        for chunk in frame.chunks_exact(4) {
            rgba_frame.extend_from_slice(&[chunk[2], chunk[1], chunk[0], chunk[3]]);
        }

        DynamicImage::ImageRgba8(ImageBuffer::from_raw(width, height, rgba_frame).unwrap())
    }).await?;

    Ok(image)
}

pub async fn capture_screen_fram() -> Result<DynamicImage, Box<dyn std::error::Error + Send + Sync>> {
    let image = task::spawn_blocking(|| {
        let display = Display::primary().expect("Couldn't find primary display.");
        let mut capturer = Capturer::new(display).expect("Couldn't begin capture.");
        let (width, height) = (capturer.width() as u32, capturer.height() as u32);

        let frame = loop {
            match capturer.frame() {
                Ok(frame) => break frame,
                Err(error) => {
                    if error.kind() != std::io::ErrorKind::WouldBlock {
                        panic!("Errore durante la cattura dello schermo: {:?}", error);
                    }
                }
            }

            // Attenzione: sleep in un contesto bloccante; considera l'uso di un approccio diverso se necessario.
            std::thread::sleep(std::time::Duration::from_millis(10));
        };

        let mut rgba_frame = Vec::with_capacity(frame.len());
        for chunk in frame.chunks_exact(4) {
            rgba_frame.extend_from_slice(&[chunk[2], chunk[1], chunk[0], chunk[3]]);
        }

        let dyn_image = DynamicImage::ImageRgba8(ImageBuffer::from_raw(width, height, rgba_frame).unwrap());
        let cropped_image= dyn_image.crop_imm(0, 0, width/40, height/40);
        println!("Dimensione immagine: {:?}", &cropped_image.as_bytes().len());
        println!("Dimensione immagine originale: {:?}", &dyn_image.as_bytes().len());

        cropped_image
    }).await?;

    Ok(image)
}

pub fn split_image_into_chunks(rows: u32, cols: u32, image: DynamicImage) -> Screen {
    let (width, height) = image.dimensions();
    let chunk_width = width / cols;
    let chunk_height = height / rows;

    // Verifica che le dimensioni siano divisibili per il numero di righe e colonne
    assert!(width % cols == 0 && height % rows == 0, "Le dimensioni dell'immagine non possono essere divise equamente per le righe e colonne specificate.");

    let mut chunks = Vec::with_capacity((rows * cols) as usize);

    for row in 0..rows {
        for col in 0..cols {
            let x = col * chunk_width;
            let y = row * chunk_height;

            // Crop l'immagine senza modificarla
            let chunk = ScreenFrame{frame: image.crop_imm(x, y, chunk_width, chunk_height)};
            chunks.push(chunk);
        }
    }

    Screen{
        frames: chunks,
    }
}
