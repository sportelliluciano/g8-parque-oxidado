use std::{
    sync::atomic::AtomicBool,
    sync::atomic::Ordering,
    sync::Condvar,
    sync::Mutex,
    sync::Barrier,
    sync::Arc,
    time::Duration
};
use std::thread;

use crate::{logger::TaggedLogger, parque::Parque, persona::Persona};

pub struct Juego {
    pub id: usize,
    parque: Arc<Parque>,
    pub precio: u32,
    tiempo: u32,
    capacidad: u32,

    cant_espacio_libre: Mutex<u32>,
    hay_espacio_mutex: Mutex<()>,
    cv_cero_espacio_libre: Condvar,

    salida_barrier: Arc<Barrier>,
    salida_mutex: Mutex<()>,

    cerrar: AtomicBool,
    log: TaggedLogger,
}

impl Juego {
    pub fn new(log: TaggedLogger, 
               id: usize, 
               parque: Arc<Parque>, 
               precio: u32,
               capacidad: u32,
               duracion_ms: u32) -> Self {
        Self {
            id,
            parque,
            precio,
            tiempo: duracion_ms,
            capacidad,

            cant_espacio_libre: Mutex::new(capacidad),
            hay_espacio_mutex: Mutex::new(()),
            cv_cero_espacio_libre: Condvar::new(),

            salida_barrier: Arc::new(Barrier::new(capacidad as usize + 1)), // +1 para esperar el del juego
            salida_mutex: Mutex::new(()),

            cerrar: AtomicBool::new(false),
            log,
        }
    }

    pub fn thread_main(&self) {
        while !self.cerrar.load(Ordering::SeqCst) {

            // *** Esperar a que entre la gente ***
            self.log.write("Esperando a la gente");
            let (mut espacio_libre, timeout) =
                self.cv_cero_espacio_libre.wait_timeout(
                    self.cant_espacio_libre.lock().expect("poisoned"),
                    Duration::from_secs(5)
                ).expect("poisoned");

            if timeout.timed_out() {
                // Timed out -> ver si hay gente y correr el juego.
                self.log.write("(TIMEOUT) esperando a la gente");
                if *espacio_libre == self.capacidad {
                    self.log.write("(TIMEOUT) no hay gente, loopeando");
                    continue
                }
            } else if *espacio_libre != 0 { // desbloqueo espurio
                self.log.write("desbloqueo espurio esperando gente");
                continue
            }

            let gente_adentro = self.capacidad - *espacio_libre;
            self.log.write(
            &format!(
                    "Arrancando el juego con {}/{} personas", 
                    gente_adentro, self.capacidad
                )
            );

            // *** Arrancar el juego ***
            thread::sleep(Duration::from_millis(self.tiempo as u64));

            self.log.write("Terminado, esperando que salga la gente");
            let mut handles = vec![];
            for _persona in 0..(*espacio_libre + 1) {
                let salida_barrier_c = Arc::clone(&self.salida_barrier);
                let handle = thread::spawn(move || {
                    salida_barrier_c.wait();
                });
                handles.push(handle);
            }
            for handle in handles {
                handle.join().unwrap();
            }

            self.log.write("Salió toda la gente, re-arrancando");
            *espacio_libre = self.capacidad;
        }

        self.log.write("Cerrado");
    }

    pub fn entrar(&self, persona: &mut Persona) {
        let hay_espacio = self.hay_espacio_mutex.lock().expect("poisoned");
        let mut espacio_libre = self.cant_espacio_libre.lock().expect("poison");
        *espacio_libre -= 1;
        if *espacio_libre == 0 {
            self.cv_cero_espacio_libre.notify_one();
        } else {
            // Si todavia queda espacio: desbloquear el mutex hay_espacio.
            // Sino se desbloquea después de que juegue y salga la ultima persona que entró.
            drop(hay_espacio);
        }
        drop(espacio_libre);
        self.jugar(persona);
    }

    fn cobrar_entrada(&self, persona: &mut Persona) {
        persona.pagar_juego(self);
        self.parque.guardar_dinero(self.precio);
    }

    fn jugar(&self, persona: &mut Persona) {
        self.cobrar_entrada(persona);
        persona.juego_iniciando(self.id);
        self.salida_barrier.wait();
        self.salir();
    }

    fn salir(&self) {
        // lockear el mutex de la salida para salir de a uno
        let _mutex = self.salida_mutex.lock().expect("poison");
    }

    /// Cantidad de desperfectos que ocurrieron (el parque lo usa)
    pub fn obtener_desperfectos(&self) -> u32 {
        0
    }

    /// EL PARQUE LE INDICA AL JUEGO QUE DEBE CERRARSE CUANDO SE FUE TODA LA GENTE
    pub fn cerrar(&self) {
        self.cerrar.store(true, Ordering::SeqCst);
    }
}