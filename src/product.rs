//! # Módulo de Productos
//! 
//! Este módulo define la estructura de los productos que atraviesan la línea
//! de ensamblaje y las métricas asociadas a su procesamiento.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::station::{StationConfig, StationState};

/// Representa un producto que atraviesa la línea de ensamblaje.
/// 
/// Cada producto mantiene su identificador único, tiempo de llegada simulado,
/// y un conjunto de métricas por estación que registran su progreso a través
/// de la línea de producción.
/// 
/// El uso de `Arc` permite que múltiples hilos (estaciones) compartan el mismo
/// producto de forma segura, mientras que `Mutex` protege las métricas contra
/// condiciones de carrera.
#[derive(Debug)]
pub struct Product {
    /// Identificador único del producto (1-indexado)
    pub id: usize,
    /// Tiempo de llegada simulado relativo al inicio de la simulación
    pub arrival_offset: Duration,
    /// Momento real en que el producto fue generado en la simulación
    pub arrival_instant: Mutex<Option<Instant>>,
    /// Estado y métricas del producto en cada estación de la línea
    pub stations: Vec<Mutex<StationState>>,
}

impl Product {
    /// Crea un nuevo producto con métricas inicializadas para todas las estaciones.
    /// 
    /// # Arguments
    /// 
    /// * `id` - Identificador único del producto (debe ser único en la simulación)
    /// * `arrival_offset` - Tiempo de llegada simulado relativo al inicio
    /// * `station_configs` - Configuraciones de todas las estaciones en la línea
    /// 
    /// # Returns
    /// 
    /// Un `Arc<Product>` listo para ser compartido entre hilos
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use std::time::Duration;
    /// use assembly_line_simulator::{Product, StationConfig};
    /// 
    /// let configs = vec![
    ///     StationConfig { name: "Corte", processing_time: Duration::from_millis(400) },
    ///     StationConfig { name: "Ensamblaje", processing_time: Duration::from_millis(600) },
    /// ];
    /// 
    /// let product = Product::new(1, Duration::from_millis(100), &configs);
    /// assert_eq!(product.id, 1);
    /// ```
    pub fn new(id: usize, arrival_offset: Duration, station_configs: &[StationConfig]) -> Arc<Self> {
        let stations = station_configs
            .iter()
            .map(|_| Mutex::new(StationState::new()))
            .collect();

        Arc::new(Self {
            id,
            arrival_offset,
            arrival_instant: Mutex::new(None),
            stations,
        })
    }

    /// Obtiene una referencia al estado protegido del producto en una estación específica.
    /// 
    /// # Arguments
    /// 
    /// * `index` - Índice de la estación (0-indexado)
    /// 
    /// # Returns
    /// 
    /// Referencia al `Mutex<StationState>` correspondiente a la estación
    /// 
    /// # Panics
    /// 
    /// Hace panic si el índice está fuera de rango
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # use std::time::Duration;
    /// # use assembly_line_simulator::{Product, StationConfig};
    /// # let configs = vec![StationConfig { name: "Test", processing_time: Duration::from_millis(100) }];
    /// # let product = Product::new(1, Duration::ZERO, &configs);
    /// let station_state = product.station_state(0);
    /// let mut state = station_state.lock().unwrap();
    /// // Modificar el estado...
    /// ```
    pub fn station_state(&self, index: usize) -> &Mutex<StationState> {
        &self.stations[index]
    }

    /// Registra el momento real de llegada del producto a la simulación.
    /// 
    /// Este método debe ser llamado por el generador cuando el producto
    /// es efectivamente introducido en la línea de ensamblaje.
    /// 
    /// # Arguments
    /// 
    /// * `instant` - Momento real en que el producto llegó
    /// 
    /// # Panics
    /// 
    /// Hace panic si no se puede obtener el lock del arrival_instant
    pub fn set_arrival_instant(&self, instant: Instant) {
        *self.arrival_instant.lock()
            .expect("No se pudo obtener lock del arrival_instant") = Some(instant);
    }

    /// Obtiene el momento real de llegada del producto.
    /// 
    /// # Returns
    /// 
    /// `Some(Instant)` si el producto ya fue generado, `None` en caso contrario
    /// 
    /// # Panics
    /// 
    /// Hace panic si no se puede obtener el lock del arrival_instant
    pub fn get_arrival_instant(&self) -> Option<Instant> {
        *self.arrival_instant.lock()
            .expect("No se pudo obtener lock del arrival_instant")
    }

    /// Calcula el tiempo total de espera del producto en todas las estaciones.
    /// 
    /// Suma los tiempos de espera acumulados en cada estación para obtener
    /// el tiempo total que el producto pasó esperando en colas.
    /// 
    /// # Returns
    /// 
    /// Duración total de espera acumulada
    /// 
    /// # Panics
    /// 
    /// Hace panic si no se puede obtener el lock de alguna estación
    pub fn total_wait_time(&self) -> Duration {
        self.stations
            .iter()
            .map(|station| {
                station.lock()
                    .expect("No se pudo obtener lock del estado de estación")
                    .total_wait
            })
            .sum()
    }

    /// Calcula el tiempo de turnaround del producto.
    /// 
    /// El turnaround es la diferencia entre el momento de finalización
    /// completa del producto (salida de la última estación) y su momento
    /// de llegada a la simulación.
    /// 
    /// # Arguments
    /// 
    /// * `start_time` - Momento de inicio de la simulación para cálculos relativos
    /// 
    /// # Returns
    /// 
    /// `Some(Duration)` con el turnaround si el producto fue completado,
    /// `None` si aún está en procesamiento
    /// 
    /// # Panics
    /// 
    /// Hace panic si no se puede obtener los locks necesarios
    pub fn turnaround_time(&self, _start_time: Instant) -> Option<Duration> {
        let arrival = self.get_arrival_instant()?;
        let last_station_index = self.stations.len() - 1;
        
        let final_exit = self.stations[last_station_index]
            .lock()
            .expect("No se pudo obtener lock de la última estación")
            .final_exit?;

        Some(final_exit.duration_since(arrival))
    }

    /// Verifica si el producto ha completado su procesamiento en todas las estaciones.
    /// 
    /// # Returns
    /// 
    /// `true` si el producto terminó de procesarse en todas las estaciones,
    /// `false` en caso contrario
    /// 
    /// # Panics
    /// 
    /// Hace panic si no se puede obtener el lock de alguna estación
    pub fn is_completed(&self) -> bool {
        if self.stations.is_empty() {
            return true;
        }

        let last_station_index = self.stations.len() - 1;
        self.stations[last_station_index]
            .lock()
            .expect("No se pudo obtener lock de la última estación")
            .final_exit
            .is_some()
    }

    /// Obtiene una representación string del estado actual del producto.
    /// 
    /// Útil para debugging y logging del progreso del producto a través
    /// de las estaciones.
    /// 
    /// # Returns
    /// 
    /// String describiendo el estado actual del producto
    pub fn status_summary(&self) -> String {
        let completed_stations = self.stations
            .iter()
            .enumerate()
            .filter(|(_, station)| {
                station.lock()
                    .expect("No se pudo obtener lock de estación")
                    .final_exit
                    .is_some()
            })
            .count();

        format!(
            "Producto {:02}: {}/{} estaciones completadas",
            self.id,
            completed_stations,
            self.stations.len()
        )
    }
}