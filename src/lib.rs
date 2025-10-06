//! # Simulador de Línea de Ensamblaje
//! 
//! Esta biblioteca implementa un simulador de línea de ensamblaje industrial que utiliza
//! algoritmos de planificación de procesos (FCFS y Round Robin) para gestionar productos
//! que atraviesan múltiples estaciones de trabajo.
//! 
//! ## Características principales
//! 
//! - **Comunicación interprocesos**: Utiliza canales (`std::sync::mpsc`) para la comunicación
//!   entre hilos que representan diferentes estaciones de trabajo.
//! - **Sincronización**: Emplea `Arc<Mutex<T>>` para compartir datos de forma segura entre hilos.
//! - **Algoritmos de scheduling**: Implementa FCFS (First-Come First-Served) y Round Robin
//!   con quantum configurable.
//! - **Métricas detalladas**: Registra tiempos de llegada, procesamiento, espera y turnaround
//!   para análisis de rendimiento.
//! 
//! ## Estructura del proyecto
//! 
//! - `station`: Módulo que define las estaciones de trabajo y su configuración
//! - `product`: Módulo que define los productos y sus métricas asociadas
//! - `scheduler`: Módulo que implementa los algoritmos de planificación
//! - `simulation`: Módulo principal que coordina la simulación
//! - `metrics`: Módulo para el cálculo y reporte de métricas

pub mod station;
pub mod product;
pub mod scheduler;
pub mod simulation;
pub mod metrics;

// Re-exportar las estructuras principales para facilitar su uso
pub use station::{Station, StationConfig, StationState};
pub use product::Product;
pub use scheduler::SchedulingAlgorithm;
pub use simulation::Simulation;
pub use metrics::MetricsCalculator;

/// Configuración por defecto del simulador
pub mod config {
    use std::time::Duration;
    
    /// Número de estaciones en la línea de ensamblaje
    pub const STATION_COUNT: usize = 3;
    
    /// Quantum por defecto para Round Robin (en milisegundos)
    pub const DEFAULT_QUANTUM_MS: u64 = 300;
    
    /// Configuración de las estaciones de trabajo
    pub fn default_station_configs() -> Vec<super::StationConfig> {
        vec![
            super::StationConfig {
                name: "Corte",
                processing_time: Duration::from_millis(400),
            },
            super::StationConfig {
                name: "Ensamblaje", 
                processing_time: Duration::from_millis(600),
            },
            super::StationConfig {
                name: "Empaque",
                processing_time: Duration::from_millis(500),
            },
        ]
    }
    
    /// Tiempos de llegada por defecto para los productos (en milisegundos)
    pub fn default_arrival_times() -> Vec<Duration> {
        vec![0u64, 120, 260, 380, 540, 720, 900, 1100, 1300, 1500]
            .into_iter()
            .map(Duration::from_millis)
            .collect()
    }
}