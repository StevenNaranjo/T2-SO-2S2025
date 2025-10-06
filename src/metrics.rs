//! # Módulo de Métricas y Reportes
//! 
//! Este módulo se encarga de calcular, almacenar y generar reportes de
//! las métricas de rendimiento de la simulación de línea de ensamblaje.

use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::product::Product;
use crate::station::StationConfig;

/// Métricas individuales de un producto en la simulación.
/// 
/// Almacena todas las métricas relevantes para el análisis de rendimiento
/// de un producto específico durante su paso por la línea de ensamblaje.
#[derive(Debug, Clone)]
pub struct ProductMetrics {
    /// ID del producto
    pub product_id: usize,
    /// Tiempo de llegada simulado relativo al inicio
    pub arrival_time: Duration,
    /// Tiempo total de espera en todas las estaciones
    pub total_wait_time: Duration,
    /// Tiempo de turnaround (desde llegada hasta finalización)
    pub turnaround_time: Duration,
    /// Tiempos de entrada y salida por estación
    pub station_times: Vec<(Duration, Duration)>, // (entrada, salida) relativas al inicio
}

/// Métricas agregadas de toda la simulación.
/// 
/// Contiene estadísticas resumidas del rendimiento general de la
/// línea de ensamblaje durante la simulación completa.
#[derive(Debug, Clone)]
pub struct SimulationMetrics {
    /// Métricas individuales de cada producto
    pub products: Vec<ProductMetrics>,
    /// Tiempo promedio de espera
    pub average_wait_time: Duration,
    /// Tiempo promedio de turnaround
    pub average_turnaround_time: Duration,
    /// Orden de finalización de los productos
    pub completion_order: Vec<usize>,
    /// Duración total de la simulación
    pub total_simulation_time: Duration,
    /// Throughput (productos por segundo)
    pub throughput: f64,
}

/// Calculadora de métricas para la simulación.
/// 
/// Proporciona métodos para calcular métricas individuales y agregadas,
/// así como para generar reportes formateados de los resultados.
pub struct MetricsCalculator;

impl MetricsCalculator {
    /// Crea una nueva instancia del calculador de métricas.
    pub fn new() -> Self {
        Self
    }

    /// Calcula las métricas para un producto individual.
    /// 
    /// # Arguments
    /// 
    /// * `product` - Referencia al producto
    /// * `station_configs` - Configuraciones de las estaciones
    /// * `start_time` - Momento de inicio de la simulación
    /// 
    /// # Returns
    /// 
    /// `ProductMetrics` con todas las métricas calculadas, o `None` si
    /// el producto no ha sido completado
    pub fn calculate_product_metrics(
        &self,
        product: &Arc<Product>,
        station_configs: &[StationConfig],
        start_time: Instant,
    ) -> Option<ProductMetrics> {
        // Verificar que el producto esté completado
        if !product.is_completed() {
            return None;
        }

        let arrival_instant = product.get_arrival_instant()?;
        let arrival_time = arrival_instant.duration_since(start_time);
        let total_wait_time = product.total_wait_time();
        let turnaround_time = product.turnaround_time(start_time)?;

        // Calcular tiempos por estación
        let mut station_times = Vec::new();
        for (index, _config) in station_configs.iter().enumerate() {
            let state = product.station_state(index).lock()
                .expect("No se pudo obtener lock del estado de estación");
            
            let entry_time = state.first_entry
                .map(|t| t.duration_since(start_time))
                .unwrap_or_default();
            
            let exit_time = state.final_exit
                .map(|t| t.duration_since(start_time))
                .unwrap_or_default();
            
            station_times.push((entry_time, exit_time));
        }

        Some(ProductMetrics {
            product_id: product.id,
            arrival_time,
            total_wait_time,
            turnaround_time,
            station_times,
        })
    }

    /// Calcula las métricas agregadas de toda la simulación.
    /// 
    /// # Arguments
    /// 
    /// * `products` - Vector de productos completados
    /// * `station_configs` - Configuraciones de las estaciones
    /// * `start_time` - Momento de inicio de la simulación
    /// * `end_time` - Momento de finalización de la simulación
    /// * `completion_order` - Orden en que se completaron los productos
    /// 
    /// # Returns
    /// 
    /// `SimulationMetrics` con todas las estadísticas agregadas
    pub fn calculate_simulation_metrics(
        &self,
        products: &[Arc<Product>],
        station_configs: &[StationConfig],
        start_time: Instant,
        end_time: Instant,
        completion_order: Vec<usize>,
    ) -> SimulationMetrics {
        let mut product_metrics = Vec::new();
        let mut total_wait = Duration::ZERO;
        let mut total_turnaround = Duration::ZERO;
        let mut completed_count = 0;

        // Calcular métricas individuales
        for product in products {
            if let Some(metrics) = self.calculate_product_metrics(product, station_configs, start_time) {
                total_wait += metrics.total_wait_time;
                total_turnaround += metrics.turnaround_time;
                completed_count += 1;
                product_metrics.push(metrics);
            }
        }

        // Calcular promedios
        let average_wait_time = if completed_count > 0 {
            total_wait / completed_count as u32
        } else {
            Duration::ZERO
        };

        let average_turnaround_time = if completed_count > 0 {
            total_turnaround / completed_count as u32
        } else {
            Duration::ZERO
        };

        // Calcular throughput
        let total_simulation_time = end_time.duration_since(start_time);
        let throughput = if total_simulation_time.as_secs_f64() > 0.0 {
            completed_count as f64 / total_simulation_time.as_secs_f64()
        } else {
            0.0
        };

        SimulationMetrics {
            products: product_metrics,
            average_wait_time,
            average_turnaround_time,
            completion_order,
            total_simulation_time,
            throughput,
        }
    }

