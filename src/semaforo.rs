use std::sync::{Condvar, Mutex, MutexGuard, WaitTimeoutResult};
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

    pub fn acquire(&self) {
        self.sem.acquire();
        let mut cantidad = self.cantidad.lock().expect("poison");
        *cantidad -= 1;
        if *cantidad == 0 {
            self.cv_cero.notify_one();
        }
    }

    pub fn release(&self) {
        self.sem.release();
        let mut cantidad = self.cantidad.lock().expect("poison");
        *cantidad += 1;
    }

    pub fn wait_zero_timeout(&self, dur: Duration) -> (MutexGuard<isize>, WaitTimeoutResult) {
        self.cv_cero.wait_timeout(
            self.cantidad.lock().expect("poisoned"), dur).expect("poisoned")
    }
}