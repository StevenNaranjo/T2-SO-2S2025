mod estaciones;
mod funciones;

use std::sync::Arc;
use std::thread;
use std::time::Instant;

use estaciones::{Product, ProdQueue, SharedProduct, QUEUE_CAPACITY, sleep_ms, ms_since};
use funciones::{estacion_fcfs, estacion_round_robin};

#[derive(Debug)]
struct ReportRow {
    id: i32,
    arrival: i32,
    s_in: i64,   s_out: i64,
    turnaround: i64,
    wait_time: i64,
}

fn print_report(rows: &[ReportRow]) {
    // Cabecera
    println!("\n{:=^86}", "  INFORME FINAL  ");
    println!(
        "{:<4} {:>8} │ {:>8} {:>8} │ {:>10} {:>10}",
        "ID", "Arr(ms)",
        "S_in", "S_out",
        "Turn(ms)", "Wait(ms)"
    );
    println!("{:-<86}", "");

    // Filas
    for r in rows {
        println!(
            "{:<4} {:>8} │ {:>8} {:>8} │ {:>10} {:>10}",
            r.id, r.arrival,
            r.s_in, r.s_out,
            r.turnaround, r.wait_time
        );
    }

    // Promedios
    let n = rows.len() as f64;
    let avg_turn: f64 = rows.iter().map(|r| r.turnaround as f64).sum::<f64>() / n;
    let avg_wait: f64 = rows.iter().map(|r| r.wait_time as f64).sum::<f64>() / n;

    println!("{:-<86}", "");
    println!(
        "{:<4} {:>8} │ {:>8} {:>8} │ {:>10.2} {:>10.2}",
        "AVG", "", "", "", avg_turn, avg_wait
    );
    println!("{:=<86}", "");
}

fn main() {
    // ---------- CONFIGURACIÓN ----------
    // Llegadas manuales como offsets desde start (ms)
    let arrivals_ms: Vec<i32> = vec![0, 0];
    let n_products = arrivals_ms.len();

    // Tiempo de proceso de la estación
    let s_work_ms = 220;
    let s_quantum = 80;

    // ---------- RELOJ ----------
    let start = Instant::now();

    // ---------- COLAS ----------
    let q_in   = ProdQueue::new(QUEUE_CAPACITY);
    let q_done = ProdQueue::new(QUEUE_CAPACITY);

    // ---------- GENERADOR ----------
    // {
    //     let q_e1_in = Arc::clone(&q_e1_in);
    //     thread::spawn(move || {
    //         for (i, &arr) in arrivals_ms.iter().enumerate() {
    //             let sp: SharedProduct = Arc::new(std::sync::Mutex::new(Product::new(i as i32, arr)));
    //             sleep_ms(arr);
    //             println!("Generador: Encolando producto #{} (llegada: {} ms)", i, arr);
    //             q_e1_in.push(sp);
    //         }
    //     });
    // }
    {
        let q_in = Arc::clone(&q_in);
        let start0 = start;
        thread::spawn(move || {
            // interpreta arrivals_ms como OFFSETS (ms desde start)
            for (i, &offset_ms) in arrivals_ms.iter().enumerate() {
                let sp: SharedProduct = Arc::new(std::sync::Mutex::new(Product::new(i as i32, 0)));

                // espera hasta el offset indicado
                sleep_ms(offset_ms);

                // registra la hora REAL de llegada
                let arrival_now = ms_since(start0) as i32;
                {
                    let mut p = sp.lock().unwrap();
                    p.arrival_ms = arrival_now;
                }

                println!("{}ms, Generador: Encolando producto #{} (arrival real: {} ms, offset pedido: {} ms)", ms_since(start), i, arrival_now, offset_ms);
                q_in.push(sp);
            }
        });
    }

    // ---------- ESTACIÓN ----------
    {
        let in_q = Arc::clone(&q_in);
        let out_q = Some(Arc::clone(&q_done));
        let start_s = start.clone();

        // ======== USA UNA DE LAS DOS OPCIONES ========

        // Opción A: Round Robin
        thread::spawn(move || {
            estacion_round_robin("Estación (RR)", 0, s_quantum, s_work_ms, in_q, out_q, start_s);
        });

        // Opción B: FCFS
        // thread::spawn(move || {
        //     estacion_fcfs("Estación (FCFS)", 0, s_work_ms, in_q, out_q, start_s);
        // });
    }

    // ---------- RECOLECTOR FINAL ----------
    let mut finished = 0usize;
    let mut report_rows: Vec<ReportRow> = Vec::with_capacity(n_products);

    while finished < n_products {
        let sp = q_done.pop();

        // Leemos los datos finales del producto
        let (id, arrival, s_in, s_out, turnaround, wait_time);
        {
            let mut p = sp.lock().unwrap();
            p.finished = true;

            id = p.id;
            arrival = p.arrival_ms;
            s_in  = p.entry_time[0];
            s_out = p.exit_time[0];

            turnaround = s_out - arrival as i64;

            let processing_sum = s_work_ms as i64;
            wait_time = turnaround - processing_sum;
        } // <-- se libera el mutex aquí

        report_rows.push(ReportRow { id, arrival, s_in, s_out, turnaround, wait_time });
        finished += 1;
    }

    // ---------- IMPRESIÓN DEL INFORME ----------
    println!("\nTiempo de procesamiento de la estación → {} ms", s_work_ms);

    println!("\nEstado final de las colas:");
    println!("Cola IN   → {} elementos", q_in.len());
    println!("Cola DONE → {} elementos", q_done.len());

    println!("\nOrden final de procesamiento:");
    let orden_final: Vec<i32> = report_rows.iter().map(|r| r.id).collect();
    println!("{:?}", orden_final);

    print_report(&report_rows);
}
