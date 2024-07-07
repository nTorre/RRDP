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

# Pacchetti Rrdp
## Dal server
- 1 byte per il tipo
    - 0 immagine
    - 1 testo copiato
    - 2 file 


Se immagine

| 0    | 1-2 | 3-4 | 5-6 | 7-8| 5-(5+widht * height * 3 OR 4) |
|------|-----|-----|-|-|-|
| type `u8`| width `u16` | height `u16`| start x `u16`| start y `u16`| data (w/wo alpha) `[u8]`|


## Dal client

- 1 byte per il tipo
    - 0 mouse move
    - 1 mouse press
    - 2 mouse release
    - 3 keyboard press
    - 4 keyboard release
    - 5 paste text
    - 6 copy text

Se mouse move

| 0         | 1-2     | 3-4     |
|-----------|---------|---------|
| type `u8` | x `u16` | y `u16` |

Se mouse press

| 0             | 1               |
|---------------|-----------------|
| type `u8` [1] | button `u8` [1] |

Se mouse release

| 0             | 1               |
|---------------|-----------------|
| type `u8` [2] | button `u8` [1] |

Se keyboard press

| 0             | 1                   | 2 - n+2         |
|---------------|---------------------|-----------------|
| type `u8` [3] | btn number `u8` [n] | btn o modifiers |

TODO:
- gestione click mouse destro
- gestione tastiera
- sistemare colori
- gestione errori completa
- prendere tutte le variabili dal file di configurazione
- login in diverse sessioni

TODO che non farò
- libreria websocket
- invio file
- copia incolla testo
- combinazione tasti
- utenti in sola lettura
