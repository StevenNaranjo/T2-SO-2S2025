
# Sincronización y Algoritmos de Scheduling

- Luis Fernando Benavides Villegas
- Alex Naranjo Masis


## Descripción de la Solución

## Justificación Técnica

## Comparación entre Algoritmos







## Explicación de los Algoritmos

### First Come First Serve
```
0ms, Generador: Encolando producto #0 (arrival real: 0 ms, offset pedido: 0 ms)
0ms, Generador: Encolando producto #1 (arrival real: 0 ms, offset pedido: 0 ms)
0ms, Estación (FCFS): Procesando producto #0 (tiempo: 220 ms)
220ms, Estación (FCFS): Producto #0 finalizado en esta estación
220ms, Estación (FCFS): Procesando producto #1 (tiempo: 220 ms)
441ms, Estación (FCFS): Producto #1 finalizado en esta estación

Tiempo de procesamiento de la estación → 220 ms

Estado final de las colas:
Cola IN   → 0 elementos
Cola DONE → 0 elementos

Orden final de procesamiento:
[0, 1]

==================================  INFORME FINAL  ===================================
ID    Arr(ms) │     S_in    S_out │   Turn(ms)   Wait(ms)
--------------------------------------------------------------------------------------
0           0 │        0      220 │        220          0
1           0 │      220      441 │        441        221
--------------------------------------------------------------------------------------
AVG           │                   │     330.50     110.50
======================================================================================
```


### Round Robin (Quantum = 80)
```
0ms, Generador: Encolando producto #0 (arrival real: 0 ms, offset pedido: 0 ms)
0ms, Generador: Encolando producto #1 (arrival real: 0 ms, offset pedido: 0 ms)
0ms, Estación (RR): Procesando producto #0 por 80 ms (restante antes del slice: 220 ms)
80ms, Estación (RR): Procesando producto #1 por 80 ms (restante antes del slice: 220 ms)
160ms, Estación (RR): Procesando producto #0 por 80 ms (restante antes del slice: 140 ms)
241ms, Estación (RR): Procesando producto #1 por 80 ms (restante antes del slice: 140 ms)
322ms, Estación (RR): Procesando producto #0 por 60 ms (restante antes del slice: 60 ms)
382ms, Estación (RR): Producto #0 finalizado en esta estación
382ms, Estación (RR): Procesando producto #1 por 60 ms (restante antes del slice: 60 ms)
442ms, Estación (RR): Producto #1 finalizado en esta estación

Tiempo de procesamiento de la estación → 220 ms

Estado final de las colas:
Cola IN   → 0 elementos
Cola DONE → 0 elementos

Orden final de procesamiento:
[0, 1]

==================================  INFORME FINAL  ===================================
ID    Arr(ms) │     S_in    S_out │   Turn(ms)   Wait(ms)
--------------------------------------------------------------------------------------
0           0 │        0      382 │        382        162
1           0 │       80      442 │        442        222
--------------------------------------------------------------------------------------
AVG           │                   │     412.00     192.00
======================================================================================
```