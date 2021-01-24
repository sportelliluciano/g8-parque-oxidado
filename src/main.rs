extern crate rand;

mod args;

use args::{parse_args, mostrar_ayuda, ParseArgsResult};


fn main()  {
    match real_main() {
        Err(e) => println!("ERROR: {}", e),
        _ => {}
    }
}

fn real_main() -> Result<(), String> {
    let args = match parse_args() {
        ParseArgsResult::Ok(args) => args,
        ParseArgsResult::MostrarAyuda => {
            mostrar_ayuda();
            return Ok(())
        },
        ParseArgsResult::Error(e) => {
            mostrar_ayuda();
            return Err(e);
        }
    };

    println!("[ADMIN] Iniciando simulación con: {}", args.as_str());
    Ok(())
}
