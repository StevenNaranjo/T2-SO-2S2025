use std::collections::VecDeque;
use std::fmt;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const STATION_COUNT: usize = 3;
const DEFAULT_QUANTUM_MS: u64 = 300;
#[derive(Clone)]
enum SchedulingAlgorithm {
    Fcfs,
    RoundRobin { quantum: Duration },
}

/// Configuración estática para cada estación de la línea de ensamblaje.
#[derive(Clone, Copy)]
struct StationConfig {
    name: &'static str,
    processing_time: Duration,
}

#[derive(Default)]
/// Estado mutable asociado a un producto dentro de una estación concreta.
struct StationState {
    queue_entry: Option<Instant>,
    first_entry: Option<Instant>,
    final_exit: Option<Instant>,
    total_wait: Duration,
    remaining: Duration,
}

impl StationState {
    /// Crea un estado inicializado sin tiempos registrados.
    fn new() -> Self {
        Self {
            queue_entry: None,
            first_entry: None,
            final_exit: None,
            total_wait: Duration::default(),
            remaining: Duration::default(),
        }
    }
}

/// Producto que atraviesa la línea de ensamblaje.
struct Product {
    id: usize,
    arrival_offset: Duration,
    arrival_instant: Mutex<Option<Instant>>,
    stations: Vec<Mutex<StationState>>,
}

impl Product {
    /// Construye un nuevo producto y reserva el espacio para las métricas por estación.
    fn new(id: usize, arrival_offset: Duration, station_configs: &[StationConfig]) -> Arc<Self> {
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

    /// Devuelve el estado protegido para la estación indicada.
    fn station_state(&self, index: usize) -> &Mutex<StationState> {
        &self.stations[index]
    }
}

/// Mensaje intercambiado entre estaciones a través de los canales.
enum Message {
    Product(Arc<Product>),
    Shutdown,
}

fn main() {
    let algorithm = parse_args();
    run_simulation(algorithm);
}

/// Procesa los argumentos de línea de comandos y determina el algoritmo de planificación.
fn parse_args() -> SchedulingAlgorithm {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("fcfs") => SchedulingAlgorithm::Fcfs,
        Some("rr") | Some("round-robin") => {
            let quantum = args
                .next()
                .and_then(|arg| arg.parse::<u64>().ok())
                .unwrap_or(DEFAULT_QUANTUM_MS);
            SchedulingAlgorithm::RoundRobin {
                quantum: Duration::from_millis(quantum),
            }
        }
        _ => {
            eprintln!(
                "Uso: cargo run -- <fcfs|rr> [quantum_ms]\n    fcfs        Ejecuta la simulación con First-Come First-Serve\n    rr quantum  Ejecuta la simulación con Round Robin y quantum en milisegundos (por defecto: {DEFAULT_QUANTUM_MS})"
            );
            std::process::exit(1);
        }
    }
}

fn run_simulation(algorithm: SchedulingAlgorithm) {
    println!(
        "=== Simulación de línea de ensamblaje ({}) ===",
        DisplayAlgorithm(&algorithm)
    );

    let station_configs = vec![
        StationConfig {
            name: "Corte",
            processing_time: Duration::from_millis(400),
        },
        StationConfig {
            name: "Ensamblaje",
            processing_time: Duration::from_millis(600),
        },
        StationConfig {
            name: "Empaque",
            processing_time: Duration::from_millis(500),
        },
    ];

    let arrival_offsets: Vec<Duration> = vec![0u64, 120, 260, 380, 540, 720, 900, 1100, 1300, 1500]
        .into_iter()
        .map(Duration::from_millis)
        .collect();

    let products: Vec<Arc<Product>> = arrival_offsets
        .into_iter()
        .enumerate()
        .map(|(idx, offset)| Product::new(idx + 1, offset, &station_configs))
        .collect();
    let total_products = products.len();

    let start_time = Instant::now();
    let (collector_tx, collector_rx) = mpsc::channel::<Arc<Product>>();

    let mut senders = Vec::new();
    let mut receivers = Vec::new();

    for _ in 0..STATION_COUNT {
        let (tx, rx) = mpsc::channel::<Message>();
        senders.push(tx);
        receivers.push(rx);
    }

    let mut receiver_iter = receivers.into_iter();
    let mut handles = Vec::new();

    for (index, config) in station_configs.iter().copied().enumerate() {
        let receiver = receiver_iter.next().expect("Receiver faltante");
        let next_sender = if index + 1 < STATION_COUNT {
            Some(senders[index + 1].clone())
        } else {
            None
        };
        let collector_sender = if index + 1 == STATION_COUNT {
            Some(collector_tx.clone())
        } else {
            None
        };
        let algorithm = algorithm.clone();
        let handle = thread::spawn(move || {
            run_station(
                index,
                config,
                algorithm,
                receiver,
                next_sender,
                collector_sender,
            );
        });
        handles.push(handle);
    }

    let generator_sender = senders
        .first()
        .expect("Se esperaba al menos una estación")
        .clone();

    let generator_configs = station_configs.clone();
    let generator_products = products.clone();
    let generator_handle = thread::spawn(move || {
        run_generator(
            generator_sender,
            generator_products,
            generator_configs[0],
            start_time,
        );
    });

    let mut completion_order = Vec::new();
    for _ in 0..total_products {
        if let Ok(product) = collector_rx.recv() {
            completion_order.push(product.id);
        }
    }

    generator_handle.join().expect("El generador falló");
    for handle in handles {
        handle.join().expect("Una estación falló");
    }

    report_results(&products, &station_configs, start_time, &completion_order);
}

