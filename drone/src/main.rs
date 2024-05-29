use lib::configuracion::Configuracion;
use drone::{aplicacion::AplicacionDron, estado::Estado};

fn main() {
    if let Err(e) = intentar_iniciar_aplicacion_dron() {
        eprintln!("Error al iniciar el dron: {}", e);
        std::process::exit(1);
    }
}

fn intentar_iniciar_aplicacion_dron() -> Result<(), Box<dyn std::error::Error>> {
    let estado = Estado::new();

    println!("Estado: {:?}", estado);

    let configuracion = Configuracion::desde_argv()?;
    println!("\nConfiguracion: {:?}", configuracion);
    
    let mut aplicacion_dron = AplicacionDron::new(configuracion, estado);
    println!("\nAplicacion del dron:: {:?}", aplicacion_dron);

    Ok(aplicacion_dron.iniciar()?)
}
