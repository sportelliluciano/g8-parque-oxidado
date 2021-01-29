use std::{
    sync::{
        Arc,
        Mutex,
        atomic::{AtomicU32, AtomicUsize, Ordering}
    },
    thread::JoinHandle
};
use rand::{Rng, SeedableRng, prelude::StdRng};
use std_semaphore::Semaphore;

use crate::{juego::Juego, logger::TaggedLogger};

pub struct Parque {
    juegos: Mutex<Vec<Arc<Juego>>>,
    juegos_threads: Mutex<Vec<JoinHandle<()>>>,
    caja: Arc<AtomicU32>,
    capacidad: Semaphore,
    cantidad_visitantes: AtomicUsize,
    rng: Mutex<StdRng>,
    log: TaggedLogger
}

impl Parque {
    pub fn new(log: TaggedLogger, capacidad: usize, semilla: u64) -> Self {
        Self {
            caja: Arc::new(AtomicU32::new(0)), 
            capacidad: Semaphore::new(capacidad as isize),
            cantidad_visitantes: AtomicUsize::new(0),
            juegos: Mutex::new(vec![]),
            juegos_threads: Mutex::new(vec![]),
            rng: Mutex::new(StdRng::seed_from_u64(semilla)),
            log
        }
    }

    pub fn registrar_juegos(&self, juegos: Vec<Juego>) {
        let mut juegos_vec = self.juegos.lock().expect("poisoned");
        let mut juegos_threads_vec = self.juegos_threads.lock().expect("poisoned");
        for juego in juegos {
            let juego_ref = Arc::new(juego);
            juegos_vec.push(juego_ref.clone());
            juegos_threads_vec.push(std::thread::spawn(move || {
                juego_ref.iniciar_funcionamiento();
            }));
        }
    }

    fn obtener_juegos_posibles(&self, presupuesto_maximo: u32) -> Vec<Arc<Juego>> {
        let mut resultado = vec![];
        for juego in self.juegos.lock().expect("poisoned").iter() {
            if presupuesto_maximo >= juego.precio {
                resultado.push(juego.clone());
            }
        }
    
        resultado
    }

    pub fn elegir_juego_random(&self, presupuesto_maximo: u32) -> Result<Arc<Juego>, &'static str> {
        let juegos_posibles = self.obtener_juegos_posibles(presupuesto_maximo);
        if juegos_posibles.is_empty() {
            Err("No alcanza el dinero")
        } else {
            let mut rng = self.rng.lock().expect("posioned rng");
            Ok(juegos_posibles[rng.gen_range(0..juegos_posibles.len())].clone())
        }
    }

    pub fn ingresar_persona(&self) {
        self.capacidad.acquire();
    }

    pub fn salir_persona(&self) {
        self.cantidad_visitantes.fetch_add(1, Ordering::SeqCst);
        self.capacidad.release();
    }

    pub fn obtener_cantidad_gente_que_salio_del_parque(&self) -> usize {
        // En lugar de ver cuanta gente hay adentro, contar
        // cuanta gente salió del parque y revisar que todos los que tenían
        // que entrar hayan salido.
        self.cantidad_visitantes.load(Ordering::SeqCst)
    }

    pub fn cerrar(&self) {
        self.log.write("Cerrando juegos");
        for juego in self.juegos.lock().expect("poisoned").iter() {
            juego.cerrar();
        }

        self.log.write("Esperando a que los juegos terminen");
        for juego_thread in self.juegos_threads.lock().expect("poisoned").drain(..) {
            juego_thread.join().expect("cannot join thread");
        }
        self.log.write("Parque cerrado");
    }

    pub fn guardar_dinero(&self, monto: u32) {
        self.caja.fetch_add(monto, Ordering::SeqCst);
    }

    pub fn obtener_caja(&self) -> u32 {
        self.caja.load(Ordering::SeqCst)
    }

    pub fn obtener_desperfectos(&self) -> u32 {
        let mut cantidad = 0;
        for juego in &(*self.juegos.lock().expect("poisoned")) {
            cantidad += juego.obtener_desperfectos();
        }

        cantidad
    }
}