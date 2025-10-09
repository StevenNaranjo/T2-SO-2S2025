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
    e1_in: i64,  e1_out: i64,
    e2_in: i64,  e2_out: i64,
    e3_in: i64,  e3_out: i64,
    turnaround: i64,
    wait_time: i64,
}

fn print_report(rows: &[ReportRow]) {
    // Cabecera
    println!("\n{:=^120}", "  INFORME FINAL  ");
    println!(
        "{:<4} {:>8} │ {:>8} {:>8} │ {:>8} {:>8} │ {:>8} {:>8} │ {:>10} {:>10}",
        "ID", "Arr(ms)",
        "E1_in", "E1_out",
        "E2_in", "E2_out",
        "E3_in", "E3_out",
        "Turn(ms)", "Wait(ms)"
    );
    println!("{:-<120}", "");

    // Filas
    for r in rows {
        println!(
            "{:<4} {:>8} │ {:>8} {:>8} │ {:>8} {:>8} │ {:>8} {:>8} │ {:>10} {:>10}",
            r.id, r.arrival,
            r.e1_in, r.e1_out,
            r.e2_in, r.e2_out,
            r.e3_in, r.e3_out,
            r.turnaround, r.wait_time
        );
    }

    // Promedios
    let n = rows.len() as f64;
    let avg_turn: f64 = rows.iter().map(|r| r.turnaround as f64).sum::<f64>() / n;
    let avg_wait: f64 = rows.iter().map(|r| r.wait_time as f64).sum::<f64>() / n;

    println!("{:-<120}", "");
    println!(
        "{:<4} {:>8} │ {:>8} {:>8} │ {:>8} {:>8} │ {:>8} {:>8} │ {:>10.2} {:>10.2}",
        "AVG", "", "", "", "", "", "", "", avg_turn, avg_wait
    );
    println!("{:=<120}", "");
}

fn main() {
    // ---------- CONFIGURACIÓN ----------
    
    // Llegadas manuales. *(se interpretan como offsets desde start)*
    let arrivals_ms: Vec<i32> = vec![0, 0, 50, 50, 100, 100, 150, 200, 250, 300]; 
    let n_products = arrivals_ms.len();

    // Tiempos de proceso por estación (ms)
    let e1_work_ms = 140;   // FCFS
    let e2_work_ms = 220;   // RR (tiempo total por producto)
    let e2_quantum  = 80;   // slice RR
    let e3_work_ms = 120;   // FCFS

    // ---------- RELOJ ----------
    let start = Instant::now();

    // ---------- COLAS ----------
    let q_e1_in = ProdQueue::new(QUEUE_CAPACITY);
    let q_e2_in = ProdQueue::new(QUEUE_CAPACITY);
    let q_e3_in = ProdQueue::new(QUEUE_CAPACITY);
    let q_done  = ProdQueue::new(QUEUE_CAPACITY);

    // ---------- GENERADOR ----------
    {
        let q_e1_in = Arc::clone(&q_e1_in);
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
                q_e1_in.push(sp);
            }
        });
    }

    // ---------- ESTACIÓN 1: FCFS ----------
    {
        let in_q = Arc::clone(&q_e1_in);
        let out_q = Some(Arc::clone(&q_e2_in));
        let start1 = start.clone();
        thread::spawn(move || {
            estacion_fcfs("Estación 1 (FCFS)", 0, e1_work_ms, in_q, out_q, start1);
        });
    }

    // ---------- ESTACIÓN 2: RR ----------
    {
        let in_q = Arc::clone(&q_e2_in);
        let out_q = Some(Arc::clone(&q_e3_in));
        let start2 = start.clone();
        thread::spawn(move || {
            estacion_round_robin("Estación 2 (RR)", 1, e2_quantum, e2_work_ms, in_q, out_q, start2);
        });
    }

    // ---------- ESTACIÓN 3: FCFS ----------
    {
        let in_q = Arc::clone(&q_e3_in);
        let out_q = Some(Arc::clone(&q_done));
        let start3 = start.clone();
        thread::spawn(move || {
            estacion_fcfs("Estación 3 (FCFS)", 2, e3_work_ms, in_q, out_q, start3);
        });
    }

    // ---------- RECOLECTOR FINAL ----------
    let mut finished = 0usize;
    let mut report_rows: Vec<ReportRow> = Vec::with_capacity(n_products);

    while finished < n_products {

        let sp = q_done.pop();

        // Tomamos el lock SOLO para leer/copiar los datos; marcamos terminado aquí.
        let (id, arrival, e1_in, e1_out, e2_in, e2_out, e3_in, e3_out, turnaround, wait_time);
        {
            let mut p = sp.lock().unwrap();
            p.finished = true;

            id = p.id;
            arrival = p.arrival_ms;
            e1_in = p.entry_time[0];  e1_out = p.exit_time[0];
            e2_in = p.entry_time[1];  e2_out = p.exit_time[1];
            e3_in = p.entry_time[2];  e3_out = p.exit_time[2];

            turnaround = e3_out - arrival as i64;

            // Como los tiempos de proceso son constantes por estación, el total procesado es la suma:
            let processing_sum = (e1_work_ms + e2_work_ms + e3_work_ms) as i64;
            wait_time = turnaround - processing_sum;
        } // <-- se libera el mutex aquí

        report_rows.push(ReportRow {
            id, arrival, e1_in, e1_out, e2_in, e2_out, e3_in, e3_out, turnaround, wait_time,
        });

        finished += 1;
    }

    // ---------- IMPRESIÓN DEL INFORME ----------
    println!("\nTiempos de procesamiento por estación:");
    println!(" - Corte      (#1, FCFS) → {} ms", e1_work_ms);
    println!(" - Ensamblaje (#2, RR)   → {} ms", e2_work_ms);
    println!(" - Empaque    (#3, FCFS) → {} ms", e3_work_ms);

    println!("\nEstado final de las colas:");
    println!(" - Cola E1_in  → {} elementos", q_e1_in.len());
    println!(" - Cola E2_in  → {} elementos", q_e2_in.len());
    println!(" - Cola E3_in  → {} elementos", q_e3_in.len());
    println!(" - Cola Done   → {} elementos", q_done.len());

    println!("\nOrden final de procesamiento:");
    let orden_final: Vec<i32> = report_rows.iter().map(|r| r.id).collect();
    println!("{:?}", orden_final);

    print_report(&report_rows);
}
