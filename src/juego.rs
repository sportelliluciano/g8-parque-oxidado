use std::{
    sync::atomic::AtomicBool,
    sync::atomic::Ordering,
    thread::sleep,
    time::Duration
};

use std_semaphore::Semaphore;

use crate::semaforo::Semaforo;

pub struct Juego {
    precio: u32,
    tiempo: u32,
    capacidad: usize,
    cerrar: AtomicBool,
    id: usize,

    sem_entrada: Semaforo,
    sem_juego: Semaphore
}

impl Juego {
    pub fn new(id: usize, precio: u32) -> Self {
        Self {
            precio,
            tiempo: 0,
            capacidad: 10000000,
            cerrar: AtomicBool::new(false),
            id,
            
            sem_entrada: Semaforo::new(10000000 as isize),
            sem_juego: Semaphore::new(0),
        }
    }

    pub fn thread_main(&self) {
        while self.cerrar.load(Ordering::SeqCst) != true {
            println!("[JUEGO {}] Esperando a la gente", self.id);
            // *** Esperar a que entre la gente ***
            //  >> gente_adentro.unlock(); atomico con el wait siguiente, las personas pueden empezar a entrar
            let gente_adentro = {
                let (espacio_libre, timeout) = 
                    self.sem_entrada.wait_zero_timeout(Duration::from_secs(5));
                //  >> gente_adentro.lock(); // los que hacen entrar se quedan lockeados atomicamente con el wait, no pueden entrar mas
                if timeout.timed_out() {
                    // Timed out -> ver si hay gente y correr el juego.
                    println!("[JUEGO {}] (TIMEOUT) esperando a la gente", self.id);
                    if *espacio_libre == self.capacidad as isize { 
                        println!("[JUEGO {}] (TIMEOUT) no hay gente, loopeando", self.id);
                        continue
                    }
                } else {
                    if *espacio_libre != 0 { // desbloqueo espurio
                        println!("[JUEGO {}] desbloqueo espurio esperando gente", self.id);
                        continue
                    }
                }

                self.capacidad as isize - *espacio_libre
            }; // espacio_libre.unlock

            println!("[JUEGO {}] Arrancando el juego con {}/{} personas", self.id, gente_adentro, self.capacidad);
            // *** Arrancar el juego ***
            sleep(Duration::from_millis(self.tiempo as u64));
            println!("[JUEGO {}] Terminado, esperando que salga la gente", self.id);
            for _persona in 0..gente_adentro {
                self.sem_juego.release();
            }
            println!("[JUEGO {}] Salió toda la gente, re-arrancando", self.id);
            for _persona in 0..gente_adentro {
                self.sem_entrada.release();
            }
        }

        println!("[JUEGO {}] Cerrado", self.id);
    }

    pub fn entrar(&self) {
        self.sem_entrada.acquire();
    }

    pub fn jugar(&self) {
        self.sem_juego.acquire();
    }

    /// Cantidad de desperfectos que ocurrieron (el parque lo usa)
    pub fn obtener_desperfectos(&self) -> u32 {
        0
    }

    /// EL PARQUE LE INDICA AL JUEGO QUE DEBE CERRARSE CUANDO SE FUE TODA LA GENTE
    pub fn cerrar(&self) {
        self.cerrar.store(true, Ordering::SeqCst);
    }

    /// ID DE JUEGO PARA LOS LOGS
    pub fn id(&self) -> usize {
        self.id
    }

    /// PRECIO DEL JUEGO
    pub fn precio(&self) -> u32 {
        self.precio
    }
}