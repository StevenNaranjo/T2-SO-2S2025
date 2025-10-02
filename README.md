# Simulador de Línea de Ensamblaje

Este proyecto implementa en Rust un simulador de una línea de ensamblaje compuesta por tres estaciones de trabajo conectadas mediante canales (`std::sync::mpsc`). Cada estación se ejecuta en un hilo independiente y procesa los productos aplicando algoritmos de planificación FCFS o Round Robin con quantum configurable.

## Requisitos

* Rust 1.70 o superior.
* Cargo para compilar y ejecutar el proyecto.

## Ejecución

1. Compilar y ejecutar con FCFS:
   ```bash
   cargo run -- fcfs
   ```
2. Compilar y ejecutar con Round Robin (quantum en milisegundos, 300 ms por defecto):
   ```bash
   cargo run -- rr 300
   ```

Durante la ejecución se registran en la consola los eventos de llegada, ejecución, interrupciones y finalización por estación. Al terminar se presenta un resumen con tiempos de llegada, ventanas de entrada/salida por estación, tiempo total de espera y turnaround de cada producto.

## Logs de referencia

Se incluyen dos archivos en `logs/` con ejecuciones completas:

* `logs/fcfs.log`
* `logs/round_robin_300ms.log`

## Estructura

* `src/main.rs`: implementación completa del simulador.
* `logs/`: ejemplos de ejecución.
* `REPORT.md`: informe técnico con descripción, decisiones y análisis comparativo.

## Métricas calculadas

* Tiempo de llegada simulado.
* Rango de entrada/salida en cada estación.
* Tiempo total de espera por producto (suma en las tres estaciones).
* Turnaround por producto.
* Promedios de espera y turnaround, además del orden final de completitud.

## Consideraciones

* La cantidad de productos y los tiempos de procesamiento por estación están definidos en el código, pero pueden ajustarse fácilmente en `run_simulation`.
* En Round Robin el quantum provoca reencolado de productos, lo que incrementa los tiempos de espera y permite comparar el comportamiento frente a FCFS.
# T2-SO-2S2025
Sincronizacion y Algoritmos de Scheduling

Creado por:
- Luis Fernando Benavides Villegas
- Alex Steven Naranjo Masis
