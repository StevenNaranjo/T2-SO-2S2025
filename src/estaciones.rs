use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

pub const QUEUE_CAPACITY: usize = 32;

/// Producto: métricas por estación y estado RR reutilizable.
#[derive(Debug)]
pub struct Product {
    pub id: i32,
    pub arrival_ms: i32,        // llegada simulada (para el productor)
    pub entry_time: [i64; 3],   // ms desde start al "primer entry" por estación
    pub exit_time:  [i64; 3],   // ms desde start al "exit" por estación
    pub entered: [bool; 3],     // true si ya entró alguna vez
    pub finished: bool,         // true cuando sale de la última estación
    pub remaining_rr: i32,      // restante solo para estaciones Round Robin (se reinicia al entrar)
}

impl Product {
    pub fn new(id: i32, arrival_ms: i32) -> Self {
        Self {
            id,
            arrival_ms,
            entry_time: [0; 3],
            exit_time: [0; 3],
            entered: [false; 3],
            finished: false,
            remaining_rr: 0,
        }
    }
}

pub type SharedProduct = Arc<Mutex<Product>>;

/// Cola acotada bloqueante (items/spaces ~ Condvar).
pub struct ProdQueue {
    inner: Mutex<Inner>,
    not_empty: Condvar,
    not_full: Condvar,
    capacity: usize,
}

struct Inner {
    buf: VecDeque<SharedProduct>,
}

impl ProdQueue {
    pub fn new(capacity: usize) -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(Inner { buf: VecDeque::with_capacity(capacity) }),
            not_empty: Condvar::new(),
            not_full: Condvar::new(),
            capacity,
        })
    }

    pub fn push(&self, item: SharedProduct) {
        let mut g = self.inner.lock().unwrap();
        while g.buf.len() == self.capacity {
            g = self.not_full.wait(g).unwrap();
        }
        g.buf.push_back(item);
        self.not_empty.notify_one();
    }

    pub fn pop(&self) -> SharedProduct {
        let mut g = self.inner.lock().unwrap();
        while g.buf.is_empty() {
            g = self.not_empty.wait(g).unwrap();
        }
        let it = g.buf.pop_front().expect("checked non-empty");
        self.not_full.notify_one();
        it
    }

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
