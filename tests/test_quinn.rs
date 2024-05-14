// #[cfg(test)]
// mod tests {
//     use std::thread::sleep;
//     use std::time::Duration;
//     use torre_rdp::server::{create_endpoint, start_server};
//
//     #[tokio::test]
//     async fn create_endpoint_test(){
//         let res = create_endpoint(None, None, None).await;
//         assert!(res.is_ok());
//         match res {
//             Ok(endpoint) => {
//                 println!("Endpoint creato correttamente");
//             }
//             Err(e) => {
//                 print!("{:?}", e);
//             }
//         }
//     }
//     #[tokio::test]
//     async fn start_server_test() {
//         let server_timeout = Duration::from_secs(2);
//
//         let res = tokio::time::timeout(server_timeout, start_server()).await;
//         // se va in errore è arrivato al timeout e va bene
//         assert!(res.is_err())
//     }
//
//     #[tokio::test]
//     async fn send_retrieve_image(){
//         // Supponiamo che start_server() non restituisca un valore.
//         tokio::spawn(async {
//             start_server().await;
//         });
//
//         // Attendi 100 millisecondi per dare tempo al server di avviarsi
//         sleep(Duration::from_millis(100)).await;
//
//         // Avvia il client - assicurati che la funzione del client sia definita per fare qualcosa di utile
//         start_client().await;
//     }
//
// }