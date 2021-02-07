use std::{sync::Arc, sync::Barrier, sync::Condvar, sync::Mutex, sync::RwLock, sync::atomic::AtomicBool, sync::atomic::{AtomicU32, Ordering}, time::Duration};
use std::thread;

use rand::{Rng, SeedableRng, prelude::StdRng};
use std_semaphore::Semaphore;

use crate::{logger::TaggedLogger, parque::Parque, persona::Persona};

const PROBABILIDAD_DE_DESPERFECTOS: f64 = 0.05; // 5%
const TIEMPO_MAXIMO_ARREGLO_DESPERFECTO: u64 = 25;

pub struct Juego {
    pub id: usize,
    parque: Arc<Parque>,
    pub precio: u32,
    tiempo: u32,
    capacidad: u32,

    cant_espacio_libre: Mutex<u32>,
    hay_espacio_mutex: Mutex<()>,
    cv_cero_espacio_libre: Condvar,

    sem_juego_en_curso: Semaphore,

    salida_barrier: RwLock<Barrier>,
    salida_mutex: Mutex<()>,

    cerrado: AtomicBool,
    log: TaggedLogger,

    rng: Mutex<StdRng>,
    cantidad_desperfectos: AtomicU32,
}

impl Juego {
    pub fn new(log: TaggedLogger,
               id: usize,
               parque: Arc<Parque>,
               precio: u32,
               capacidad: u32,
               duracion_ms: u32,
               semilla: u64) -> Self {
        Self {
            id,
            parque,
            precio,
            tiempo: duracion_ms,
            capacidad,

            cant_espacio_libre: Mutex::new(capacidad),
            hay_espacio_mutex: Mutex::new(()),
            cv_cero_espacio_libre: Condvar::new(),

            sem_juego_en_curso: Semaphore::new(0),

            salida_barrier: RwLock::new(Barrier::new(capacidad as usize + 1)), // +1 para esperar el del juego
            salida_mutex: Mutex::new(()),

            cerrado: AtomicBool::new(false),
            log,

            rng: Mutex::new(StdRng::seed_from_u64(semilla)),
            cantidad_desperfectos: AtomicU32::new(0),
        }
    }

    pub fn iniciar_funcionamiento(&self) {
        // TODO refactorizar
        let mut rng = self.rng.lock().expect("posioned rng");
        while !self.cerrado.load(Ordering::SeqCst) {
            let hubo_desperfecto: f64 = rng.gen();
            if hubo_desperfecto < PROBABILIDAD_DE_DESPERFECTOS {
                // desperfecto generado
                self.log.write("Desperfecto generado");
                self.cantidad_desperfectos.fetch_add(1, Ordering::SeqCst);
                // simular tiempo de reparacion del desperfecto
                thread::sleep(
                    Duration::from_millis(
                        rng.gen_range(0..TIEMPO_MAXIMO_ARREGLO_DESPERFECTO) as u64
                    )
                );
                self.log.write("Desperfecto arreglado, iniciando una nueva vuelta");
            } else {
                // funcionamiento correcto, dar una vuelta del juego

                // *** Esperar a que entre la gente ***
                self.log.write("Esperando personas para iniciar la vuelta");
                let (mut espacio_libre, timeout) =
                    self.cv_cero_espacio_libre.wait_timeout(
                        self.cant_espacio_libre.lock().expect("poisoned"),
                        Duration::from_secs(5)
                    ).expect("poisoned");

                if timeout.timed_out() {
                    // Timed out -> ver si hay gente y correr el juego.
                    if *espacio_libre == self.capacidad {
                        self.log.write("Tiempo de espera de personas agotado sin ninguna persona lista para jugar, reiniciando espera de personas");
                        continue
                    } else {
                        self.log.write("Tiempo de espera de personas agotado con personas listas para jugar, iniciando vuelta");
                    }
                } else if *espacio_libre != 0 { // desbloqueo espurio de la condvar
                    continue
                }

                let gente_adentro = self.capacidad - *espacio_libre;
                self.log.write(
                &format!(
                        "Arrancando la vuelta del juego con {}/{} personas",
                        gente_adentro, self.capacidad
                    )
                );

                // *** Arrancar el juego ***
                thread::sleep(Duration::from_millis(self.tiempo as u64));

                self.terminar_vuelta(gente_adentro);

                // Marcar todo el espacio como libre para que puedan entrar nuevas personas al juego en la siguiente vuelta
                *espacio_libre = self.capacidad;
            }
        }

        self.log.write("Cerrado");
    }

    fn terminar_vuelta(&self, gente_adentro: u32) {
        // setear la cantidad de personas a esperar que usen la salida previo a avisar que dejen sus lugares
        {
            let mut salida_barrier = self.salida_barrier.write().expect("poison");
            *salida_barrier = Barrier::new(gente_adentro as usize + 1); // +1 para esperar el del juego
        }
        self.log.write("Vuelta terminada, esperando que las personas dejen sus lugares");
        // avisar que el juego terminó
        for _persona in 0..gente_adentro {
            self.sem_juego_en_curso.release();
        }
        self.log.write("Esperando que las personas salgan del juego");
        let salida_barrier = self.salida_barrier.read().expect("poison");
        salida_barrier.wait();
        self.log.write("Todas las personas salieron del juego, iniciando una nueva vuelta");
    }

    pub fn agregar_a_la_fila(&self, persona: &mut Persona) {
        // TODO cobrar la entrada acá?
        // la consigna dice Cuando la persona llega a la puerta del juego elegido, debe abonar la entrada
        // cuestionable si la "fila" esta es la entrada o no
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
        self.admitir_para_jugar(persona);
    }

    fn cobrar_entrada(&self, persona: &mut Persona) {
        persona.pagar_juego(self);
        self.parque.guardar_dinero(self.precio);
    }

    fn admitir_para_jugar(&self, persona: &mut Persona) {
        self.cobrar_entrada(persona);
        self.log.write(&format!("Persona {} logró entrar al juego", persona.id));
        self.sem_juego_en_curso.acquire();
        self.permitir_salir();
    }

    fn permitir_salir(&self) {
        let barrier = self.salida_barrier.read().expect("poisoned");
        barrier.wait();
        // lockear el mutex de la salida para salir de a uno
        let _mutex = self.salida_mutex.lock().expect("poison");
    }

    /// Cantidad de desperfectos que ocurrieron (el parque lo usa)
    pub fn obtener_desperfectos(&self) -> u32 {
        self.cantidad_desperfectos.load(Ordering::SeqCst)
    }

    /// EL PARQUE LE INDICA AL JUEGO QUE DEBE CERRARSE CUANDO SE FUE TODA LA GENTE
    pub fn cerrar(&self) {
        self.cerrado.store(true, Ordering::SeqCst);
    }
}