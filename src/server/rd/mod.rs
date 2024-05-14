// Aggiungere qua la logica del server vero e proprio

// impostare le fasi di connessione (scambio info, login (sesman), inizio condivisione)

// unificare handlers e runners qui. Nel main semplice chiamata al rd::RrdpRemoteDesktop::start(config)
// qualcosa del genere

mod handle;
pub use handle::*;