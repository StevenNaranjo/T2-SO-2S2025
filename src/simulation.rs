//! # Módulo de Simulación Principal
//! 
//! Este módulo contiene la lógica principal para ejecutar la simulación
//! de línea de ensamblaje, incluyendo la generación de productos, 
//! coordinación de estaciones y recolección de resultados.

use std::sync::{Arc, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use crate::config;
use crate::metrics::{MetricsCalculator, SimulationMetrics};
use crate::product::Product;
use crate::scheduler::SchedulingAlgorithm;
use crate::station::{Message, Station, StationConfig};

/// Orquestador principal de la simulación de línea de ensamblaje.
/// 
/// La `Simulation` coordina todos los aspectos de la simulación:
/// - Configuración de estaciones y algoritmos de scheduling
/// - Generación y distribución de productos
/// - Sincronización entre hilos
/// - Recolección y cálculo de métricas
pub struct Simulation {
    /// Configuraciones de todas las estaciones en la línea
    station_configs: Vec<StationConfig>,
    /// Algoritmo de scheduling utilizado por todas las estaciones
    algorithm: SchedulingAlgorithm,
    /// Tiempos de llegada de los productos
    arrival_times: Vec<Duration>,
    /// Calculadora de métricas para generar reportes
    metrics_calculator: MetricsCalculator,
}

impl Simulation {
    /// Crea una nueva simulación con configuración por defecto.
    /// 
    /// # Arguments
    /// 
    /// * `algorithm` - Algoritmo de scheduling a utilizar
    /// 
    /// # Returns
    /// 
    /// Nueva instancia de `Simulation` con configuración estándar
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use assembly_line_simulator::{Simulation, SchedulingAlgorithm};
    /// 
    /// let simulation = Simulation::new(SchedulingAlgorithm::fcfs());
    /// ```
    pub fn new(algorithm: SchedulingAlgorithm) -> Self {
        Self {
            station_configs: config::default_station_configs(),
            algorithm,
            arrival_times: config::default_arrival_times(),
            metrics_calculator: MetricsCalculator::new(),
        }
    }

    /// Crea una simulación con configuración personalizada.
    /// 
    /// # Arguments
    /// 
    /// * `station_configs` - Configuraciones personalizadas de estaciones
    /// * `algorithm` - Algoritmo de scheduling a utilizar
    /// * `arrival_times` - Tiempos de llegada personalizados
    /// 
    /// # Returns
    /// 
    /// Nueva instancia de `Simulation` con configuración personalizada
    pub fn with_config(
        station_configs: Vec<StationConfig>,
        algorithm: SchedulingAlgorithm,
        arrival_times: Vec<Duration>,
    ) -> Self {
        Self {
            station_configs,
            algorithm,
            arrival_times,
            metrics_calculator: MetricsCalculator::new(),
        }
    }

    /// Ejecuta la simulación completa y retorna las métricas resultantes.
    /// 
    /// Este método implementa el ciclo completo de la simulación:
    /// 1. Inicializa productos y canales de comunicación
    /// 2. Lanza hilos para cada estación de trabajo
    /// 3. Lanza el generador de productos
    /// 4. Recolecta productos completados
    /// 5. Calcula y retorna métricas finales
    /// 
    /// # Returns
    /// 
    /// `SimulationMetrics` con todos los resultados y estadísticas
    /// 
    /// # Panics
    /// 
    /// Puede hacer panic si:
    /// - No se pueden crear los canales de comunicación
    /// - Los hilos de estaciones fallan durante la ejecución
    /// - El generador de productos falla
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use assembly_line_simulator::{Simulation, SchedulingAlgorithm};
    /// 
    /// let mut simulation = Simulation::new(SchedulingAlgorithm::fcfs());
    /// let metrics = simulation.run();
    /// println!("Tiempo promedio de espera: {:?}", metrics.average_wait_time);
    /// ```
    pub fn run(&mut self) -> SimulationMetrics {
        println!(
            "=== Simulación de línea de ensamblaje ({}) ===",
            self.algorithm
        );
        println!("Configuración:");
        for (i, config) in self.station_configs.iter().enumerate() {
            println!("  Estación {}: {} ({}ms)", 
                i + 1, 
                config.name, 
                config.processing_time.as_millis()
            );
        }
        println!("Productos a procesar: {}", self.arrival_times.len());
        println!();

        let start_time = Instant::now();
        
        // Crear productos
        let products = self.create_products();
        let total_products = products.len();

        // Configurar canales de comunicación
        let (channels, collector_rx) = self.setup_channels();
        
        // Lanzar estaciones de trabajo
        let station_handles = self.launch_stations(channels);
        
        // Lanzar generador de productos
        let first_sender = station_handles.first()
            .expect("Debe haber al menos una estación")
            .sender.clone();
        
        let generator_handle = self.launch_generator(
            first_sender,
            products.clone(),
            start_time,
        );

        // Recolectar productos completados
        let completion_order = self.collect_completed_products(collector_rx, total_products);
        
        let end_time = Instant::now();

        // Esperar a que terminen todos los hilos
        generator_handle.join()
            .expect("El generador falló");
        
        for handle_info in station_handles {
            handle_info.handle.join()
                .expect("Una estación falló");
        }

        // Calcular y retornar métricas
        let metrics = self.metrics_calculator.calculate_simulation_metrics(
            &products,
            &self.station_configs,
            start_time,
            end_time,
            completion_order,
        );

        println!("\n=== Simulación completada ===");
        println!("Duración total: {}", 
            MetricsCalculator::format_duration(end_time.duration_since(start_time)));

        metrics
    }

    /// Genera un reporte detallado de los resultados.
    /// 
    /// # Arguments
    /// 
    /// * `metrics` - Métricas de la simulación
    /// 
    /// # Returns
    /// 
    /// String con el reporte formateado
    pub fn generate_report(&self, metrics: &SimulationMetrics) -> String {
        self.metrics_calculator.generate_report(metrics, &self.station_configs)
    }

    /// Genera un reporte en formato CSV.
    /// 
    /// # Arguments
    /// 
    /// * `metrics` - Métricas de la simulación
    /// 
    /// # Returns
    /// 
    /// String con los datos en formato CSV
    pub fn generate_csv_report(&self, metrics: &SimulationMetrics) -> String {
        self.metrics_calculator.generate_csv_report(metrics)
    }

    /// Crea todos los productos para la simulación.
    fn create_products(&self) -> Vec<Arc<Product>> {
        self.arrival_times
            .iter()
            .enumerate()
            .map(|(idx, &offset)| {
                Product::new(idx + 1, offset, &self.station_configs)
            })
            .collect()
    }

    /// Configura los canales de comunicación entre estaciones.
    fn setup_channels(&self) -> (Vec<ChannelPair>, mpsc::Receiver<Arc<Product>>) {
        let (collector_tx, collector_rx) = mpsc::channel::<Arc<Product>>();
        
        let mut channels = Vec::new();
        for i in 0..self.station_configs.len() {
            let (tx, rx) = mpsc::channel::<Message>();
            let next_sender = if i + 1 < self.station_configs.len() {
                None // Se configurará después
            } else {
                None
            };
            let collector = if i + 1 == self.station_configs.len() {
                Some(collector_tx.clone())
            } else {
                None
            };
            
            channels.push(ChannelPair {
                sender: tx,
                receiver: rx,
                next_sender,
                collector,
            });
        }

        // Configurar next_sender para cada canal
        for i in 0..(channels.len() - 1) {
            channels[i].next_sender = Some(channels[i + 1].sender.clone());
        }

        (channels, collector_rx)
    }

    /// Lanza todos los hilos de las estaciones de trabajo.
    fn launch_stations(&self, channels: Vec<ChannelPair>) -> Vec<StationHandle> {
        let mut handles = Vec::new();
        
        for (index, (config, channel)) in self.station_configs.iter().zip(channels).enumerate() {
            let station = Station::new(index, *config, self.algorithm.clone());
            
            let handle = thread::spawn(move || {
                station.run(
                    channel.receiver,
                    channel.next_sender,
                    channel.collector,
                );
            });
            
            handles.push(StationHandle {
                handle,
                sender: channel.sender,
            });
        }
        
        handles
    }

    /// Lanza el generador de productos.
    fn launch_generator(
        &self,
        sender: mpsc::Sender<Message>,
        products: Vec<Arc<Product>>,
        start_time: Instant,
    ) -> thread::JoinHandle<()> {
        let first_station_config = self.station_configs[0];
        
        thread::spawn(move || {
            ProductGenerator::run(sender, products, first_station_config, start_time);
        })
    }

    /// Recolecta los productos completados en orden de finalización.
    fn collect_completed_products(
        &self,
        collector_rx: mpsc::Receiver<Arc<Product>>,
        total_products: usize,
    ) -> Vec<usize> {
        let mut completion_order = Vec::new();
        
        for _ in 0..total_products {
            if let Ok(product) = collector_rx.recv() {
                completion_order.push(product.id);
                println!("[COMPLETADO] Producto {:02} terminó toda la línea", product.id);
            }
        }
        
        completion_order
    }
}

/// Generador de productos que respeta los tiempos de llegada simulados.
struct ProductGenerator;

impl ProductGenerator {
    /// Ejecuta el ciclo de generación de productos.
    /// 
    /// Genera productos respetando sus tiempos de llegada simulados,
    /// los introduce en la primera estación y envía la señal de apagado
    /// al final.
    fn run(
        sender: mpsc::Sender<Message>,
        products: Vec<Arc<Product>>,
        first_station: StationConfig,
        start_time: Instant,
    ) {
        println!("[GENERADOR] Iniciando generación de {} productos", products.len());
        
        for product in products {
            // Esperar hasta el momento de llegada simulado
            let target_time = start_time + product.arrival_offset;
            let now = Instant::now();
            
            if now < target_time {
                let wait_time = target_time - now;
                thread::sleep(wait_time);
            }

            // Registrar llegada real
            let arrival_instant = Instant::now();
            product.set_arrival_instant(arrival_instant);

            // Inicializar estado en la primera estación
            {
                let mut station_state = product.station_state(0).lock()
                    .expect("No se pudo obtener lock del estado de la primera estación");
                station_state.queue_entry = Some(arrival_instant);
                if station_state.remaining.is_zero() {
                    station_state.remaining = first_station.processing_time;
                }
            }

            println!(
                "[GENERADOR] Producto {:02} disponible en t={}",
                product.id,
                MetricsCalculator::format_duration(arrival_instant.duration_since(start_time))
            );

            // Enviar producto a la primera estación
            sender
                .send(Message::Product(product))
                .expect("No se pudo enviar producto a la primera estación");
        }

        // Enviar señal de apagado
        sender
            .send(Message::Shutdown)
            .expect("No se pudo enviar señal de apagado");
        
        println!("[GENERADOR] Generación completada, señal de apagado enviada");
    }
}

/// Información de canales para una estación.
struct ChannelPair {
    sender: mpsc::Sender<Message>,
    receiver: mpsc::Receiver<Message>,
    next_sender: Option<mpsc::Sender<Message>>,
    collector: Option<mpsc::Sender<Arc<Product>>>,
}

/// Handle para controlar una estación.
struct StationHandle {
    handle: thread::JoinHandle<()>,
    sender: mpsc::Sender<Message>,
}