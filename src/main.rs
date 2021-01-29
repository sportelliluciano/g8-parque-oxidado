extern crate rand;
extern crate std_semaphore;

mod args;
mod logger;
mod parque;
mod persona;
mod juego;

use std::{
    sync::Arc,
    time::Duration,
    thread::sleep,
};

use args::{parse_args, mostrar_ayuda, ParseArgsResult};
use logger::{Logger, TaggedLogger};
use parque::Parque;
use juego::Juego;
use persona::iniciar_hilos_personas;


fn main()  {
    if let Err(e) = real_main() {
        println!("ERROR: {}", e);
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

    let logger = Arc::new(if args.debug {
        Logger::new_to_file("debug.txt").expect("No se pudo crear el archivo de log.")
    } else {
        Logger::new_to_stdout()
    });

    let log = TaggedLogger::new("ADMIN", logger.clone());
    log.write(&format!("Iniciando simulación con: {}", args.as_str()));
    let parque = Arc::new(Parque::new(
        TaggedLogger::new("PARQUE", logger.clone()),
        args.capacidad_parque as usize,
        args.semilla as u64
    ));
    let juegos = args.costo_juegos.unwrap()
        .iter()
        .zip(args.capacidad_juegos.unwrap())
        .zip(args.duracion_juegos.unwrap())
        .enumerate()
        .map(|(id, ((costo, capacidad), duracion_ms))| Juego::new(
            TaggedLogger::new(&format!("JUEGO {}", id), logger.clone()),
            id, 
            Arc::clone(&parque), 
            *costo,
            capacidad,
            duracion_ms
        ))
        .collect::<Vec<Juego>>();
    
    // iniciar thread de juegos
    parque.registrar_juegos(juegos);

    let personas_threads = iniciar_hilos_personas(
        Arc::clone(&logger), 
        Arc::clone(&parque), 
        &args.presupuesto_personas
    );
    
    while parque.obtener_cantidad_gente_que_salio_del_parque() < args.presupuesto_personas.len() {
        sleep(Duration::from_millis(5000));
        log.write(&format!("Caja: $ {}, desperfectos: {}, gente adentro: {}", 
                 parque.obtener_caja(), 
                 parque.obtener_desperfectos(),
                 parque.obtener_genete_adentro()));
    }
    
    log.write("Salieron todos, cerrando el parque");
    parque.cerrar();
    log.write("Terminado");

    log.write(&format!("Caja final: $ {}, desperfectos: {}", 
                 parque.obtener_caja(), 
                 parque.obtener_desperfectos()));

    for persona in personas_threads {
        persona.join().expect("no se pudo joinear hilo de persona");
    }
    logger.close();
    Ok(())
}
