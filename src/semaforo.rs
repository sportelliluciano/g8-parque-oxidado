use std::{sync::{Condvar, Mutex, MutexGuard, WaitTimeoutResult}, thread};
use std::time::Duration;

use std_semaphore::Semaphore;

pub struct Semaforo {
    sem: Semaphore,
    cantidad: Mutex<isize>,
    cv_cero: Condvar,
}

impl Semaforo {
    pub fn new(cantidad: isize) -> Self {
        Self {
            sem: Semaphore::new(cantidad),
            cantidad: Mutex::new(cantidad),
            cv_cero: Condvar::new()
        }
    }

    pub fn acquire(&self, persona_id: usize) {
        // self.sem.acquire();
        let mut cantidad = self.cantidad.lock().expect("poison");
        if *cantidad == 0 {
            // Si pude tomar el lock aunque la condicion ya se cumplió
            // Esto es posible ya que hay un tiempo entre el notify_one y que se despierte el thread juego y por lo tanto se lockee la cantidad
            // En este caso, desalojar este thread para permitir que el otro se despierte e intentar nuevamente entra al juego
            drop(cantidad);
            println!("[PERSONA {}] NOTIFICO CANTIDAD 0 cuando ya era 0", persona_id);
            // std::thread::sleep(Duration::from_millis(25 as u64));
            self.cv_cero.notify_one();
            std::thread::yield_now();
            self.acquire(persona_id);
        } else {
            *cantidad -= 1;
            if *cantidad == 0 {
                println!("[PERSONA {}] NOTIFICO CANTIDAD 0", persona_id);
                self.cv_cero.notify_one();
            }
        }
    }

    pub fn release(&self) {
        // self.sem.release();
        let mut cantidad = self.cantidad.lock().expect("poison");
        *cantidad += 1;
    }

    pub fn wait_zero_timeout(&self, dur: Duration) -> (MutexGuard<isize>, WaitTimeoutResult) {
        self.cv_cero.wait_timeout(
            self.cantidad.lock().expect("poisoned"), dur).expect("poisoned")
    }
}