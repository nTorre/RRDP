git clone "https://..."
./install.sh rrdp-server o cargo run --bin server
rrdp-server [etc/rrdp.config]

rrdp.config uno per utente
- use custom display: true/false
- display n (0..), default ne crea uno, altrimenti cerca di connettersi a quello specificato
- desktop name, tipo di desktop, se non specificato xfce4. Fornire lista xfce4 -> startxfce4, openbox -> openbox-session
- dimension NxNxN, default 1024x768x24
- certs path path, defualt ne creo due al volo
- log path, defualt da vedere
- maxsize, massima size di ogni sezione di immagine, definisce in quante sottoimmagini suddividere, default da vedere
e tanti altri

TODO configurare systemctl

git clone "https://..."
prerequisiti: rust
cargo run --bin client
rrdp-server [~/rrdp.config]


Cose supportabili ma non implementate:
- più di un display per utente
- più di una connessione contemporanea per utente (solo una i scrittura, le altre in lettura) eventualmente impostare un timeout per evitare stalli
- webtransfer o websocket/desktop (creare sulla parte client un sistema per poter mostrare il risultato del server in maniera indipendente dal client )


