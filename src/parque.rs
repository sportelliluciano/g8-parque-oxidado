use std::{
    sync::{
        Arc,
        Mutex,
        atomic::{AtomicU32, AtomicUsize, Ordering}
    },
    thread::JoinHandle
};
use std_semaphore::Semaphore;

use crate::juego::Juego;

pub struct Parque {
    juegos: Vec<Arc<Juego>>,
    // Este mutex no debería ser necesario, pero si no lo pongo el compilador shora
    juegos_threads: Mutex<Vec<JoinHandle<()>>>,
    caja: Arc<AtomicU32>,
    capacidad: Semaphore,
    cantidad_visitantes: AtomicUsize,
    gente_adentro: AtomicU32,
}

impl Parque {
    pub fn new(capacidad: usize, juegos: Vec<Juego>) -> Self {
        let mut arc_juegos = vec![];
        let mut juegos_threads = vec![];

        for juego in juegos {
            let juego_ref = Arc::new(juego);
            arc_juegos.push(juego_ref.clone());
            juegos_threads.push(std::thread::spawn(move || {
                juego_ref.thread_main();
            }))
        }

        Self {
            juegos: arc_juegos, 
            juegos_threads: Mutex::new(juegos_threads),
            caja: Arc::new(AtomicU32::new(0)), 
            capacidad: Semaphore::new(capacidad as isize),
            cantidad_visitantes: AtomicUsize::new(0),
            gente_adentro: AtomicU32::new(0)
        }
    }

    pub fn pagar(&self, monto: u32) {
        self.caja.fetch_add(monto, Ordering::SeqCst);
    }

    fn obtener_juegos_posibles(&self, presupuesto_maximo: u32) -> Vec<Arc<Juego>> {
        let mut resultado = vec![];
        for juego in self.juegos.iter() {
            if presupuesto_maximo >= juego.precio() {
                resultado.push(juego.clone());
            }
        }
    
        resultado
    }

    pub fn elegir_juego_random(&self, presupuesto_maximo: u32) -> Result<Arc<Juego>, &'static str> {
        let juegos_posibles = self.obtener_juegos_posibles(presupuesto_maximo);
        if juegos_posibles.len() == 0 {
            Err("Sos pobre pa, tomatelas")
        } else {   
            use rand::Rng;
            let mut rng = rand::thread_rng();
            Ok(juegos_posibles[rng.gen_range(0..juegos_posibles.len())].clone())
        }
    }

    /// TODO: Chequear race condition / inconsistencia????
    pub fn ingresar_persona(&self) {
        self.capacidad.acquire();
        self.gente_adentro.fetch_add(1, Ordering::SeqCst);
    }

    /// TODO: Chequear race condition / inconsistencia????
    pub fn salir_persona(&self) {
        self.cantidad_visitantes.fetch_add(1, Ordering::SeqCst);
        self.capacidad.release();
        self.gente_adentro.fetch_sub(1, Ordering::SeqCst);
    }

    pub fn obtener_cantidad_gente_que_salio_del_parque(&self) -> usize {
        // Originalmente esto era while hay_gente_adentro() { ... }
        // Pero eso llevaba a una race condition donde el parque se cerraba
        // antes de que entre la primer persona...
        // Solución fea: en lugar de ver cuanta gente hay adentro, contar
        // cuanta gente salió del parque (y revisar que todos los que tenían
        // que entrar hayan salido).
        self.cantidad_visitantes.load(Ordering::SeqCst)
    }

    pub fn obtener_genete_adentro(&self) -> u32 {
        self.gente_adentro.load(Ordering::SeqCst)
    }

    pub fn cerrar(&self) {
        println!("[PARQUE] Cerrando juegos");
        for juego in &self.juegos {
            juego.cerrar();
        }

        println!("[PARQUE] Esperando a que los juegos terminen");
        let mut threads = 
            self.juegos_threads.lock().expect("mutex poisoned");
        while threads.len() > 0 {
            match threads.pop() {
                Some(handle) => handle.join().expect("cannot join thread"),
                None => break
            }
        }
        println!("[PARQUE] Parque cerrado");
    }

    pub fn obtener_caja(&self) -> u32 {
        self.caja.load(Ordering::SeqCst)
    }

    pub fn obtener_desperfectos(&self) -> u32 {
        let mut cantidad = 0;
        for juego in &self.juegos {
            cantidad += juego.obtener_desperfectos();
        }

        cantidad
    }
}