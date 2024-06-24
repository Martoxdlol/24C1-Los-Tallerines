# Taller de Programacion

## Grupo

Nombre del grupo: **"Los Tallerines"**

- Valeria Brzoza (107523)
- Tomás Cichero (107973)


## Como usar

### Iniciar Servidor de NATS

```bash
# Valores por defecto
cargo run --bin messaging-server
# Parámetros de configuración
cargo run --bin messaging-server -- puerto=4222 direccion=0.0.0.0 cuentas=users.csv
# Archivo de configuración
cargo run --bin messaging-server -- config=config.txt
# Mixto
cargo run --bin messaging-server -- puerto=4222 config=config.txt
# No mostrar logs de todo en la consola
cargo run --bin messaging-server -- noinfo=true
# Iniciar server con soporte TLS (puerto por defecto: 8222)
cargo run --bin messaging-server -- cert=fullchain.pem key=privkey.pem
# Iniciar server con soporte TLS, puerto custom
cargo run --bin messaging-server -- cert=fullchain.pem key=privkey.pem puerto_tls=4223
```

**Configuración: config.txt**
```txt
puerto=4222
direccion=0.0.0.0
cuentas=users.csv
```

**Cuentas: users.csv**
```csv
1,admin,1234
2,usuario,1234
```

### Iniciar Sistema Central de Cámaras

```bash
# Valores por defecto
cargo run --bin cameras
# Parámetros de configuración
cargo run --bin cameras -- direccion=localhost puerto=4222 camaras=camaras.csv
# Archivo de configuración
cargo run --bin cameras -- config=config.txt
```

### Iniciar App de Monitoreo

Antes de ejecutar el monitoreo, se debe tener en cuenta que el sistema central de cámaras y el servidor de mensajería deben estar corriendo.

Si se utiliza Linux se deben instalar algunas dependencias para que funcione la librería de UI:
```bash
sudo apt update -y && sudo apt-get install -y libclang-dev libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev
```

```bash
# Valores por defecto
cargo run --bin monitoring
# Parámetros de configuración
cargo run --bin monitoring -- direccion=localhost puerto=4222
# Archivo de configuración
cargo run --bin monitoring -- config=config.txt
```

## Como testear

`cargo test`

Para testear un paquete en particular:
`cargo test --bin <nombre>`

## Pruebas de Rendimiento

Se puede probar el rendimiento del server de nats siguiendo su documentación oficial
[natsbench](https://docs.nats.io/using-nats/nats-tools/nats_cli/natsbench).

**Envio de mensajes (pub)**

```sh
nats bench foo --pub 1 --size 16
```

**Envio de mensajes y suscripciones (pub/sub)**

```sh
nats bench foo --pub 1 --sub 1 --size 16
```

**Multiples Publicadores y Multiples Suscriptores**

```sh
nats bench foo --pub 8 --sub 8 --size 16
```

## Funciones Soportadas de JetStream

**Crear stream**

```sh
nats stream create ordenes
# ? Subjects ordenes.nuevas ordenes.finalizadas
```

El resto de las opciones de creación de stream no son soportadas y serán ignoradas

**Ver info de stream**

```sh
nats stream create ordenes
nats stream info
# ? Select a Stream ordenes
```

**Listar streams**

```sh
nats stream ls
╭──────────────────────────────────────────────────────────────────────────────╮
│                                    Streams                                   │
├─────────┬─────────────┬─────────────────────┬──────────┬──────┬──────────────┤
│ Name    │ Description │ Created             │ Messages │ Size │ Last Message │
├─────────┼─────────────┼─────────────────────┼──────────┼──────┼──────────────┤
│ ordenes │             │ 2024-06-21 09:38:40 │ 0        │ 0 B  │ 0s           │
╰─────────┴─────────────┴─────────────────────┴──────────┴──────┴──────────────╯
```

**Eliminar stream**

```sh
nats stream delete
# ? Select a Stream ordenes
# ? Really delete Stream ordenes Yes
```

**Crear consumer**

Por el momento solo se soportan consumers pull.

```sh
nats consumer create
# ? Consumer name ordenes-nuevas
# ? Delivery target (empty for Pull Consumers) 
# ? Start policy (all, new, last, subject, 1h, msg sequence) new
# ? Acknowledgment policy explicit
# ? Replay policy instant
# ? Filter Stream by subjects (blank for all) ordenes.nuevas
# ? Maximum Allowed Deliveries -1
# ? Maximum Acknowledgments Pending 0
# ? Select a Stream ordenes
```

Solo se soporta cero acks pendiente, no tienen limites de mensajes y la start policy es siempre new.
Estan soportados el filtro por stream subjects. 
El resto de las opciones serán ignoradas.

**Info consumer**

```sh
nats consumer info
# ? Select a Stream ordenes
# ? Select a Consumer ordenes-nuevas
```

**Listar consumers**

```sh
nats consumer ls
# ? Select a Stream ordenes
```

**Publicar a stream**

```sh
nats pub ordenes.nuevas "1,Fulano,10.00"
```

**Leer del consumer**

```sh
nats consumer next ordenes ordenes-nuevas
# --- subject: _INBOX.MVusmWdqSD4uWjjyROSOjq.qLcHDjJM reply: $JS.ACK.ordenes.ordenes-nuevas.yr9xD6AIARpfBVyqw2ScbP

# 1,Fulano,0.00

# Acknowledged message
```

**Eliminar consumer**

```sh
nats consumer delete
? Select a Stream ordenes
? Select a Consumer ordenes-nuevas
? Really delete Consumer ordenes > ordenes-nuevas Yes
```
