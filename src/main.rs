//! # Simulador de Línea de Ensamblaje - Aplicación Principal
//! 
//! Esta aplicación implementa un simulador de línea de ensamblaje industrial
//! que utiliza algoritmos de planificación de procesos para gestionar el
//! flujo de productos a través de múltiples estaciones de trabajo.
//! 
//! ## Características
//! 
//! - **Algoritmos de scheduling**: FCFS y Round Robin con quantum configurable
//! - **Comunicación interprocesos**: Canales de comunicación entre hilos
//! - **Sincronización**: Uso de Arc<Mutex<T>> para compartir datos
//! - **Métricas detalladas**: Análisis de rendimiento y tiempos de espera
//! 
//! ## Uso
//! 
//! ```bash
//! # FCFS (First-Come First-Served)
//! cargo run -- fcfs
//! 
//! # Round Robin con quantum personalizado (default: 300ms)
//! cargo run -- rr 250
//! ```

use std::env;
use std::process;

use assembly_line_simulator::{
    config,
    SchedulingAlgorithm,
    Simulation,
};

fn main() {
    // Parsear argumentos de línea de comandos
    let algorithm = match parse_args() {
        Ok(alg) => alg,
        Err(err) => {
            eprintln!("Error: {}", err);
            print_usage();
            process::exit(1);
        }
    };

    // Ejecutar simulación
    let mut simulation = Simulation::new(algorithm);
    let metrics = simulation.run();
    
    // Generar y mostrar reporte
    let report = simulation.generate_report(&metrics);
    println!("{}", report);
}

/// Parsea los argumentos de línea de comandos y determina el algoritmo de planificación.
/// 
/// # Returns
/// 
/// `Ok(SchedulingAlgorithm)` si los argumentos son válidos,
/// `Err(String)` con mensaje de error en caso contrario
fn parse_args() -> Result<SchedulingAlgorithm, String> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        return Err("Se requiere especificar un algoritmo de scheduling".to_string());
    }

    match args[1].as_str() {
        "fcfs" => {
            if args.len() > 2 {
                return Err("FCFS no acepta parámetros adicionales".to_string());
            }
            Ok(SchedulingAlgorithm::fcfs())
        }
        "rr" | "round-robin" => {
            let quantum_ms = if args.len() > 2 {
                args[2].parse::<u64>()
                    .map_err(|_| "El quantum debe ser un número entero positivo".to_string())?
            } else {
                config::DEFAULT_QUANTUM_MS
            };

            if quantum_ms == 0 {
                return Err("El quantum debe ser mayor que 0".to_string());
            }

            Ok(SchedulingAlgorithm::round_robin(std::time::Duration::from_millis(quantum_ms)))
        }
        algorithm => Err(format!("Algoritmo desconocido: '{}'", algorithm)),
    }
}

/// Muestra información de uso del programa.
fn print_usage() {
    println!("Simulador de Línea de Ensamblaje");
    println!();
    println!("USO:");
    println!("    cargo run -- <algoritmo> [parámetros]");
    println!();
    println!("ALGORITMOS:");
    println!("    fcfs                    First-Come First-Served (no preemptivo)");
    println!("    rr [quantum_ms]         Round Robin preemptivo");
    println!("                           quantum_ms: tiempo en milisegundos (default: {})", 
             config::DEFAULT_QUANTUM_MS);
    println!();
    println!("EJEMPLOS:");
    println!("    cargo run -- fcfs");
    println!("    cargo run -- rr");
    println!("    cargo run -- rr 250");
    println!();
    println!("DESCRIPCIÓN:");
    println!("    Simula una línea de ensamblaje con 3 estaciones (Corte, Ensamblaje, Empaque)");
    println!("    procesando 10 productos con diferentes algoritmos de scheduling.");
}
