use std::sync::Arc;
use std::time::Instant;

use crate::estaciones::{ms_since, sleep_ms, ProdQueue};

/// Estación First-Come, First-Served
/*  
    Simula una estación que atiende los productos en orden de llegada (FCFS).
    Cada producto se procesa por un tiempo fijo (`work_time_ms`) y luego pasa a la siguiente estación o termina si no hay más.
*/
pub fn estacion_fcfs(
    name: &'static str,             // nombre de la estación
    station_idx: usize,             // índice de la estación
    work_time_ms: i32,              // tiempo de servicio fijo
    in_q: Arc<ProdQueue>,           // cola de entrada (bloqueante)
    out_q: Option<Arc<ProdQueue>>,  // cola de salida
    start: Instant,                 // tiempo de inicio del simulador
) {
    loop {
        let sp = in_q.pop();        // Espera hasta que haya un producto en la cola de entrada.
        
        // Lectura breve bajo lock (lectura bajo lock)
        {
            let p = sp.lock().unwrap();
            println!("{}ms, {}: Procesando producto #{} (tiempo: {} ms)", ms_since(start), name, p.id, work_time_ms);
        }

        // Marcar primer entry (escritura bajo lock)
        {
            let mut p = sp.lock().unwrap();
            if p.entered[station_idx] == false {
                p.entered[station_idx] = true;
                p.entry_time[station_idx] = ms_since(start);
            }
        }

        // Simulación del Trabajo
        sleep_ms(work_time_ms);

        // Marcar salida y encolar en la siguiente cola (escritura bajo lock)
        let mut p = sp.lock().unwrap();
        
        p.exit_time[station_idx] = ms_since(start);
        println!("{}ms, {}: Producto #{} finalizado en esta estación", ms_since(start), name, p.id);
        
        match &out_q {
            Some(q_out) => {
                    let next = station_idx + 1;
                    p.queue_arrival[next] = ms_since(start);
                    q_out.push(sp.clone());
                },
            None => p.finished = true,
        }
    }
}

/// Estación Round Robin
/*
    Simula una estación con planificación Round Robin.
    Cada producto recibe un “slice” (quantum) de CPU, y si no termina, se reencola para continuar en la siguiente ronda.
*/
pub fn estacion_round_robin(
    name: &'static str,             // nombre de la estación
    station_idx: usize,             // índice de la estación
    quantum_ms: i32,                // quantum de CPU
    work_time_ms: i32,              // tiempo total de trabajo requerido
    in_q: Arc<ProdQueue>,           // cola de entrada (bloqueante)
    out_q: Option<Arc<ProdQueue>>,  // cola de salida
    start: Instant,                 // tiempo de inicio del simulador
) {
    loop {
        let sp = in_q.pop();        // Espera hasta que haya un producto en la cola de entrada.

        // Marcar primer entry (escritura bajo lock)
        {
            let mut p = sp.lock().unwrap();
            if p.entered[station_idx] == false {
                p.entered[station_idx] = true;
                p.entry_time[station_idx] = ms_since(start);
                p.remaining_rr = work_time_ms;
                //println!("{}ms, {}: Inicia producto #{} (tiempo total {} ms, quantum {} ms)", ms_since(start), name, p.id, work_time_ms, quantum_ms);
            } else {
                //println!("{}ms, {}: Reanuda producto #{} (restante {} ms)", ms_since(start), name, p.id, p.remaining_rr);
            }
        }

        // Calcular slice de esta ronda (lectura bajo lock)
        let slice = {
            let p = sp.lock().unwrap();
            p.remaining_rr.max(0).min(quantum_ms) // mínimo entre lo que queda y el quantum
        };

        // Log de ejecución del slice (lectura bajo lock)
        {
            let p = sp.lock().unwrap();
            println!("{}ms, {}: Procesando producto #{} por {} ms (restante antes del slice: {} ms)", ms_since(start), name, p.id, slice, p.remaining_rr);
        }

        // Simulación del Trabajo
        sleep_ms(slice);

        // Actualizar restante y decidir si termina o reencola (escritura bajo lock)
        let mut p = sp.lock().unwrap();
        p.remaining_rr -= slice;

        if p.remaining_rr <= 0 {    // Producto terminado, marcar salida y encolar en la siguiente cola
            p.exit_time[station_idx] = ms_since(start);
            println!("{}ms, {}: Producto #{} finalizado en esta estación", ms_since(start), name, p.id);
            
            match &out_q {
                Some(q_out) => {
                    let next = station_idx + 1;
                    p.queue_arrival[next] = ms_since(start);
                    q_out.push(sp.clone());
                },
                None => p.finished = true,
            }
        } else {                    // Producto no terminado, reencolar
            //println!("{}ms, {}: Producto #{} reencolado (quedan {} ms)", ms_since(start), name, p.id, p.remaining_rr);
            drop(p);
            in_q.push(sp.clone());
        }
    }
}
