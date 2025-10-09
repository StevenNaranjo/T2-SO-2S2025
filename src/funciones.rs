use std::sync::Arc;
use std::time::Instant;

use crate::estaciones::{ms_since, sleep_ms, ProdQueue};

/// Estación FCFS
pub fn estacion_fcfs(
    name: &'static str,
    station_idx: usize,
    work_time_ms: i32,
    in_q: Arc<ProdQueue>,
    out_q: Option<Arc<ProdQueue>>,
    start: Instant,
) {
    loop {
        let sp = in_q.pop();
        {
            let p = sp.lock().unwrap();
            println!("{}ms, {}: Procesando producto #{} (tiempo: {} ms)", ms_since(start), name, p.id, work_time_ms);
        }

        // Marcar primer entry
        {
            let mut p = sp.lock().unwrap();
            if p.entered[station_idx] == false {
                p.entered[station_idx] = true;
                p.entry_time[station_idx] = ms_since(start);
            }
        }

        // Trabajo
        sleep_ms(work_time_ms);

        // Salida
        let mut p = sp.lock().unwrap();
        p.exit_time[station_idx] = ms_since(start);
        println!("{}ms, {}: Producto #{} finalizado en esta estación", ms_since(start), name, p.id);
        match &out_q {
            Some(q_out) => q_out.push(sp.clone()),
            None => p.finished = true,
        }
    }
}

/// Estación Round Robin
pub fn estacion_round_robin(
    name: &'static str,
    station_idx: usize,
    quantum_ms: i32,
    work_time_ms: i32,
    in_q: Arc<ProdQueue>,
    out_q: Option<Arc<ProdQueue>>,
    start: Instant,
) {
    loop {
        let sp = in_q.pop();

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

        // Slice
        let slice = {
            let p = sp.lock().unwrap();
            p.remaining_rr.max(0).min(quantum_ms)
        };

        {
            let p = sp.lock().unwrap();
            println!("{}ms, {}: Procesando producto #{} por {} ms (restante antes del slice: {} ms)", ms_since(start), name, p.id, slice, p.remaining_rr);
        }

        sleep_ms(slice);

        let mut p = sp.lock().unwrap();
        p.remaining_rr -= slice;

        if p.remaining_rr <= 0 {
            p.exit_time[station_idx] = ms_since(start);
            println!("{}ms, {}: Producto #{} finalizado en esta estación", ms_since(start), name, p.id);
            match &out_q {
                Some(q_out) => q_out.push(sp.clone()),
                None => p.finished = true,
            }
        } else {
            //println!("{}ms, {}: Producto #{} reencolado (quedan {} ms)", ms_since(start), name, p.id, p.remaining_rr);
            drop(p);
            in_q.push(sp.clone());
        }
    }
}
