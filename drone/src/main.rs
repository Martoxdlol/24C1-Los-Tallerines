use lib::{configuracion::Configuracion, dron::Dron};

fn main() {
    if let Err(e) = intentar_iniciar_aplicacion_dron() {
        eprintln!("Error al iniciar el dron: {}", e);
        std::process::exit(1);
    }
}

fn intentar_iniciar_aplicacion_dron() -> Result<(), Box<dyn std::error::Error>> {
    let configuracion: Configuracion = Configuracion::desde_argv()?;
    println!("\nConfiguracion: {:?}", configuracion);

    let mut dron = Dron::new(configuracion);
    println!("\nDron: {:?}", dron);

    Ok(dron.iniciar()?)
}
