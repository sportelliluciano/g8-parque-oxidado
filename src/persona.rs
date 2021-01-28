use std::{
    sync::Arc,
    thread::JoinHandle
};

use crate::{logger::{Logger, TaggedLogger}, parque::Parque};
use crate::juego::Juego;

pub fn iniciar_hilos_personas(logger: Arc<Logger>, parque: Arc<Parque>, presupuestos: &[u32]) -> Vec<JoinHandle<()>> {
    let mut handles = vec![];
    for (id, presupuesto_persona) in presupuestos.iter().enumerate() {
        let parque_child = parque.clone();
        let presupuesto_persona = *presupuesto_persona;
        let mut persona = Persona::new(
            TaggedLogger::new(&format!("PERSONA {}", id), logger.clone()),
            id, 
            presupuesto_persona
        );
        handles.push(std::thread::spawn(move || {
            persona.visitar_parque(parque_child);
        }));
    }
    handles
}

pub struct Persona {
    pub id: usize,
    presupuesto: u32,
    log: TaggedLogger
}

impl Persona {
    pub fn new(log: TaggedLogger, id: usize, presupuesto: u32) -> Self {
        Self {
            id,
            presupuesto,
            log,
        }
    }

    pub fn pagar_juego(&mut self, juego: &Juego) -> u32 {
        let presupuesto_restante = self.presupuesto - juego.precio;
        self.log.write(&format!("Pagando juego {}. Tenía $ {} y pagué $ {}, me quedan $ {}", juego.id, self.presupuesto, juego.precio, presupuesto_restante));
        self.presupuesto = presupuesto_restante;
        self.presupuesto
    }

    pub fn visitar_parque(&mut self, parque: Arc<Parque>) {
        self.log.write("Esperando para entrar al parque");
        parque.ingresar_persona();
        self.log.write(&format!("Entre al parque con $ {}", self.presupuesto));
        while self.presupuesto > 0 {
            let juego = match parque.elegir_juego_random(self.presupuesto) {
                Ok(juego) => juego,
                Err(_) => break
            };

            self.jugar(juego);
        }
        self.log.write(&format!("No me alcanza para ningun juego (me quedaron $ {})", self.presupuesto));
        parque.salir_persona();
        self.log.write("Me fui del parque");
    }

    fn jugar(&mut self, juego: Arc<Juego>) {
        self.log.write(&format!("Entrando a la fila del juego {}.", juego.id));
        juego.admitir(self);
        self.log.write(&format!("Jugué al juego {} y salí.", juego.id));
    }

    /// TODO: Esto es puramente para logging. Eliminar?
    pub fn juego_iniciando(&self, id_juego: usize) {
        self.log.write(&format!("Logré entrar al juego {}, empenzado a jugar", id_juego));
    }
}