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

use crate::{juego::Juego, logger::{TaggedLogger}};

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
        // TODO la consigna dice Los visitantes van llegando de a uno, es suficiente el semaforo?
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


#[cfg(test)]
mod tests {
    use crate::logger::Logger;

    use super::*;

    #[test]
    fn caja_inicial_es_cero() {
        let parque = crear_parque(2);
        assert_eq!(parque.obtener_caja(), 0);
    }

    #[test]
    fn desperfectos_inicial_es_cero() {
        let parque = crear_parque(2);
        assert_eq!(parque.obtener_desperfectos(), 0);
    }

    #[test]
    fn juegos_inicial_es_vacio() {
        let parque = crear_parque(2);
        assert!(parque.juegos.lock().expect("poisoned").is_empty());
    }

    #[test]
    fn registrar_juegos_guarda_los_juegos() {
        let parque = Arc::new(crear_parque(2));
        let vec: Vec<u32> = (0..10).collect();
        let juegos = vec.iter()
            .map(|id| crear_juego(
                *id as usize,
                Arc::clone(&parque),
                20,
                2,
                25
            ))
            .collect::<Vec<Juego>>();
        parque.registrar_juegos(juegos);
        assert_eq!(parque.juegos.lock().expect("poisoned").len(), 10);
    }

    #[test]
    fn obtener_juegos_posibles_cuando_no_hay_juegos() {
        let parque = Arc::new(crear_parque(2));
        assert!(parque.obtener_juegos_posibles(2).is_empty());
    }

    #[test]
    fn obtener_juegos_posibles_cuando_todos_son_posibles() {
        let parque = Arc::new(crear_parque(2));
        let vec: Vec<u32> = (0..10).collect();
        let juegos = vec.iter()
            .map(|id| crear_juego(
                *id as usize,
                Arc::clone(&parque),
                20,
                2,
                25
            ))
            .collect::<Vec<Juego>>();
        parque.registrar_juegos(juegos);
        assert_eq!(parque.obtener_juegos_posibles(30).len(), 10);
    }

    #[test]
    fn obtener_juegos_posibles_cuando_ninguno_es_posible() {
        let parque = Arc::new(crear_parque(2));
        let vec: Vec<u32> = (0..10).collect();
        let juegos = vec.iter()
            .map(|id| crear_juego(
                *id as usize,
                Arc::clone(&parque),
                20,
                2,
                25
            ))
            .collect::<Vec<Juego>>();
        parque.registrar_juegos(juegos);
        assert_eq!(parque.obtener_juegos_posibles(10).len(), 0);
    }

    #[test]
    fn obtener_juegos_posibles_cuando_algunos_son_posibles() {
        let parque = Arc::new(crear_parque(2));
        let vec1: Vec<u32> = (0..5).collect();
        let juegos1 = vec1.iter()
            .map(|id| crear_juego(
                *id as usize,
                Arc::clone(&parque),
                20,
                2,
                25
            ))
            .collect::<Vec<Juego>>();
        parque.registrar_juegos(juegos1);
        let vec2: Vec<u32> = (0..5).collect();
        let juegos2 = vec2.iter()
            .map(|id| crear_juego(
                *id as usize,
                Arc::clone(&parque),
                30,
                2,
                25
            ))
            .collect::<Vec<Juego>>();
        parque.registrar_juegos(juegos2);
        assert_eq!(parque.obtener_juegos_posibles(25).len(), 5);
    }

    #[test]
    fn elegir_juego_random_devuelve_uno_de_los_posibles() {
        let parque = Arc::new(crear_parque(2));
        let vec: Vec<u32> = (0..10).collect();
        let juegos = vec.iter()
            .map(|id| crear_juego(
                *id as usize,
                Arc::clone(&parque),
                20,
                2,
                25
            ))
            .collect::<Vec<Juego>>();
        parque.registrar_juegos(juegos);
        let juego_random = parque.elegir_juego_random(30).unwrap();
        assert!(
            parque.juegos.lock().expect("poisoned").iter().any(
                |juego| juego.id == juego_random.id
            )
        );
    }

    #[test]
    fn elegir_juego_random_da_error_si_no_hay_ninguno_posible() {
        let parque = Arc::new(crear_parque(2));
        let vec: Vec<u32> = (0..10).collect();
        let juegos = vec.iter()
            .map(|id| crear_juego(
                *id as usize,
                Arc::clone(&parque),
                20,
                2,
                25
            ))
            .collect::<Vec<Juego>>();
        parque.registrar_juegos(juegos);
        assert!(!parque.elegir_juego_random(10).is_ok());
    }

    fn crear_parque(capacidad: usize) -> Parque {
        crear_parque_con_semilla(capacidad, 2)
    }

    fn crear_parque_con_semilla(capacidad: usize, semilla: u64) -> Parque {
        Parque::new(
            crear_logger(), capacidad, semilla
        )
    }

    fn crear_logger() -> TaggedLogger {
        TaggedLogger::new("ADMIN", Arc::new(Logger::new_to_stdout()))
    }

    fn crear_juego(id: usize, parque: Arc<Parque>, precio: u32, capacidad: u32, duracion_ms: u32) -> Juego {
        Juego::new(
            crear_logger(),
            id,
            Arc::clone(&parque),
            precio,
            capacidad,
            duracion_ms,
            3 // TODO que no todos tengan la misma
        )
    }
}