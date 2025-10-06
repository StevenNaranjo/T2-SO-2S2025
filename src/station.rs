//! # Módulo de Estaciones de Trabajo
//! 
//! Este módulo define las estructuras y funciones relacionadas con las estaciones
//! de trabajo en la línea de ensamblaje. Cada estación procesa productos de forma
//! secuencial aplicando algoritmos de planificación.

use std::collections::VecDeque;
use std::sync::mpsc;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::thread;

use crate::product::Product;
use crate::scheduler::SchedulingAlgorithm;
use crate::metrics::MetricsCalculator;

/// Configuración estática para una estación de trabajo.
/// 
/// Define las características inmutables de una estación, como su nombre
/// y el tiempo de procesamiento requerido para cada producto.
#[derive(Clone, Copy, Debug)]
pub struct StationConfig {
    /// Nombre identificador de la estación
    pub name: &'static str,
    /// Tiempo requerido para procesar completamente un producto
    pub processing_time: Duration,
}

/// Estado mutable de un producto dentro de una estación específica.
/// 
/// Almacena las métricas temporales de un producto durante su paso
/// por una estación particular.
#[derive(Debug, Default)]
pub struct StationState {
    /// Momento en que el producto entró por primera vez a la cola de la estación
    pub queue_entry: Option<Instant>,
    /// Momento en que el producto comenzó a ser procesado por primera vez
    pub first_entry: Option<Instant>,
    /// Momento en que el producto completó totalmente el procesamiento en la estación
    pub final_exit: Option<Instant>,
    /// Tiempo total acumulado que el producto esperó en cola en esta estación
    pub total_wait: Duration,
    /// Tiempo de procesamiento restante para completar el producto en esta estación
    pub remaining: Duration,
}

impl StationState {
    /// Crea un nuevo estado inicializado para un producto en una estación.
    /// 
    /// # Returns
    /// 
    /// Una nueva instancia de `StationState` con todos los campos en sus valores por defecto.
    pub fn new() -> Self {
        Self {
            queue_entry: None,
            first_entry: None,
            final_exit: None,
            total_wait: Duration::default(),
            remaining: Duration::default(),
        }
    }
}

/// Representa una estación de trabajo física en la línea de ensamblaje.
/// 
/// Cada estación se ejecuta en su propio hilo y procesa productos de forma
/// secuencial según el algoritmo de planificación configurado.
pub struct Station {
    /// Índice único de la estación en la línea
    pub index: usize,
    /// Configuración estática de la estación
    pub config: StationConfig,
    /// Algoritmo de planificación que utiliza la estación
    pub algorithm: SchedulingAlgorithm,
}

/// Mensajes que se intercambian entre estaciones a través de canales.
/// 
/// Permite la comunicación asíncrona entre las diferentes estaciones
/// de trabajo para transferir productos y coordinar el apagado.
#[derive(Debug)]
pub enum Message {
    /// Contiene un producto que debe ser procesado por la estación receptora
    Product(Arc<Product>),
    /// Señal para que la estación finalice su operación de forma ordenada
    Shutdown,
}

impl Station {
    /// Crea una nueva instancia de estación.
    /// 
    /// # Arguments
    /// 
    /// * `index` - Índice único de la estación en la línea de ensamblaje
    /// * `config` - Configuración estática de la estación
    /// * `algorithm` - Algoritmo de planificación a utilizar
    /// 
    /// # Returns
    /// 
    /// Una nueva instancia de `Station`
    pub fn new(index: usize, config: StationConfig, algorithm: SchedulingAlgorithm) -> Self {
        Self {
            index,
            config,
            algorithm,
        }
    }

    /// Ejecuta el bucle principal de procesamiento de la estación.
    /// 
    /// Esta función representa el ciclo de vida completo de una estación:
    /// 1. Recibe productos desde la estación anterior (o generador)
    /// 2. Los encola internamente según el algoritmo de planificación
    /// 3. Los procesa aplicando el quantum correspondiente
    /// 4. Los envía a la siguiente estación o al colector final
    /// 5. Maneja las señales de apagado de forma ordenada
    /// 
    /// # Arguments
    /// 
    /// * `receiver` - Canal para recibir productos y señales de la estación anterior
    /// * `next_sender` - Canal opcional para enviar productos a la siguiente estación
    /// * `collector` - Canal opcional para enviar productos completados al colector final
    /// 
    /// # Panics
    /// 
    /// La función puede hacer panic si:
    /// - El canal de recepción se cierra inesperadamente
    /// - No se puede enviar un producto o señal a través de los canales de salida
    pub fn run(
        &self,
        receiver: mpsc::Receiver<Message>,
        next_sender: Option<mpsc::Sender<Message>>,
        collector: Option<mpsc::Sender<Arc<Product>>>,
    ) {
        let mut queue: VecDeque<Arc<Product>> = VecDeque::new();
        let mut shutdown_received = false;

        println!("[INFO] Estación '{}' iniciada", self.config.name);

        loop {
            // Si la cola está vacía, esperamos por mensajes
            if queue.is_empty() {
                if shutdown_received {
                    // Si ya recibimos la señal de apagado y no hay productos en cola,
                    // reenviamos la señal y terminamos
                    if let Some(sender) = &next_sender {
                        sender
                            .send(Message::Shutdown)
                            .expect("No se pudo reenviar señal de apagado");
                    }
                    println!("[INFO] Estación '{}' finalizando", self.config.name);
                    break;
                }

                // Esperamos por el próximo mensaje (bloqueo)
                match receiver.recv().expect("Canal de recepción cerrado inesperadamente") {
                    Message::Product(product) => {
                        self.register_arrival(&product);
                        queue.push_back(product);
                    }
                    Message::Shutdown => {
                        shutdown_received = true;
                    }
                }
                continue;
            }

            // Procesamos mensajes adicionales sin bloquear
            while let Ok(message) = receiver.try_recv() {
                match message {
                    Message::Product(product) => {
                        self.register_arrival(&product);
                        queue.push_back(product);
                    }
                    Message::Shutdown => {
                        shutdown_received = true;
                    }
                }
            }

            // Procesamos el próximo producto en la cola
            if let Some(product) = queue.pop_front() {
                self.process_product(product, &mut queue, &next_sender, &collector);
            }
        }
    }

