# Informe Técnico

## Descripción de la Solución

El simulador está compuesto por tres hilos principales que representan las estaciones de Corte, Ensamblaje y Empaque. Los productos son modelados mediante estructuras compartidas (`Arc`) que almacenan métricas protegidas con `Mutex`. La comunicación entre estaciones se realiza con canales `mpsc`, lo que permite aislar a cada estación y garantizar el orden de mensajes.

Un generador produce diez productos con tiempos de llegada escalonados. Cada producto se encola en la primera estación, respetando el tiempo simulado mediante `thread::sleep`. Las estaciones consumen productos desde su cola local, aplicando el algoritmo de planificación seleccionado.

## Sincronización e IPC

* **Canales (`mpsc`)**: conectan estaciones y garantizan entrega en orden FIFO.
* **Mutexes**: protegen las métricas de cada producto (tiempos de entrada, salida y espera) contra condiciones de carrera.
* **Semántica de exclusión mutua**: cada estación procesa un solo producto a la vez. El propio hilo actúa como recurso exclusivo y el canal evita que múltiples productos ingresen simultáneamente.
* **Mensajes de apagado**: se envía un mensaje `Shutdown` para finalizar la cadena sin dejar productos pendientes.

## Algoritmos de Scheduling

* **FCFS**: procesa el producto completo antes de tomar el siguiente. Las esperas se deben únicamente a la cola previa.
* **Round Robin**: utiliza un quantum configurable (300 ms por defecto). Los productos que no finalizan dentro del quantum se reencolan con el tiempo restante, simulando preempciones.

## Métricas Registradas

Para cada producto se captura:

* Tiempo de llegada simulado.
* Primer instante de servicio y salida final en cada estación.
* Tiempo de espera acumulado (sumatoria de permanencia en cola por estación).
* Turnaround total.
* Orden de finalización global.

## Comparación entre Algoritmos

Los logs incluidos muestran que FCFS mantiene el orden de llegada con esperas crecientes pero controladas. Round Robin incrementa el tiempo de espera promedio debido al reencolado constante, aunque mejora la equidad al permitir que productos nuevos avancen temprano.

| Algoritmo | Promedio espera | Promedio turnaround |
|-----------|-----------------|---------------------|
| FCFS      | 1.561 s         | 3.523 s             |
| Round Robin (300 ms) | 2.513 s         | 4.277 s             |

La diferencia evidencia la sobrecarga que introduce la preempción cuando el quantum es pequeño frente a los tiempos de procesamiento reales.

## Justificación Técnica

* Rust garantiza seguridad en memoria y ausencia de data races gracias al modelo de propiedad y al uso explícito de `Arc<Mutex<_>>`.
* Los canales `mpsc` simplifican la comunicación y actúan como mecanismo de sincronización natural entre hilos.
* Las métricas se calculan a partir de `Instant`, ofreciendo precisión suficiente para comparar algoritmos.
* Se optó por tiempos de procesamiento relativamente cortos (400/600/500 ms) para obtener resultados rápidos pero observables.

## Trabajo Futuro

* Permitir configurar número de estaciones y tiempos desde un archivo externo.
* Añadir más algoritmos (SPN, Priority) y gráficas automáticas de métricas.
* Persistir resultados en CSV para análisis adicional.
