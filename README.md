# QRDP - Quic RDP

![image](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![image](https://img.shields.io/badge/Docker-2CA5E0?style=for-the-badge&logo=docker&logoColor=white)

A complete Remote Desktop application for servers and clients.

## Description

Quic RDP is a complete, fully working and secure RDP protocol. All is written in Rust and the comunication is based on Quic. In this repo is provided the **server** and also the **client** which can become a server as well for the web implementation.

QRDP is a solution that merge RDP, VNC and Guacamole in a single light and modern project.

The client works on Mac os, Windows and Linux. \
The server on Linux, using X11.

## Getting started

### Prerequisites

For the **server** you need the following packages (ubuntu 22):
```
apt-get install -y \
    xvfb \
    x11-apps \
    x11-xserver-utils \
    x11-utils \
    rustc \
    cargo \
    xfce4 \
    pkg-config libglib2.0-dev libcairo2-dev libjpeg-dev libgif-dev libgtk-4-dev libgtk-3-dev \
    dbus-x11 \
    libopencv-dev clang libclang-dev libpam0g-dev libpam0g libxcb-randr0-dev \
    xautomation
```

`rustc` and `cargo` are necessaries because is not compiled yet, so you compile and run it on your server.

### Installation

Clone the repo:

```
git clone "https://github.com/nTorre/RRDP.git"
```

Then enter in the directory and use this command:

```
cargo run --bin server
```

while for the **client**
```
cargo run --bin client
```

### Installation with docker

For testing purpose use Docker. TODO Documentazione docker

## Configurations

For config your server modify the `rrdp.config` file. It has to be one for user and you can leave it in the root directory of the project or specify the path on start.

- use custom display: true/false
- display n (0..), default ne crea uno, altrimenti cerca di connettersi a quello specificato
- desktop name, tipo di desktop, se non specificato xfce4. Fornire lista xfce4 -> startxfce4, openbox -> openbox-session
- dimension NxNxN, default 1024x768x24
- certs path path, defualt ne creo due al volo
- log path, defualt da vedere
- maxsize, massima size di ogni sezione di immagine, definisce in quante sottoimmagini suddividere, default da vedere
e tanti altri

## TODOs:

Cose da implementare a breve:
- gestione click mouse destro
- sistemare colori
- gestione errori completa
- prendere tutte le variabili dal file di configurazione
- specificare path file config
- login in diverse sessioni

Cose implementabili ma non farò a breve:
- libreria websocket
- invio file
- copia incolla testo
- combinazione tasti
- utenti in sola lettura


## Devs things Rrdp
### Dal server
- 1 byte per il tipo
    - 0 immagine
    - 1 testo copiato
    - 2 file 


Se immagine

| 0    | 1-2 | 3-4 | 5-6 | 7-8| 5-(5+widht * height * 3 OR 4) |
|------|-----|-----|-|-|-|
| type `u8`| width `u16` | height `u16`| start x `u16`| start y `u16`| data (w/wo alpha) `[u8]`|


### Dal client

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

