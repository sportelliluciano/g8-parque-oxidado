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

use std_semaphore::Semaphore;

use crate::semaforo::Semaforo;

pub struct Juego {
    pub precio: u32,
    tiempo: u32,
    capacidad: usize,
    cerrar: AtomicBool,
    pub id: usize,

    sem_entrada: Semaforo,
    sem_juego: Semaphore,

    salida: Mutex<bool>,
    juego_terminado: Condvar,

    salida_2: Arc<Barrier>,
}

impl Juego {
    pub fn new(id: usize, precio: u32) -> Self {
        Self {
            precio,
            tiempo: 0,
            capacidad: 2,
            cerrar: AtomicBool::new(false),
            id,
            
            sem_entrada: Semaforo::new(2 as isize),
            sem_juego: Semaphore::new(0),

            juego_terminado: Condvar::new(),
            salida: Mutex::new(true),

            salida_2: Arc::new(Barrier::new(2 + 1)), // TODO modificar junto a la capacidad, +1 para esperar el del juego
        }
    }

    pub fn thread_main(&self) {
        while self.cerrar.load(Ordering::SeqCst) != true {
            println!("[JUEGO {}] Esperando a la gente", self.id);
            // *** Esperar a que entre la gente ***
            //  >> gente_adentro.unlock(); atomico con el wait siguiente, las personas pueden empezar a entrar
            let (mut espacio_libre, timeout) =
                self.sem_entrada.wait_zero_timeout(Duration::from_secs(5));
            //  >> gente_adentro.lock(); // los que hacen entrar se quedan lockeados atomicamente con el wait, no pueden entrar mas
            if timeout.timed_out() {
                // Timed out -> ver si hay gente y correr el juego.
                println!("[JUEGO {}] (TIMEOUT) esperando a la gente", self.id);
                if *espacio_libre == self.capacidad as isize {
                    println!("[JUEGO {}] (TIMEOUT) no hay gente, loopeando", self.id);
                    continue
                }
            } else if *espacio_libre != 0 { // desbloqueo espurio
                println!("[JUEGO {}] desbloqueo espurio esperando gente", self.id);
                continue
            }
            let gente_adentro = self.capacidad as isize - *espacio_libre;

            println!("[JUEGO {}] Arrancando el juego con {}/{} personas", self.id, gente_adentro, self.capacidad);
            // *** Arrancar el juego ***
            thread::sleep(Duration::from_millis(self.tiempo as u64));
            println!("[JUEGO {}] Terminado, esperando que salga la gente", self.id);
            // self.juego_terminado.notify_all();
            // for _persona in 0..gente_adentro {
                // self.sem_juego.release();
            // }
            let mut handles = vec![];
            for _persona in 0..(*espacio_libre + 1) {
                let salida_2_c = Arc::clone(&self.salida_2);
                let handle = thread::spawn(move || {
                    salida_2_c.wait();
                });
                handles.push(handle);
            }
            for handle in handles {
                handle.join().unwrap();
            }
            println!("[JUEGO {}] Salió toda la gente, re-arrancando", self.id);
            *espacio_libre = self.capacidad as isize;
        }

        println!("[JUEGO {}] Cerrado", self.id);
    }

    pub fn entrar(&self, id: usize) {
        self.sem_entrada.acquire(id);
    }

    pub fn jugar(&self) {
        // TODO que este metodo no exista mas, que sea implicito con el entrar
        // self.juego_terminado.wait(self.salida.lock().expect("poisoned")).expect("TODO no se");
        // self.sem_juego.acquire();
        self.salida_2.wait();
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