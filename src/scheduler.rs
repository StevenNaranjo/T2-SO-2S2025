//! # Módulo de Algoritmos de Planificación
//! 
//! Este módulo implementa los diferentes algoritmos de scheduling utilizados
//! por las estaciones de trabajo para determinar el orden y quantum de
//! procesamiento de los productos.

use std::fmt;
use std::time::Duration;

/// Algoritmos de planificación disponibles para las estaciones.
/// 
/// Cada algoritmo define una estrategia diferente para procesar productos:
/// - FCFS garantiza que los productos se procesen en orden de llegada
/// - Round Robin permite compartir tiempo de CPU entre múltiples productos
#[derive(Clone, Debug, PartialEq)]
pub enum SchedulingAlgorithm {
    /// First-Come First-Served: procesamiento no preemptivo en orden de llegada.
    /// 
    /// Cada producto se procesa completamente antes de tomar el siguiente.
    /// Es simple pero puede causar esperas largas si llegan productos con
    /// tiempos de procesamiento muy diferentes.
    Fcfs,
    
    /// Round Robin: procesamiento preemptivo con quantum fijo.
    /// 
    /// Los productos se procesan en rondas, donde cada uno recibe un quantum
    /// de tiempo antes de ser interrumpido (si no ha terminado) y volver
    /// al final de la cola. Mejora la equidad pero puede incrementar el
    /// tiempo promedio de turnaround.
    RoundRobin {
        /// Tiempo máximo de procesamiento continuo por ronda
        quantum: Duration,
    },
}

impl SchedulingAlgorithm {
    /// Crea un nuevo algoritmo FCFS.
    /// 
    /// # Returns
    /// 
    /// Una instancia de `SchedulingAlgorithm::Fcfs`
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use assembly_line_simulator::SchedulingAlgorithm;
    /// 
    /// let algorithm = SchedulingAlgorithm::fcfs();
    /// ```
    pub fn fcfs() -> Self {
        Self::Fcfs
    }

    /// Crea un nuevo algoritmo Round Robin con el quantum especificado.
    /// 
    /// # Arguments
    /// 
    /// * `quantum` - Duración máxima de procesamiento continuo por ronda
    /// 
    /// # Returns
    /// 
    /// Una instancia de `SchedulingAlgorithm::RoundRobin`
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use std::time::Duration;
    /// use assembly_line_simulator::SchedulingAlgorithm;
    /// 
    /// let algorithm = SchedulingAlgorithm::round_robin(Duration::from_millis(300));
    /// ```
    pub fn round_robin(quantum: Duration) -> Self {
        Self::RoundRobin { quantum }
    }

    /// Determina si el algoritmo es preemptivo.
    /// 
    /// # Returns
    /// 
    /// `true` si el algoritmo puede interrumpir productos en proceso,
    /// `false` si los procesa hasta completar
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use std::time::Duration;
    /// use assembly_line_simulator::SchedulingAlgorithm;
    /// 
    /// assert!(!SchedulingAlgorithm::fcfs().is_preemptive());
    /// assert!(SchedulingAlgorithm::round_robin(Duration::from_millis(100)).is_preemptive());
    /// ```
    pub fn is_preemptive(&self) -> bool {
        match self {
            Self::Fcfs => false,
            Self::RoundRobin { .. } => true,
        }
    }

    /// Calcula el quantum de procesamiento para un producto dado.
    /// 
    /// # Arguments
    /// 
    /// * `remaining_time` - Tiempo de procesamiento restante del producto
    /// 
    /// # Returns
    /// 
    /// La duración máxima que se debe procesar el producto en esta ronda
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use std::time::Duration;
    /// use assembly_line_simulator::SchedulingAlgorithm;
    /// 
    /// let fcfs = SchedulingAlgorithm::fcfs();
    /// let remaining = Duration::from_millis(500);
    /// assert_eq!(fcfs.calculate_quantum(remaining), remaining);
    /// 
    /// let rr = SchedulingAlgorithm::round_robin(Duration::from_millis(300));
    /// assert_eq!(rr.calculate_quantum(remaining), Duration::from_millis(300));
    /// ```
    pub fn calculate_quantum(&self, remaining_time: Duration) -> Duration {
        match self {
            Self::Fcfs => remaining_time,
            Self::RoundRobin { quantum } => remaining_time.min(*quantum),
        }
    }

