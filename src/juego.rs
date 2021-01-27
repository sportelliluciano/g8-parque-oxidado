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

use crate::{parque::Parque, persona::Persona};

pub struct Juego {
    pub id: usize,
    parque: Arc<Parque>,
    pub precio: u32,
    tiempo: u32,
    capacidad: usize,

    cant_espacio_libre: Mutex<usize>,
    cv_cero_espacio_libre: Condvar,

    salida_barrier: Arc<Barrier>,
    salida_mutex: Mutex<bool>,

    cerrar: AtomicBool,
}

impl Juego {
    pub fn new(id: usize, parque: Arc<Parque>, precio: u32) -> Self {
        let capacidad = 2; // TODO que sea parametro
        Self {
            id,
            parque,
            precio,
            tiempo: 25, // TODO que sea parametro
            capacidad,

            cant_espacio_libre: Mutex::new(capacidad),
            cv_cero_espacio_libre: Condvar::new(),

            salida_barrier: Arc::new(Barrier::new(capacidad + 1)), // +1 para esperar el del juego
            salida_mutex: Mutex::new(true),

            cerrar: AtomicBool::new(false),
        }
    }

    pub fn thread_main(&self) {
        while self.cerrar.load(Ordering::SeqCst) != true {

            // *** Esperar a que entre la gente ***
            println!("[JUEGO {}] Esperando a la gente", self.id);
            //  >> espacio_libre.unlock(); atomico con el wait siguiente, las personas pueden empezar a entrar
            let (mut espacio_libre, timeout) =
                self.cv_cero_espacio_libre.wait_timeout(
                    self.cant_espacio_libre.lock().expect("poisoned"),
                    Duration::from_secs(5)
                ).expect("poisoned");
            //  >> espacio_libre.lock(); los que hacen entrar se quedan lockeados atomicamente con el wait, no pueden entrar mas

            if timeout.timed_out() {
                // Timed out -> ver si hay gente y correr el juego.
                println!("[JUEGO {}] (TIMEOUT) esperando a la gente", self.id);
                if *espacio_libre == self.capacidad {
                    println!("[JUEGO {}] (TIMEOUT) no hay gente, loopeando", self.id);
                    continue
                }
            } else if *espacio_libre != 0 { // desbloqueo espurio
                println!("[JUEGO {}] desbloqueo espurio esperando gente", self.id);
                continue
            }

            let gente_adentro = self.capacidad - *espacio_libre;
            println!("[JUEGO {}] Arrancando el juego con {}/{} personas", self.id, gente_adentro, self.capacidad);

            // *** Arrancar el juego ***
            thread::sleep(Duration::from_millis(self.tiempo as u64));

            println!("[JUEGO {}] Terminado, esperando que salga la gente", self.id);
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

            println!("[JUEGO {}] Salió toda la gente, re-arrancando", self.id);
            *espacio_libre = self.capacidad;
        }

        println!("[JUEGO {}] Cerrado", self.id);
    }

    pub fn entrar(&self, persona: &mut Persona) {
        let mut espacio_libre = self.cant_espacio_libre.lock().expect("poison");
        if *espacio_libre == 0 {
            // Si pude tomar el lock aunque la condicion ya se cumplió
            // Esto es posible ya que hay un tiempo entre el notify_one y que se despierte el thread juego y por lo tanto se lockee la cantidad
            // En este caso, desalojar este thread para permitir que el otro se despierte e intentar nuevamente entrar al juego
            drop(espacio_libre);
            self.cv_cero_espacio_libre.notify_one();
            std::thread::yield_now();
            self.entrar(persona);
        } else {
            *espacio_libre -= 1;
            if *espacio_libre == 0 {
                println!("[PERSONA {}] NOTIFICO CANTIDAD 0", persona.id);
                self.cv_cero_espacio_libre.notify_one();
            }
            drop(espacio_libre);
            self.jugar(persona);
        }
    }

    fn cobrar_entrada(&self, persona: &mut Persona) {
        persona.pagar_juego(self);
        self.parque.guardar_dinero(self.precio);
    }

    fn jugar(&self, persona: &mut Persona) {
        self.cobrar_entrada(persona);
        println!("[Persona {}] Logré entrar al juego {}, empenzado a jugar", persona.id, self.id);
        self.salida_barrier.wait();
        self.salir();
    }

    fn salir(&self) {
        // lockear el mutex de la salida para salir de a uno
        self.salida_mutex.lock().expect("poison");
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