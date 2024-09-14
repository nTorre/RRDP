use std::borrow::Cow;
use std::error::Error;
use std::io::{Cursor, IoSlice};
use std::sync::{Arc};
use std::thread::sleep;
use std::time::{Duration, Instant};
use image::{DynamicImage, EncodableLayout, ImageBuffer};
use image::ImageFormat::Png;
use log::info;
use tokio::runtime::Runtime;
use crate::errors::connection_errors::ConnectionError;
use crate::handlers::{Handle};
use quinn::{Connection, Endpoint, ReadDatagram, RecvStream, SendStream, ServerConfig as QuinnServerConfig};
use rustls::internal::msgs::codec::u24;
use rustls::ServerConfig;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{Semaphore, Mutex, Notify};
use tokio::time::interval;
use x11rb::connection::{Connection as XConnection, RequestConnection};
use x11rb::cookie::VoidCookie;
use x11rb::protocol::xproto::{BUTTON_PRESS_EVENT, ButtonPressEvent, ButtonReleaseEvent, ChangeWindowAttributesAux, ConnectionExt as TConnectionExt, CreateWindowAux, EventMask, GetInputFocusReply, GrabMode, GrabStatus, ImageFormat, KeyButMask, QueryPointerReply, Rectangle, Screen, send_event, SendEventRequest, Time, Timestamp, Window, WindowClass};
use x11rb::rust_connection::RustConnection;
use x11rb::x11_utils::Serialize;
use chrono::Utc;
use x11rb::protocol::{damage, Event, xproto};
use x11rb::protocol::damage::{ConnectionExt, ReportLevel};
use x11rb::protocol::xtest::{self, ConnectionExt as XTestExt};
use crate::display::RrdpDisplay;
use crate::handlers;


// To implement
pub struct RrdpServer {
    config: ServerConfig
}

impl RrdpServer{

    pub fn new(server_crypto: ServerConfig) -> Self {
        Self{
            config: server_crypto,
        }
    }
    pub fn start_listening<T: Handle + Send + Sync + 'static>(self, handler: T)
                                                              -> Result<(), ConnectionError> {

        let rt = match Runtime::new(){
            Ok(rt) => rt,
            Err(_) => return Err(ConnectionError::RunTimeError())
        };

        rt.block_on(async {
            let address = "0.0.0.0:4433"; // sistemare perché lo devo prendere dal file config
            let server_config = QuinnServerConfig::with_crypto(Arc::new(self.config));
            let parsed_address = match address.parse() {
                Ok(parsed_address) => parsed_address,
                Err(_) => return Err(ConnectionError::AddressParsingError(address.to_string()))
            };

            let endpoint = Endpoint::server(server_config, parsed_address).unwrap();
            info!("Server listening on {}", endpoint.local_addr().unwrap());
            // Accept incoming connections
            while let Some(connecting) = endpoint.accept().await {
                let connection = match connecting.await {
                    Ok(connection) => connection,
                    Err(e) => {return Err(
                        ConnectionError::CreateConnectionError(e.to_string()))}
                };
                info!("New connection from {}", connection.remote_address());
                tokio::spawn(
                handler.handle_connection(connection).await
                );
            }
            Ok(())
        })
    }
}