    /// Obtiene el quantum configurado para Round Robin.
    /// 
    /// # Returns
    /// 
    /// `Some(Duration)` con el quantum si es Round Robin,
    /// `None` si es FCFS
    pub fn get_quantum(&self) -> Option<Duration> {
        match self {
            Self::Fcfs => None,
            Self::RoundRobin { quantum } => Some(*quantum),
        }
    }

    /// Obtiene una descripción textual del algoritmo.
    /// 
    /// # Returns
    /// 
    /// String describiendo el algoritmo y sus parámetros
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use std::time::Duration;
    /// use assembly_line_simulator::SchedulingAlgorithm;
    /// 
    /// let fcfs = SchedulingAlgorithm::fcfs();
    /// assert_eq!(fcfs.description(), "First-Come First-Served (no preemptivo)");
    /// 
    /// let rr = SchedulingAlgorithm::round_robin(Duration::from_millis(300));
    /// assert!(rr.description().contains("Round Robin"));
    /// ```
    pub fn description(&self) -> String {
        match self {
            Self::Fcfs => "First-Come First-Served (no preemptivo)".to_string(),
            Self::RoundRobin { quantum } => {
                format!(
                    "Round Robin preemptivo (quantum: {} ms)",
                    quantum.as_millis()
                )
            }
        }
    }
}

impl fmt::Display for SchedulingAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fcfs => write!(f, "FCFS"),
            Self::RoundRobin { quantum } => {
                write!(f, "Round Robin (quantum {} ms)", quantum.as_millis())
            }
        }
    }
}

/// Wrapper para mostrar algoritmos de planificación de forma legible.
/// 
/// Esta estructura permite formatear algoritmos de scheduling de manera
/// consistente en logs y reportes.
pub struct DisplayAlgorithm<'a>(pub &'a SchedulingAlgorithm);

impl<'a> DisplayAlgorithm<'a> {
    /// Crea un nuevo wrapper para mostrar un algoritmo.
    /// 
    /// # Arguments
    /// 
    /// * `algorithm` - Referencia al algoritmo a mostrar
    /// 
    /// # Returns
    /// 
    /// Nueva instancia de `DisplayAlgorithm`
    pub fn new(algorithm: &'a SchedulingAlgorithm) -> Self {
        Self(algorithm)
    }
}

impl fmt::Display for DisplayAlgorithm<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fcfs_creation() {
        let algorithm = SchedulingAlgorithm::fcfs();
        assert_eq!(algorithm, SchedulingAlgorithm::Fcfs);
        assert!(!algorithm.is_preemptive());
    }

    #[test]
    fn test_round_robin_creation() {
        let quantum = Duration::from_millis(500);
        let algorithm = SchedulingAlgorithm::round_robin(quantum);
        
        match algorithm {
            SchedulingAlgorithm::RoundRobin { quantum: q } => {
                assert_eq!(q, quantum);
            }
            _ => panic!("Expected RoundRobin variant"),
        }
        
        assert!(algorithm.is_preemptive());
    }

    #[test]
    fn test_quantum_calculation() {
        let remaining = Duration::from_millis(800);
        
        // FCFS should return the full remaining time
        let fcfs = SchedulingAlgorithm::fcfs();
        assert_eq!(fcfs.calculate_quantum(remaining), remaining);
        
        // Round Robin should return min(remaining, quantum)
        let rr_small = SchedulingAlgorithm::round_robin(Duration::from_millis(300));
        assert_eq!(rr_small.calculate_quantum(remaining), Duration::from_millis(300));
        
        let rr_large = SchedulingAlgorithm::round_robin(Duration::from_millis(1000));
        assert_eq!(rr_large.calculate_quantum(remaining), remaining);
    }

    #[test]
    fn test_display() {
        let fcfs = SchedulingAlgorithm::fcfs();
        assert_eq!(format!("{}", fcfs), "FCFS");
        
        let rr = SchedulingAlgorithm::round_robin(Duration::from_millis(250));
        assert_eq!(format!("{}", rr), "Round Robin (quantum 250 ms)");
    }
}