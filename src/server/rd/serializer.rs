use image::{EncodableLayout, RgbaImage, RgbImage};

pub fn serialize_image(image: &RgbaImage) -> Vec<u8> {
    let width = image.width() as u16;
    let height = image.height() as u16;
    let start_x: u16 = 0;
    let start_y: u16 = 0;

    let custom_type: [u8; 1] = [0x0];
    let width_bytes: [u8; 2] = width.to_be_bytes();
    let height_bytes: [u8; 2] = height.to_be_bytes();
    let start_x_bytes: [u8; 2] = start_x.to_be_bytes();
    let start_y_bytes: [u8; 2] = start_y.to_be_bytes();

    let mut new_data = Vec::new();
    new_data.extend_from_slice(&custom_type);
    new_data.extend_from_slice(&width_bytes);
    new_data.extend_from_slice(&height_bytes);
    new_data.extend_from_slice(&start_x_bytes);
    new_data.extend_from_slice(&start_y_bytes);
    new_data.extend_from_slice(&image.as_bytes());

    new_data
}
