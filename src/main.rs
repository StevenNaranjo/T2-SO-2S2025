mod estaciones;
mod funciones;

use std::env;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use estaciones::{Product, ProdQueue, SharedProduct, QUEUE_CAPACITY, sleep_ms, ms_since};
use funciones::{estacion_fcfs, estacion_round_robin};

/// Fila del informe final con las métricas de cada producto tras pasar por 3 estaciones.
#[derive(Debug)]
struct ReportRow {
    id: i32,                    // ID del producto
    arrival: i32,               // llegada real (ms desde start) medida por el generador
    e1_in: i64,  e1_out: i64,   // tiempos de entrada/salida en estación 1
    e2_in: i64,  e2_out: i64,   // tiempos de entrada/salida en estación 2
    e3_in: i64,  e3_out: i64,   // tiempos de entrada/salida en estación 3
    turnaround: i64,            // tiempo total en el sistema = e3_out - arrival
    wait_time: i64,             // tiempo total de espera = turnaround - (suma de work_ms de cada estación)
}

/// Tipo de planificación de una estación: FCFS o RR (con quantum).
#[derive(Clone, Copy, Debug)]
enum StationKind {
    Fcfs,
    Rr { q: i32 },
}

/// Configuración concreta de una estación: tipo y duración de servicio.
#[derive(Clone, Copy, Debug)]
struct StationCfg {
    kind: StationKind,
    work_ms: i32,
}

/// Parseo de CLI para una estación: <tipo> <dur_ms> [<q_si_es_rr>]
fn parse_station(args: &[String], idx: &mut usize, label: &str) -> Result<StationCfg, String> {
    
    // Verifica que haya al menos un token para el tipo
    if *idx >= args.len() {
        return Err(format!("Faltan argumentos para {}", label));
    }
    let ty = args[*idx].to_lowercase();
    *idx += 1;

    // Siempre se espera la duración a continuación
    if *idx >= args.len() {
        return Err(format!("Falta duración (ms) para {}", label));
    }
    let work_ms: i32 = args[*idx]
        .parse()
        .map_err(|_| format!("Duración inválida para {}: {}", label, args[*idx]))?;
    if work_ms <= 0 {
        return Err(format!("Duración para {} debe ser > 0", label));
    }
    *idx += 1;

    // Si es rr, también espera quantum; si es fcfs, no espera nada más
    match ty.as_str() {
        "fcfs" => Ok(StationCfg { kind: StationKind::Fcfs, work_ms }),
        "rr" => {
            if *idx >= args.len() {
                return Err(format!("Falta quantum para {} (tipo rr)", label));
            }
            let q: i32 = args[*idx]
                .parse()
                .map_err(|_| format!("Quantum inválido para {}: {}", label, args[*idx]))?;
            if q <= 0 {
                return Err(format!("Quantum para {} debe ser > 0", label));
            }
            *idx += 1;
            Ok(StationCfg { kind: StationKind::Rr { q }, work_ms })
        }
        other => Err(format!("Tipo de estación desconocido para {}: {}", label, other)),
    }
}

