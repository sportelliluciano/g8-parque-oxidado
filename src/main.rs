extern crate rand;
extern crate std_semaphore;

mod args;
mod parque;
mod persona;
mod juego;
mod semaforo;

use std::{
    sync::Arc,
    thread::sleep,
    time::Duration
};

use args::{parse_args, mostrar_ayuda, ParseArgsResult};
use parque::Parque;
use juego::Juego;
use persona::iniciar_hilos_personas;


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
    let parque = Arc::new(Parque::new(
        args.capacidad_parque as usize,
        args.costo_juegos
            .iter()
            .enumerate()
            .map(|(id, c)| Juego::new(id, *c))
            .collect::<Vec<Juego>>()
    ));
    let personas = iniciar_hilos_personas(parque.clone(), &args.presupuesto_personas);
    
    while parque.obtener_cantidad_gente_que_salio_del_parque() < args.presupuesto_personas.len() {
        sleep(Duration::from_millis(5000));
        println!("[ADMIN] Caja: $ {}, desperfectos: {}, gente adentro: {}", 
                 parque.obtener_caja(), 
                 parque.obtener_desperfectos(),
                 parque.obtener_genete_adentro());
    }
    
    println!("[ADMIN] Salieron todos, cerrando el parque");
    parque.cerrar();
    println!("[ADMIN] Terminado");

    for persona in personas {
        persona.join().expect("no se pudo joinear hilo de persona");
    }
    Ok(())
}
