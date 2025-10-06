# Ejemplos de Uso del Simulador

Este directorio contiene ejemplos prácticos de cómo utilizar el simulador de línea de ensamblaje para diferentes casos de uso.

## 📁 Estructura de Ejemplos

```
examples/
├── basic_usage.rs          # Uso básico del simulador
├── custom_config.rs        # Configuración personalizada
├── performance_analysis.rs # Análisis de rendimiento
└── batch_simulation.rs     # Simulaciones en lote
```

## 🚀 Ejecutar Ejemplos

```bash
# Ejemplo básico
cargo run --example basic_usage

# Configuración personalizada  
cargo run --example custom_config

# Análisis de rendimiento
cargo run --example performance_analysis

# Simulaciones en lote
cargo run --example batch_simulation
```

## 📋 Descripción de Ejemplos

### basic_usage.rs
Demuestra el uso básico del simulador con configuraciones predeterminadas.

### custom_config.rs
Muestra cómo crear configuraciones personalizadas de estaciones y productos.

### performance_analysis.rs
Compara diferentes algoritmos y parámetros para análisis de rendimiento.

### batch_simulation.rs
Ejecuta múltiples simulaciones con diferentes configuraciones para análisis estadístico.

## 📊 Salida de Ejemplos

Cada ejemplo genera reportes detallados que incluyen:
- Métricas de rendimiento
- Comparaciones entre algoritmos
- Visualizaciones de datos
- Archivos CSV para análisis posterior