/// Imprime la tabla del informe con promedios.
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
    // ---------- CLI ----------
    let args: Vec<String> = env::args().collect();
    let mut i = 1;
    let e1 = parse_station(&args, &mut i, "Estación 1").unwrap_or_else(|e| {
        eprintln!("Uso:\n  {} <e1_tipo> <e1_ms> [e1_q] <e2_tipo> <e2_ms> [e2_q] <e3_tipo> <e3_ms> [e3_q]\n\
                   Donde tipo ∈ {{fcfs, rr}}; si tipo=rr, debe pasarse quantum (ms).\n\
                   Ejemplos:\n  {} fcfs 140 rr 220 80 fcfs 120\n  {} rr 140 50 rr 220 120 rr 120 80\nError: {}",
                  args.get(0).map(String::as_str).unwrap_or("bin"),
                  args.get(0).map(String::as_str).unwrap_or("bin"),
                  args.get(0).map(String::as_str).unwrap_or("bin"),
                  e);
        std::process::exit(1);
    });
    let e2 = parse_station(&args, &mut i, "Estación 2").unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    });
    let e3 = parse_station(&args, &mut i, "Estación 3").unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    });

    // ---------- CONFIGURACIÓN ----------
    let arrivals_ms: Vec<i32> = vec![0, 0, 50, 50, 100, 100, 150, 200, 250, 300];   // Llegadas programadas como offsets en ms desde `start`.
    let n_products = arrivals_ms.len();

    // ---------- RELOJ ----------
    let start = Instant::now();

    // ---------- COLAS ----------
    let q_e1_in = ProdQueue::new(QUEUE_CAPACITY);
    let q_e2_in = ProdQueue::new(QUEUE_CAPACITY);
    let q_e3_in = ProdQueue::new(QUEUE_CAPACITY);
    let q_done  = ProdQueue::new(QUEUE_CAPACITY);

    // ---------- GENERADOR ----------
    // Hilo productor que crea productos y los va encolando en E1 según sus offsets.
    {
        let q_e1_in = Arc::clone(&q_e1_in);
        let start0 = start;
        thread::spawn(move || {
            // Interpreta arrivals_ms como offsets (ms desde start)
            for (i, &offset_ms) in arrivals_ms.iter().enumerate() {

                // Cada producto es un Arc<Mutex<Product>> para compartirlo entre hilos con seguridad
                let sp: SharedProduct = Arc::new(std::sync::Mutex::new(Product::new(i as i32, 0)));

                // Espera hasta su tiempo de llegada programado
                sleep_ms(offset_ms);

                // Registra la hora real de llegada
                let arrival_now = ms_since(start0) as i32;
                {
                    let mut p = sp.lock().unwrap();
                    p.arrival_ms = arrival_now;
                }

                println!("{}ms, Generador: Encolando producto #{} (arrival real: {} ms, offset pedido: {} ms)", ms_since(start0), i, arrival_now, offset_ms);

                // Encola a la estación 1
                q_e1_in.push(sp);
            }
        });
    }

    // ---------- ESTACIÓN 1 ----------
    {
        let in_q = Arc::clone(&q_e1_in);
        let out_q = Some(Arc::clone(&q_e2_in));
        let start1 = start.clone();

        match e1.kind {
            StationKind::Fcfs => {
                thread::spawn(move || {
                    estacion_fcfs("Estación 1 (FCFS)", 0, e1.work_ms, in_q, out_q, start1);
                });
            }
            StationKind::Rr { q } => {
                thread::spawn(move || {
                    estacion_round_robin("Estación 1 (RR)", 0, q, e1.work_ms, in_q, out_q, start1);
                });
            }
        }
    }

    // ---------- ESTACIÓN 2 ----------
    {
        let in_q = Arc::clone(&q_e2_in);
        let out_q = Some(Arc::clone(&q_e3_in));
        let start2 = start.clone();

        match e2.kind {
            StationKind::Fcfs => {
                thread::spawn(move || {
                    estacion_fcfs("Estación 2 (FCFS)", 1, e2.work_ms, in_q, out_q, start2);
                });
            }
            StationKind::Rr { q } => {
                thread::spawn(move || {
                    estacion_round_robin("Estación 2 (RR)", 1, q, e2.work_ms, in_q, out_q, start2);
                });
            }
        }
    }

    // ---------- ESTACIÓN 3 ----------
    {
        let in_q = Arc::clone(&q_e3_in);
        let out_q = Some(Arc::clone(&q_done));
        let start3 = start.clone();

        match e3.kind {
            StationKind::Fcfs => {
                thread::spawn(move || {
                    estacion_fcfs("Estación 3 (FCFS)", 2, e3.work_ms, in_q, out_q, start3);
                });
            }
            StationKind::Rr { q } => {
                thread::spawn(move || {
                    estacion_round_robin("Estación 3 (RR)", 2, q, e3.work_ms, in_q, out_q, start3);
                });
            }
        }
    }

    // ---------- RECOLECTOR FINAL ----------
    let mut finished = 0usize;
    let mut report_rows: Vec<ReportRow> = Vec::with_capacity(n_products);

    while finished < n_products {

        // Espera el próximo producto finalizado
        let sp = q_done.pop();

        // Lee todos los campos que necesita bajo lock y calcula métricas
        let (id, arrival, e1_in, e1_out, e2_in, e2_out, e3_in, e3_out, turnaround, wait_time);
        {
            let mut p = sp.lock().unwrap();
            p.finished = true;

            id = p.id;
            arrival = p.arrival_ms;
            e1_in = p.entry_time[0];  e1_out = p.exit_time[0];
            e2_in = p.entry_time[1];  e2_out = p.exit_time[1];
            e3_in = p.entry_time[2];  e3_out = p.exit_time[2];

            // turnaround: tiempo total desde llegada real hasta salida de E3
            turnaround = e3_out - arrival as i64;

            // tiempo de servicio total: suma de las duraciones configuradas
            let processing_sum = (e1.work_ms + e2.work_ms + e3.work_ms) as i64;

            // espera total: turnaround - servicio
            wait_time = turnaround - processing_sum;
        } // Se libera el mutex aquí

        // Guarda la fila en el vector para imprimir luego
        report_rows.push(ReportRow {
            id, arrival, e1_in, e1_out, e2_in, e2_out, e3_in, e3_out, turnaround, wait_time,
        });

        finished += 1;
    }

    // ---------- IMPRESIÓN DEL INFORME ----------
    println!("\nTiempos de procesamiento por estación:");
    match e1.kind {
        StationKind::Fcfs => println!(" - Corte      (#1, FCFS)            → {} ms", e1.work_ms),
        StationKind::Rr { q } => println!(" - Corte      (#1, RR, q={})        → {} ms", q, e1.work_ms),
    }
    match e2.kind {
        StationKind::Fcfs => println!(" - Ensamblaje (#2, FCFS)            → {} ms", e2.work_ms),
        StationKind::Rr { q } => println!(" - Ensamblaje (#2, RR, q={})        → {} ms", q, e2.work_ms),
    }
    match e3.kind {
        StationKind::Fcfs => println!(" - Empaque    (#3, FCFS)            → {} ms", e3.work_ms),
        StationKind::Rr { q } => println!(" - Empaque    (#3, RR, q={})        → {} ms", q, e3.work_ms),
    }

    println!("\nOrden final de procesamiento:");
    let orden_final: Vec<i32> = report_rows.iter().map(|r| r.id).collect();
    println!("{:?}", orden_final);

    print_report(&report_rows);
}
