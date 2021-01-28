use std::{
    fs::File, 
    io::prelude::*, 
    sync::{Arc, Mutex}, 
    time::{Duration, Instant}
};

pub struct Logger {
    file: Option<Mutex<File>>,
    timer: Instant
}

impl Logger {
    pub fn new_to_stdout() -> Self {
        Self { file: None, timer: Instant::now() }
    }

    pub fn new_to_file(path: &str) -> Result<Self, String> {
        Ok(Self {
            file: Some(Mutex::new(
                File::create(path)
                    .map_err(|e| e.to_string())?
            )),
            timer: Instant::now()
        })
    }

    /// Escribe msg al log sin agregar nada (ni salto de línea,
    /// ni etiquetas).
    pub fn write_raw(&self, msg: &str) {
        if let Some(file_mutex) = &self.file {
            let mut file = file_mutex.lock().expect("log poisoned");
            file.write_all(msg.as_bytes())
                .expect("No se puede escribir al archivo de log.");
        } else {
            print!("{}", msg);
        }
    }

    /// Vacía los buffers y cierra el archivo de log.
    pub fn close(&self) {
        if let Some(mutex_lock) = &self.file {
            let mut file = mutex_lock.lock().expect("log mutex poisoned");
            file.flush().expect("Error al flushear el log");
        }
    }

    /// Obtiene el tiempo que pasó desde que se creó el Logger hasta
    /// que se llamó a este método.
    pub fn get_elapsed_time(&self) -> Duration {
        self.timer.elapsed()
    }
}

pub struct TaggedLogger {
    tag: String,
    logger: Arc<Logger>
}

impl TaggedLogger {
    pub fn new(tag: &str, logger: Arc<Logger>) -> Self {
        Self {
            tag: tag.into(),
            logger
        }
    }

    /// Escribe un mensaje al log. 
    ///
    /// El mensaje estará marcado con la etiqueta correspondiente
    /// y un timestamp.
    pub fn write(&self, msg: &str) {
        let time = self.logger.get_elapsed_time();

        self.logger.write_raw(&format!(
            "{:8.3}| {:>12}| {}\n", time.as_secs_f32(), self.tag, msg
        ));
    }
}