/// Genera los productos respetando sus tiempos de llegada simulados.
fn run_generator(
    sender: mpsc::Sender<Message>,
    products: Vec<Arc<Product>>,
    first_station: StationConfig,
    start_time: Instant,
) {
    for product in products {
        let target_time = start_time + product.arrival_offset;
        loop {
            let now = Instant::now();
            if now >= target_time {
                break;
            }
            thread::sleep(target_time - now);
        }
        let arrival_instant = Instant::now();
        *product.arrival_instant.lock().unwrap() = Some(arrival_instant);
        {
            let mut station_state = product.station_state(0).lock().unwrap();
            station_state.queue_entry = Some(arrival_instant);
            if station_state.remaining.is_zero() {
                station_state.remaining = first_station.processing_time;
            }
        }
        println!(
            "[Generador] Producto {:02} disponible en t={}",
            product.id,
            format_duration(arrival_instant.duration_since(start_time))
        );
        sender
            .send(Message::Product(product))
            .expect("No se pudo enviar el producto a la primera estación");
    }

    sender
        .send(Message::Shutdown)
        .expect("No se pudo notificar el cierre a la primera estación");
}

/// Bucle principal de cada estación: recibe, planifica y procesa productos.
fn run_station(
    index: usize,
    config: StationConfig,
    algorithm: SchedulingAlgorithm,
    receiver: mpsc::Receiver<Message>,
    next_sender: Option<mpsc::Sender<Message>>,
    collector: Option<mpsc::Sender<Arc<Product>>>,
) {
    let mut queue: VecDeque<Arc<Product>> = VecDeque::new();
    let mut shutdown_received = false;

    loop {
        if queue.is_empty() {
            if shutdown_received {
                if let Some(sender) = &next_sender {
                    sender
                        .send(Message::Shutdown)
                        .expect("No se pudo reenviar el cierre");
                }
                break;
            }

            match receiver.recv().expect("Canal cerrado inesperadamente") {
                Message::Product(product) => {
                    register_arrival(&product, index, config);
                    queue.push_back(product);
                }
                Message::Shutdown => {
                    shutdown_received = true;
                }
            }
            continue;
        }

        while let Ok(message) = receiver.try_recv() {
            match message {
                Message::Product(product) => {
                    register_arrival(&product, index, config);
                    queue.push_back(product);
                }
                Message::Shutdown => {
                    shutdown_received = true;
                }
            }
        }

        if let Some(product) = queue.pop_front() {
            process_product(
                product,
                index,
                config,
                &algorithm,
                &mut queue,
                &next_sender,
                &collector,
            );
        }
    }
}

/// Registra que un producto ha llegado a la cola de la estación.
fn register_arrival(product: &Arc<Product>, index: usize, config: StationConfig) {
    let now = Instant::now();
    let mut station_state = product.station_state(index).lock().unwrap();
    station_state.queue_entry = Some(now);
    if station_state.remaining.is_zero() {
        station_state.remaining = config.processing_time;
    }
}

