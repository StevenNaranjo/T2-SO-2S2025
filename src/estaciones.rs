use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

pub const QUEUE_CAPACITY: usize = 32;

/// Producto: métricas por estación y estado RR reutilizable.
#[derive(Debug)]
pub struct Product {
    pub id: i32,
    pub arrival_ms: i32,            // llegada simulada (para el productor)
    pub entry_time: [i64; 3],       // ms desde start al "primer entry" por estación
    pub exit_time:  [i64; 3],       // ms desde start al "exit" por estación
    pub entered: [bool; 3],         // true si ya entró alguna vez
    pub queue_arrival: [i64; 4],    // ms desde start a la llegada a la cola de cada estación
    pub finished: bool,             // true cuando sale de la última estación
    pub remaining_rr: i32,          // restante solo para estaciones Round Robin (se reinicia al entrar)
}

impl Product {
    // Crea un producto con métricas en 0.
    pub fn new(id: i32, arrival_ms: i32) -> Self {
        Self {
            id,
            arrival_ms,
            entry_time: [0; 3],
            exit_time: [0; 3],
            entered: [false; 3],
            queue_arrival: [0; 4],
            finished: false,
            remaining_rr: 0,
        }
    }
}

/// Puntero compartido y seguro a mutuo acceso de un Product.
// - `Arc` permite compartir entre hilos.
// - `Mutex` protege el acceso concurrente al `Product`.
pub type SharedProduct = Arc<Mutex<Product>>;

/// Estructura con el buffer real (VecDeque) de punteros a productos.
struct Inner {
    buf: VecDeque<SharedProduct>,
}

/// Cola acotada bloqueante.
pub struct ProdQueue {
    inner: Mutex<Inner>,    // Estado interno protegido por `Mutex`.
    not_empty: Condvar,     // Señal usada para despertar consumidores cuando la cola deja de estar vacía.
    not_full: Condvar,      // Señal usada para despertar productores cuando la cola deja de estar llena.
    capacity: usize,        // Capacidad máxima de la cola.
}

impl ProdQueue {
    // Construye la cola acotada con la capacidad indicada y la envuelve en `Arc` para poder clonarla y compartirla entre hilos.
    pub fn new(capacity: usize) -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(Inner { buf: VecDeque::with_capacity(capacity) }),
            not_empty: Condvar::new(),
            not_full: Condvar::new(),
            capacity,
        })
    }

    // Encola un elemento. Si la cola está llena, se bloquea hasta que haya espacio.
    pub fn push(&self, item: SharedProduct) {
        let mut g = self.inner.lock().unwrap();     // Toma el lock del estado interno.
        while g.buf.len() == self.capacity {        
            g = self.not_full.wait(g).unwrap();     // Mientras esté llena, espera a `not_full`.
        }
        g.buf.push_back(item);
        self.not_empty.notify_one();                // Despierta a un consumidor que esté esperando a `not_empty`.
    }

    // Desencola un elemento. Si la cola está vacía, se bloquea hasta que haya ítems.
    pub fn pop(&self) -> SharedProduct {
        let mut g = self.inner.lock().unwrap();     // Toma el lock del estado interno.
        while g.buf.is_empty() {
            g = self.not_empty.wait(g).unwrap();    // Mientras esté vacía, espera a `not_empty`
        }
        let it = g.buf.pop_front().expect("checked non-empty");
        self.not_full.notify_one();                 // Despierta a un productor que esté esperando a `not_full`.
        it
    }

    // Devuelve el tamaño actual de la cola.
    pub fn len(&self) -> usize {
        self.inner.lock().unwrap().buf.len()
    }
}

/// Convierte tiempo desde `start` a milisegundos (i64).
pub fn ms_since(start: Instant) -> i64 {
    let d = start.elapsed();
    (d.as_secs() as i64) * 1000 + (d.subsec_millis() as i64)
}

/// Duerme `ms` milisegundos.
pub fn sleep_ms(ms: i32) {
    std::thread::sleep(Duration::from_millis(ms as u64));
}