    /// Registra la llegada de un producto a la estación.
    /// 
    /// Actualiza las métricas del producto para reflejar su entrada a la cola
    /// de esta estación. Inicializa el tiempo restante de procesamiento si
    /// es la primera vez que el producto llega a esta estación.
    /// 
    /// # Arguments
    /// 
    /// * `product` - Referencia al producto que llega a la estación
    fn register_arrival(&self, product: &Arc<Product>) {
        let now = Instant::now();
        let mut station_state = product.station_state(self.index).lock()
            .expect("No se pudo obtener el lock del estado de la estación");
        
        station_state.queue_entry = Some(now);
        
        // Inicializar tiempo restante si es la primera vez que llega
        if station_state.remaining.is_zero() {
            station_state.remaining = self.config.processing_time;
        }

        println!(
            "[{}] Producto {:02} agregado a la cola (restante: {})",
            self.config.name,
            product.id,
            format_duration(station_state.remaining)
        );
    }

    /// Procesa un producto aplicando el algoritmo de planificación configurado.
    /// 
    /// Esta función implementa la lógica central del procesamiento:
    /// 1. Calcula el tiempo de espera acumulado
    /// 2. Determina el quantum de procesamiento según el algoritmo
    /// 3. Simula el procesamiento mediante `thread::sleep`
    /// 4. Actualiza las métricas del producto
    /// 5. Decide si enviar el producto a la siguiente estación o reencolarlo
    /// 
    /// # Arguments
    /// 
    /// * `product` - Producto a procesar
    /// * `queue` - Cola de productos de la estación (para reencolar si es necesario)
    /// * `next_sender` - Canal opcional para enviar a la siguiente estación
    /// * `collector` - Canal opcional para enviar al colector final
    fn process_product(
        &self,
        product: Arc<Product>,
        queue: &mut VecDeque<Arc<Product>>,
        next_sender: &Option<mpsc::Sender<Message>>,
        collector: &Option<mpsc::Sender<Arc<Product>>>,
    ) {
        let now = Instant::now();
        
        // Obtener y actualizar el estado del producto en esta estación
        let remaining = {
            let mut station_state = product.station_state(self.index).lock()
                .expect("No se pudo obtener el lock del estado de la estación");
            
            let queue_entry = station_state
                .queue_entry
                .take()
                .expect("Se esperaba tiempo de entrada a la cola");
            
            // Acumular tiempo de espera
            station_state.total_wait += now - queue_entry;
            
            // Registrar primera entrada si es necesario
            if station_state.first_entry.is_none() {
                station_state.first_entry = Some(now);
            }
            
            station_state.remaining
        };

        // Determinar quantum de procesamiento según el algoritmo
        let slice = match &self.algorithm {
            SchedulingAlgorithm::Fcfs => remaining,
            SchedulingAlgorithm::RoundRobin { quantum } => remaining.min(*quantum),
        };

        println!(
            "[{}] Producto {:02} inicia procesamiento por {} (restante: {})",
            self.config.name,
            product.id,
            format_duration(slice),
            format_duration(remaining)
        );

        // Simular el procesamiento
        thread::sleep(slice);
        let completed_at = Instant::now();

        // Actualizar estado después del procesamiento
        let mut station_state = product.station_state(self.index).lock()
            .expect("No se pudo obtener el lock del estado de la estación");

        if slice >= remaining {
            // Producto completado en esta estación
            station_state.remaining = Duration::ZERO;
            station_state.final_exit = Some(completed_at);
            drop(station_state);

            println!(
                "[{}] Producto {:02} completado en la estación",
                self.config.name,
                product.id
            );

            // Enviar a la siguiente estación o al colector
            if let Some(sender) = next_sender {
                sender
                    .send(Message::Product(product))
                    .expect("No se pudo enviar producto a la siguiente estación");
            } else if let Some(collector) = collector {
                collector
                    .send(product)
                    .expect("No se pudo enviar producto al colector");
            }
        } else {
            // Producto interrumpido, vuelve a la cola
            station_state.remaining = remaining - slice;
            station_state.queue_entry = Some(completed_at);
            let remaining_after = station_state.remaining;
            drop(station_state);

            println!(
                "[{}] Producto {:02} interrumpido, vuelve a la cola (restante: {})",
                self.config.name,
                product.id,
                format_duration(remaining_after)
            );

            queue.push_back(product);
        }
    }
}

/// Formatea una duración para mostrar en formato legible.
/// 
/// Convierte una `Duration` a una representación de string en formato
/// "segundos.milisegundos" (ej: "1.234s").
/// 
/// # Arguments
/// 
/// * `duration` - La duración a formatear
/// 
/// # Returns
/// 
/// String formateado con la duración
fn format_duration(duration: Duration) -> String {
    MetricsCalculator::format_duration(duration)
}