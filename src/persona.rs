use std::{
    sync::Arc,
    thread::JoinHandle
};

use crate::parque::Parque;
use crate::juego::Juego;

pub fn iniciar_hilos_personas(parque: Arc<Parque>, presupuestos: &Vec<u32>) -> Vec<JoinHandle<()>> {
    let mut handles = vec![];
    for (id, presupuesto_persona) in presupuestos.iter().enumerate() {
        let parque_child = parque.clone();
        let presupuesto_persona = *presupuesto_persona;
        let mut persona = Persona::new(id, presupuesto_persona);
        handles.push(std::thread::spawn(move || {
            persona.visitar_parque(parque_child);
        }));
    }
    handles
}

pub struct Persona {
    pub id: usize,
    presupuesto: u32,
}

impl Persona {
    pub fn new(id: usize, presupuesto: u32) -> Self {
        Self {
            id,
            presupuesto,
        }
    }

    pub fn pagar_juego(&mut self, juego: &Juego) -> u32 {
        let presupuesto_restante = self.presupuesto - juego.precio;
        println!("[Persona {}] Pagando juego {}. Tenía $ {} y pagué $ {}, me quedan {}", self.id, juego.id, self.presupuesto, juego.precio, presupuesto_restante);
        self.presupuesto = presupuesto_restante;
        self.presupuesto
    }

    pub fn visitar_parque(&mut self, parque: Arc<Parque>) {
        println!("[Persona {}] Esperando para entrar al parque", self.id);
        parque.ingresar_persona();
        println!("[Persona {}] Entre al parque con {} pesos", self.id, self.presupuesto);
        while self.presupuesto > 0 {
            let juego = match parque.elegir_juego_random(self.presupuesto) {
                Ok(juego) => juego,
                Err(_) => break
            };

            println!("[Persona {}] Entrando a la fila del juego {}.", self.id, juego.id);
            juego.entrar(self);
            println!("[Persona {}] Jugué al juego {} y salí.", self.id, juego.id);
        }
        println!("[Persona {}] No me alcanza para ningun juego (me quedaron $ {})", self.id, self.presupuesto);
        parque.salir_persona();
        println!("[Persona {}] Me fui del parque", self.id);
    }
}