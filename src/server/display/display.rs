use x11rb::rust_connection::RustConnection;
pub type ScreenDimensions = (i32, i32, i32);

pub struct Display{
    connection: RustConnection,
    image: Vec<u8>
}

impl Display{
    pub fn new(display: String){

    }
}