/// Procesa un único tramo de trabajo (o quantum) para un producto en la estación.
fn process_product(
    product: Arc<Product>,
    index: usize,
    config: StationConfig,
    algorithm: &SchedulingAlgorithm,
    queue: &mut VecDeque<Arc<Product>>,
    next_sender: &Option<mpsc::Sender<Message>>,
    collector: &Option<mpsc::Sender<Arc<Product>>>,
) {
    let now = Instant::now();
    let mut station_state = product.station_state(index).lock().unwrap();
    let queue_entry = station_state
        .queue_entry
        .take()
        .expect("Se esperaba tiempo de llegada a la cola");
    station_state.total_wait += now - queue_entry;
    if station_state.first_entry.is_none() {
        station_state.first_entry = Some(now);
    }
    let remaining = station_state.remaining;
    drop(station_state);

    let slice = match algorithm {
        SchedulingAlgorithm::Fcfs => remaining,
        SchedulingAlgorithm::RoundRobin { quantum } => remaining.min(*quantum),
    };

    println!(
        "[{}] Producto {:02} inicia tramo de {} (restante: {})",
        config.name,
        product.id,
        format_duration(slice),
        format_duration(remaining)
    );

    thread::sleep(slice);
    let completed_at = Instant::now();

    let mut station_state = product.station_state(index).lock().unwrap();
    if slice >= remaining {
        station_state.remaining = Duration::ZERO;
        station_state.final_exit = Some(completed_at);
        drop(station_state);
        println!(
            "[{}] Producto {:02} completó la estación (tiempo total restante 0)",
            config.name, product.id
        );
        if let Some(sender) = next_sender {
            sender
                .send(Message::Product(product))
                .expect("No se pudo enviar el producto a la siguiente estación");
        } else if let Some(collector) = collector {
            collector
                .send(product)
                .expect("No se pudo enviar el producto al colector");
        }
    } else {
        station_state.remaining = remaining - slice;
        station_state.queue_entry = Some(completed_at);
        let remaining_after = station_state.remaining;
        drop(station_state);
        println!(
            "[{}] Producto {:02} interrumpido, vuelve a la cola (restante: {})",
            config.name,
            product.id,
            format_duration(remaining_after)
        );
        queue.push_back(product);
    }
}

/// Genera el reporte con las métricas finales de todos los productos.
fn report_results(
    products: &[Arc<Product>],
    station_configs: &[StationConfig],
    start_time: Instant,
    completion_order: &[usize],
) {
    println!("\n=== Resultados ===");
    println!(
        "{:^8} {:^12} {:^15} {:^15} {:^15} {:^12} {:^15}",
        "Prod", "Llegada", "Corte", "Ensamblaje", "Empaque", "Espera", "Turnaround"
    );

    let mut total_wait = Duration::ZERO;
    let mut total_turnaround = Duration::ZERO;
    let total_products = products.len() as u32;

    for product in products {
        let arrival = product
            .arrival_instant
            .lock()
            .unwrap()
            .expect("Cada producto debe tener tiempo de llegada");
        let arrival_time = arrival.duration_since(start_time);

        let mut station_times = Vec::new();
        let mut product_wait = Duration::ZERO;
        for (index, _) in station_configs.iter().enumerate() {
            let state = product.station_state(index).lock().unwrap();
            let entry = state
                .first_entry
                .map(|t| t.duration_since(start_time))
                .unwrap_or_default();
            let exit = state
                .final_exit
                .map(|t| t.duration_since(start_time))
                .unwrap_or_default();
            station_times.push(format!(
                "{}-{}",
                format_duration(entry),
                format_duration(exit)
            ));
            product_wait += state.total_wait;
        }

        let final_exit = product
            .station_state(station_configs.len() - 1)
            .lock()
            .unwrap()
            .final_exit
            .expect("El producto debe haber terminado")
            .duration_since(start_time);
        let turnaround = final_exit - arrival_time;

        total_wait += product_wait;
        total_turnaround += turnaround;

        println!(
            "{:^8} {:^12} {:^15} {:^15} {:^15} {:^12} {:^15}",
            format!("#{:02}", product.id),
            format_duration(arrival_time),
            station_times[0].clone(),
            station_times[1].clone(),
            station_times[2].clone(),
            format_duration(product_wait),
            format_duration(turnaround)
        );
    }

    let avg_wait = total_wait / total_products;
    let avg_turnaround = total_turnaround / total_products;

    println!(
        "\nPromedio de espera: {}\nPromedio de turnaround: {}\nOrden final de terminación: {:?}",
        format_duration(avg_wait),
        format_duration(avg_turnaround),
        completion_order
    );
}

/// Formatea una duración en segundos con precisión de milisegundos.
fn format_duration(duration: Duration) -> String {
    let millis = duration.as_millis();
    let seconds = millis / 1000;
    let milliseconds = millis % 1000;
    format!("{}.{:03}s", seconds, milliseconds)
}

struct DisplayAlgorithm<'a>(&'a SchedulingAlgorithm);

impl fmt::Display for DisplayAlgorithm<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            SchedulingAlgorithm::Fcfs => write!(f, "FCFS"),
            SchedulingAlgorithm::RoundRobin { quantum } => {
                write!(f, "Round Robin (quantum {} ms)", quantum.as_millis())
            }
        }
    }
}
