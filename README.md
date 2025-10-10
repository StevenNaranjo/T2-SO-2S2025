# Sincronización y Algoritmos de Scheduling - Simulador de Línea de Ensamblaje

**Autores:** Luis Fernando Benavides Villegas – Alex Naranjo Masís  
**Curso:** Principios de Sistemas Operativos – Instituto Tecnológico de Costa Rica  

## Descripción General

Este proyecto implementa una **línea de ensamblaje concurrente** que simula el flujo de productos a través de tres estaciones de trabajo:  
**Corte → Ensamblaje → Empaque**

Cada estación se ejecuta en un **hilo independiente** y se comunica con las demás mediante **colas sincronizadas** que garantizan exclusión mutua y paso seguro de datos.  
El sistema permite seleccionar dinámicamente el algoritmo de planificación utilizado por cada estación:

- **FCFS (First-Come, First-Served)**  
- **RR (Round Robin)** con *quantum* configurable  

El objetivo es analizar el impacto de ambos algoritmos en métricas como el tiempo de espera, turnaround, y orden de finalización.

---

### Estructura del Proyecto

```bash
T2-SO-2S2025/
├── src/
│   ├── main.rs                 # Punto de entrada, configuración y generación de estadísticas.
│   ├── estaciones.rs           # Structs de colas con mutex, productos y cálculo de tiempos.
│   └── funciones.rs            # Implementa los algoritmos de scheduling.
├── test/
│   ├── main_individual.rs      # Para hacer pruebas con una sola estación a la vez.
│   └── mutex_demo.rs           # Prueba de concepto de exclusión mútua.
├── Cargo.toml
├── README.md                   # Este archivo.
└── Informe.pdf                 # Informe técnico de la solución.
```

---

## Requisitos Previos

Para compilar y ejecutar el proyecto, es necesario tener instalado:

- **Rust y Cargo**
- **Versión mínima recomendada:** Rust 1.76 o superior

Comprobar instalación con:
```bash
rustc --version
cargo --version
```

---

## Ejecución
El programa recibe tres parámetros, uno por estación, indicando el **algoritmo** y el **tiempo de procesamiento** (y opcionalmente el *quantum* en caso de RR):

```bash
cargo run -- <E1_alg> <E1_tiempo> [<E1_q>] <E2_alg> <E2_tiempo> [<E2_q>] <E3_alg> <E3_tiempo> [<E3_q>]
```

## Configuración
En el archivo `src/main.rs`, en la función `main()` es posible localizar en la sección de configuración el array de las llegadas de los procesos. Estos son el tiempo en milisegundos que hay entre la entrada de un proceso y el anterior.

```rust
let arrivals_ms: Vec<i32> = vec![0, 0, 50, 50, 100, 100, 150, 200, 250, 300];   // Cambiar a los offsets deseados.
```

Por lo que si ponemos `[0, 50, 100]`:
- **P0** llega en 0ms.
- **P1** llega en 50ms.
- **P2** llega en 150ms.


## Ejemplos

### Todas las estaciones con FCFS
Simula una línea donde todas las estaciones procesan productos de forma secuencial completa.
```bash
cargo run -- fcfs 120 fcfs 220 fcfs 100
```

### Todas las estaciones con Round Robin (q = 80)
Cada estación usa planificación por *quantum* de 80 ms, con interrupciones y reencolado de productos.
```bash
cargo run -- rr 120 80 rr 220 80 rr 100 80
```

### Combinación de algoritmos
La primera y tercera estaciones usan FCFS, mientras la segunda utiliza Round Robin.
```bash
cargo run -- fcfs 140 rr 220 100 fcfs 120
```

#### Ejemplo de Salida
El sistema genera un informe tabulado con los tiempos por producto y promedios globales:
```bash
Tiempos de procesamiento por estación:
 - Corte      (#1, FCFS)            → 140 ms
 - Ensamblaje (#2, RR, q=100)        → 220 ms
 - Empaque    (#3, FCFS)            → 120 ms

Orden final de procesamiento:
[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]

========================================================  INFORME FINAL  =========================================================
ID    Arr(ms) │    E1_in   E1_out    Wait1 │    E2_in   E2_out    Wait2 │    E3_in   E3_out    Wait3 │   Turn(ms)    WaitTot
----------------------------------------------------------------------------------------------------------------------------------
0           0 │        0      140        0 │      141      464      103 │      465      585        1 │        585        105
1           0 │      141      281      141 │      342      887      386 │      888     1008        1 │       1008        528
2          51 │      282      422      231 │      464     1209      566 │     1209     1330        1 │       1279        799
3         101 │      423      563      322 │      665     1431      647 │     1431     1552        1 │       1451        971
4         202 │      564      704      362 │      887     1954     1030 │     1955     2075        1 │       1873       1393
5         303 │      705      845      402 │     1088     2076     1011 │     2076     2197        1 │       1894       1414
6         454 │      845      986      392 │     1209     2197      991 │     2198     2318        1 │       1864       1384
7         654 │      987     1127      333 │     1431     2319      972 │     2319     2440        1 │       1786       1306
8         905 │     1127     1268      223 │     1632     2340      852 │     2440     2560      100 │       1655       1175
9        1206 │     1268     1408       62 │     1833     2361      732 │     2561     2681      200 │       1475        995
----------------------------------------------------------------------------------------------------------------------------------
AVG           │                     246.80 │                     729.00 │                      30.80 │    1487.00    1007.00
==================================================================================================================================
```