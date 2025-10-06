//! Tests de integración para el simulador de línea de ensamblaje

use std::time::Duration;
use assembly_line_simulator::{
    Simulation, SchedulingAlgorithm, StationConfig, config
};

#[test]
fn test_fcfs_simulation_completes() {
    let mut simulation = Simulation::new(SchedulingAlgorithm::fcfs());
    let metrics = simulation.run();
    
    // Verificar que todos los productos fueron completados
    assert_eq!(metrics.products.len(), 10);
    assert_eq!(metrics.completion_order.len(), 10);
    
    // En FCFS, el orden debe ser el mismo que el de llegada
    assert_eq!(metrics.completion_order, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
}

#[test]
fn test_round_robin_simulation_completes() {
    let algorithm = SchedulingAlgorithm::round_robin(Duration::from_millis(300));
    let mut simulation = Simulation::new(algorithm);
    let metrics = simulation.run();
    
    // Verificar que todos los productos fueron completados
    assert_eq!(metrics.products.len(), 10);
    assert_eq!(metrics.completion_order.len(), 10);
    
    // Verificar que el throughput es positivo
    assert!(metrics.throughput > 0.0);
}

#[test]
fn test_custom_configuration() {
    let custom_stations = vec![
        StationConfig {
            name: "Test1",
            processing_time: Duration::from_millis(100),
        },
        StationConfig {
            name: "Test2", 
            processing_time: Duration::from_millis(200),
        },
    ];
    
    let custom_arrivals = vec![
        Duration::from_millis(0),
        Duration::from_millis(50),
        Duration::from_millis(100),
    ];
    
    let algorithm = SchedulingAlgorithm::fcfs();
    let mut simulation = Simulation::with_config(custom_stations, algorithm, custom_arrivals);
    let metrics = simulation.run();
    
    // Verificar configuración personalizada
    assert_eq!(metrics.products.len(), 3);
    assert_eq!(metrics.completion_order.len(), 3);
}

#[test]
fn test_fcfs_vs_round_robin_performance() {
    // Simulación FCFS
    let mut fcfs_simulation = Simulation::new(SchedulingAlgorithm::fcfs());
    let fcfs_metrics = fcfs_simulation.run();
    
    // Simulación Round Robin
    let rr_algorithm = SchedulingAlgorithm::round_robin(Duration::from_millis(300));
    let mut rr_simulation = Simulation::new(rr_algorithm);
    let rr_metrics = rr_simulation.run();
    
    // En este caso específico, FCFS debería ser más eficiente
    // (quantum = 300ms vs tiempos de procesamiento de 400-600ms)
    assert!(fcfs_metrics.average_wait_time <= rr_metrics.average_wait_time);
    
    // Ambos deben completar todos los productos
    assert_eq!(fcfs_metrics.products.len(), rr_metrics.products.len());
}

#[test]
fn test_metrics_consistency() {
    let mut simulation = Simulation::new(SchedulingAlgorithm::fcfs());
    let metrics = simulation.run();
    
    // Verificar que las métricas son consistentes
    for product_metrics in &metrics.products {
        // El turnaround debe ser mayor que el tiempo de espera
        assert!(product_metrics.turnaround_time >= product_metrics.total_wait_time);
        
        // Debe haber datos para todas las estaciones
        assert_eq!(product_metrics.station_times.len(), config::STATION_COUNT);
        
        // Los tiempos de salida deben ser mayores que los de entrada
        for (entry, exit) in &product_metrics.station_times {
            assert!(exit >= entry);
        }
    }
    
    // El throughput debe ser realista (mayor que 0, menor que productos/segundo físicamente posibles)
    assert!(metrics.throughput > 0.0);
    assert!(metrics.throughput < 100.0); // Límite superior razonable
}

#[test]
fn test_algorithm_parameter_validation() {
    // Quantum muy pequeño
    let small_quantum = SchedulingAlgorithm::round_robin(Duration::from_millis(1));
    let mut simulation = Simulation::new(small_quantum);
    let metrics = simulation.run();
    assert_eq!(metrics.products.len(), 10);
    
    // Quantum muy grande (efectivamente como FCFS)
    let large_quantum = SchedulingAlgorithm::round_robin(Duration::from_secs(10));
    let mut simulation = Simulation::new(large_quantum);
    let metrics = simulation.run();
    assert_eq!(metrics.products.len(), 10);
}

#[test]
fn test_report_generation() {
    let mut simulation = Simulation::new(SchedulingAlgorithm::fcfs());
    let metrics = simulation.run();
    
    // Generar reporte de texto
    let text_report = simulation.generate_report(&metrics);
    assert!(text_report.contains("REPORTE DE RESULTADOS"));
    assert!(text_report.contains("ESTADÍSTICAS RESUMIDAS"));
    
    // Generar reporte CSV
    let csv_report = simulation.generate_csv_report(&metrics);
    assert!(csv_report.contains("ProductID"));
    assert!(csv_report.contains("ArrivalTime"));
    assert!(csv_report.contains("WaitTime"));
    
    // Verificar que el CSV tiene el número correcto de líneas
    let lines: Vec<&str> = csv_report.lines().collect();
    assert_eq!(lines.len(), 11); // 1 header + 10 products
}