    /// Genera un reporte detallado de los resultados de la simulación.
    /// 
    /// # Arguments
    /// 
    /// * `metrics` - Métricas de la simulación
    /// * `station_configs` - Configuraciones de las estaciones
    /// 
    /// # Returns
    /// 
    /// String con el reporte formateado
    pub fn generate_report(
        &self,
        metrics: &SimulationMetrics,
        station_configs: &[StationConfig],
    ) -> String {
        let mut report = String::new();
        
        report.push_str("\n=== REPORTE DE RESULTADOS ===\n\n");

        // Encabezado de la tabla
        report.push_str(&format!(
            "{:^8} {:^12} {:^15} {:^15} {:^15} {:^12} {:^15}\n",
            "Prod", "Llegada", 
            station_configs.get(0).map(|c| c.name).unwrap_or("Est1"),
            station_configs.get(1).map(|c| c.name).unwrap_or("Est2"),
            station_configs.get(2).map(|c| c.name).unwrap_or("Est3"),
            "Espera", "Turnaround"
        ));

        report.push_str(&format!("{}\n", "-".repeat(100)));

        // Datos de cada producto
        for product_metrics in &metrics.products {
            let mut station_ranges = Vec::new();
            for (entry, exit) in &product_metrics.station_times {
                station_ranges.push(format!(
                    "{}-{}",
                    Self::format_duration(*entry),
                    Self::format_duration(*exit)
                ));
            }

            // Asegurar que tenemos exactamente 3 estaciones para el formato
            while station_ranges.len() < 3 {
                station_ranges.push("N/A".to_string());
            }

            report.push_str(&format!(
                "{:^8} {:^12} {:^15} {:^15} {:^15} {:^12} {:^15}\n",
                format!("#{:02}", product_metrics.product_id),
                Self::format_duration(product_metrics.arrival_time),
                station_ranges[0],
                station_ranges.get(1).unwrap_or(&"N/A".to_string()),
                station_ranges.get(2).unwrap_or(&"N/A".to_string()),
                Self::format_duration(product_metrics.total_wait_time),
                Self::format_duration(product_metrics.turnaround_time),
            ));
        }

        // Estadísticas resumidas
        report.push_str("\n=== ESTADÍSTICAS RESUMIDAS ===\n");
        report.push_str(&format!(
            "Productos completados: {}\n",
            metrics.products.len()
        ));
        report.push_str(&format!(
            "Tiempo promedio de espera: {}\n",
            Self::format_duration(metrics.average_wait_time)
        ));
        report.push_str(&format!(
            "Tiempo promedio de turnaround: {}\n",
            Self::format_duration(metrics.average_turnaround_time)
        ));
        report.push_str(&format!(
            "Duración total de simulación: {}\n",
            Self::format_duration(metrics.total_simulation_time)
        ));
        report.push_str(&format!(
            "Throughput: {:.3} productos/segundo\n",
            metrics.throughput
        ));
        report.push_str(&format!(
            "Orden de finalización: {:?}\n",
            metrics.completion_order
        ));

        report
    }

    /// Genera un reporte resumido en formato CSV.
    /// 
    /// # Arguments
    /// 
    /// * `metrics` - Métricas de la simulación
    /// 
    /// # Returns
    /// 
    /// String con los datos en formato CSV
    pub fn generate_csv_report(&self, metrics: &SimulationMetrics) -> String {
        let mut csv = String::new();
        
        // Encabezado CSV
        csv.push_str("ProductID,ArrivalTime,WaitTime,Turnaround,Station1_Entry,Station1_Exit,Station2_Entry,Station2_Exit,Station3_Entry,Station3_Exit\n");
        
        // Datos de cada producto
        for product_metrics in &metrics.products {
            csv.push_str(&format!("{},", product_metrics.product_id));
            csv.push_str(&format!("{:.3},", product_metrics.arrival_time.as_secs_f64()));
            csv.push_str(&format!("{:.3},", product_metrics.total_wait_time.as_secs_f64()));
            csv.push_str(&format!("{:.3},", product_metrics.turnaround_time.as_secs_f64()));
            
            // Tiempos por estación
            for (entry, exit) in &product_metrics.station_times {
                csv.push_str(&format!("{:.3},{:.3},", entry.as_secs_f64(), exit.as_secs_f64()));
            }
            
            // Completar con N/A si faltan estaciones
            for _ in product_metrics.station_times.len()..3 {
                csv.push_str("N/A,N/A,");
            }
            
            csv.push('\n');
        }
        
        csv
    }

    /// Formatea una duración para mostrar en formato legible.
    /// 
    /// # Arguments
    /// 
    /// * `duration` - La duración a formatear
    /// 
    /// # Returns
    /// 
    /// String formateado con la duración en formato "s.mmm"
    pub fn format_duration(duration: Duration) -> String {
        let millis = duration.as_millis();
        let seconds = millis / 1000;
        let milliseconds = millis % 1000;
        format!("{}.{:03}s", seconds, milliseconds)
    }
}

impl Default for MetricsCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::station::StationConfig;
    use std::sync::Mutex;

    #[test]
    fn test_format_duration() {
        assert_eq!(
            MetricsCalculator::format_duration(Duration::from_millis(1500)),
            "1.500s"
        );
        assert_eq!(
            MetricsCalculator::format_duration(Duration::from_millis(250)),
            "0.250s"
        );
        assert_eq!(
            MetricsCalculator::format_duration(Duration::ZERO),
            "0.000s"
        );
    }

    #[test]
    fn test_metrics_calculator_creation() {
        let calculator = MetricsCalculator::new();
        let default_calculator = MetricsCalculator::default();
        
        // Simplemente verificar que se pueden crear
        drop(calculator);
        drop(default_calculator);
    }
}