# Taller de Programacion

## Grupo

Nombre del grupo: **"Los Tallerines"**

- Valeria Brzoza (107523)
- Tomás Cichero (107973)
- Rodolfo Valentin Albornoz Tomé (107975)
- Francisco Antonio Pereyra (105666)

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