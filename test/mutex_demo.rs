use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

fn demo_seccion_critica() {
    println!("=== Sección crítica con Mutex ===");

    const N_THREADS: usize = THREADS;

    // "Puerta" sin datos, solo para exclusión mutua. Un lock de () es un patrón común.
    let puerta = Arc::new(Mutex::new(()));

    let mut joins = Vec::with_capacity(N_THREADS);
    
    for id in 0..N_THREADS {
        let puerta = Arc::clone(&puerta);
        
        joins.push(thread::spawn(move || {
            // Simula trabajo antes de intentar entrar
            thread::sleep(Duration::from_millis(50 as u64));

            // Tomar el lock de la puerta (entrar a la sección crítica)
            let _pass = puerta.lock().expect("mutex poisoned");

            // A partir de aquí, solo hay un hilo dentro de la sección crítica a la vez
            println!("Hilo {id}: ENTER");

            // Simula trabajo dentro de la sección crítica
            thread::sleep(Duration::from_millis(10 * id as u64));
            
            println!("Hilo {id}: EXIT");

            // _pass se suelta al final del scope y otro hilo podrá entrar
        }));
    }

    for j in joins {
        j.join().expect("hilo falló");
    }
}

fn demo_seccion_critica_sin_mutex() {
    println!("=== Sección crítica sin Mutex ===");

    const N_THREADS: usize = THREADS;

    let mut joins = Vec::with_capacity(N_THREADS);
    
    for id in 0..N_THREADS {
        
        joins.push(thread::spawn(move || {
            // Simula trabajo antes de intentar entrar
            thread::sleep(Duration::from_millis(50 as u64));

            // Eentra a la sección crítica
            println!("Hilo {id}: ENTER");

            // Simula trabajo dentro de la sección crítica
            thread::sleep(Duration::from_millis(10 * id as u64));
            
            println!("Hilo {id}: EXIT");
        }));
    }

    for j in joins {
        j.join().expect("hilo falló");
    }
}

pub const THREADS: usize = 20;

fn main() {
    /*
        Al correr este programa, se puede observar que en la primera demo (con Mutex),
            los mensajes de ENTER y EXIT están ordenados y no se intercalan, demostrando
            que solo un hilo está en la sección crítica a la vez.
        En la segunda demo (sin Mutex), los mensajes de ENTER y EXIT se intercalan,
            mostrando que múltiples hilos entran a la sección crítica simultáneamente.
    */
    demo_seccion_critica();
    println!();
    demo_seccion_critica_sin_mutex()
}
