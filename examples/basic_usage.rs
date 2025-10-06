//! Ejemplo básico de uso del simulador de línea de ensamblaje

use assembly_line_simulator::{Simulation, SchedulingAlgorithm};

fn main() {
    println!("=== Ejemplo: Uso Básico del Simulador ===\n");

    // Ejecutar simulación con FCFS
    println!("1. Ejecutando simulación con FCFS...");
    let mut fcfs_simulation = Simulation::new(SchedulingAlgorithm::fcfs());
    let fcfs_metrics = fcfs_simulation.run();
    
    println!("\n--- Reporte FCFS ---");
    let fcfs_report = fcfs_simulation.generate_report(&fcfs_metrics);
    println!("{}", fcfs_report);

    // Ejecutar simulación con Round Robin
    println!("\n2. Ejecutando simulación con Round Robin (300ms)...");
    let rr_algorithm = SchedulingAlgorithm::round_robin(std::time::Duration::from_millis(300));
    let mut rr_simulation = Simulation::new(rr_algorithm);
    let rr_metrics = rr_simulation.run();
    
    println!("\n--- Reporte Round Robin ---");
    let rr_report = rr_simulation.generate_report(&rr_metrics);
    println!("{}", rr_report);

    // Comparación de resultados
    println!("\n=== Comparación de Algoritmos ===");
    println!("| Métrica                    | FCFS      | Round Robin | Diferencia |");
    println!("|-----------------------------|-----------|-------------|------------|");
    
    let wait_diff = ((rr_metrics.average_wait_time.as_millis() as f64 / fcfs_metrics.average_wait_time.as_millis() as f64) - 1.0) * 100.0;
    let turnaround_diff = ((rr_metrics.average_turnaround_time.as_millis() as f64 / fcfs_metrics.average_turnaround_time.as_millis() as f64) - 1.0) * 100.0;
    let throughput_diff = ((rr_metrics.throughput / fcfs_metrics.throughput) - 1.0) * 100.0;
    
    println!("| Tiempo promedio de espera  | {:.3}s    | {:.3}s      | {:+.1}%     |", 
             fcfs_metrics.average_wait_time.as_secs_f64(),
             rr_metrics.average_wait_time.as_secs_f64(),
             wait_diff);
    
    println!("| Tiempo promedio turnaround | {:.3}s    | {:.3}s      | {:+.1}%     |",
             fcfs_metrics.average_turnaround_time.as_secs_f64(),
             rr_metrics.average_turnaround_time.as_secs_f64(),
             turnaround_diff);
    
    println!("| Throughput                 | {:.3}/s   | {:.3}/s     | {:+.1}%     |",
             fcfs_metrics.throughput,
             rr_metrics.throughput,
             throughput_diff);

    // Generar archivos CSV para análisis posterior
    let fcfs_csv = fcfs_simulation.generate_csv_report(&fcfs_metrics);
    let rr_csv = rr_simulation.generate_csv_report(&rr_metrics);
    
    std::fs::write("fcfs_results.csv", fcfs_csv)
        .expect("No se pudo escribir archivo FCFS CSV");
    std::fs::write("rr_results.csv", rr_csv)
        .expect("No se pudo escribir archivo RR CSV");
    
    println!("\n📁 Archivos CSV generados:");
    println!("   - fcfs_results.csv");
    println!("   - rr_results.csv");
    
    println!("\n✅ Ejemplo completado exitosamente!");
}