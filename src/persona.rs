use std::{
    sync::Arc,
    thread::JoinHandle
};

use crate::parque::Parque;

pub fn iniciar_hilos_personas(parque: Arc<Parque>, presupuestos: &Vec<u32>) -> Vec<JoinHandle<()>> {
    let mut handles = vec![];
    for (id, presupuesto_persona) in presupuestos.iter().enumerate() {
        let parque_child = parque.clone();
        let presupuesto_persona = *presupuesto_persona;
        handles.push(std::thread::spawn(move || {
            persona_thread_main(parque_child, presupuesto_persona, id);
        }));
    }
    handles
}

fn persona_thread_main(parque: Arc<Parque>, presupuesto_inicial: u32, id: usize) {
    let mut presu = presupuesto_inicial;
    println!("[Persona {}] Esperando para entrar al parque", id);
    parque.ingresar_persona();
    println!("[Persona {}] Entre al parque con {} pesos", id, presu);
    while presu > 0 {
        let juego = match parque.elegir_juego_random(presu) {
            Ok(juego) => juego,
            Err(_) => break
        };
        
        println!("[Persona {}] Entrando al juego {}.", id, juego.id());
        juego.entrar(); // Bajar el sem
        println!("[Persona {}] Tengo $ {} y voy a pagar $ {}", id, presu, juego.precio());
        presu -= juego.precio();  // ????
        parque.pagar(juego.precio());
        println!("[Persona {}] Pagué el juego {} (me quedan $ {}).", id, juego.id(), presu);
        juego.jugar();  // Esperar la barrera
        println!("[Persona {}] Jugué al juego {} y salí.", id, juego.id());
        //juego.salir();  // Subir el sem
        //println!("[Persona {}] Salí del juego {}.", id, juego.id());
    }
    println!("[Persona {}] No me alcanza para ningun juego (me quedaron $ {})", id, presu);
    parque.salir_persona();
    println!("[Persona {}] Me fui del parque", id);
}