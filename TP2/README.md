# YPF Ruta - Sistema Distribuido de Gestión de Pagos

Sistema distribuido de gestión de transacciones de pago para estaciones de servicio YPF, implementado con Rust y Actix. El sistema permite procesar pagos de forma concurrente, tolerante a fallos y con alta disponibilidad mediante un cluster de nodos replicados.

---

## Descripción del Sistema

YPF Ruta es una plataforma que permite a empresas gestionar los pagos de combustible de sus empleados en estaciones de servicio. El sistema está diseñado para:

- **Procesar transacciones de pago** de forma distribuida y concurrente
- **Mantener consistencia** de datos a través de múltiples nodos
- **Tolerar fallos** de nodos individuales sin perder disponibilidad
- **Operar en modo offline** cuando las estaciones pierden conectividad
- **Sincronizar automáticamente** los datos cuando se recupera la conexión

---

## Arquitectura del Sistema

El sistema está compuesto por cuatro componentes principales que se comunican entre sí:

```
┌──────────┐         ┌──────────┐         ┌──────────┐
│  Pump 1  │         │  Pump 2  │         │  Pump N  │
└────┬─────┘         └────┬─────┘         └────┬─────┘
     │                    │                    │
     └────────────────────┼────────────────────┘
                          │ TCP
                    ┌─────▼──────┐
                    │  Station   │ ◄─── Modo Offline/Online
                    └─────┬──────┘
                          │ TCP
              ┌───────────┼───────────┐
              │           │           │
         ┌────▼───┐  ┌────▼───┐  ┌───▼────┐
         │ Node 1 │  │ Node 2 │  │ Node N │
         │(Leader)│◄─┤        │◄─┤        │
         └────┬───┘  └────┬───┘  └───┬────┘
              │           │          │
              └───────────┼──────────┘
                     UDP (Heartbeat)
                          │
                    ┌─────▼──────┐
                    │ Enterprise │ ◄─── Proxy
                    └────────────┘
```

---

## Componentes

### 1. [Cluster](cluster/README.md)

Nodos del cluster distribuido que mantienen réplicas de los datos y procesan transacciones.

**Características principales:**
- Elección de líder mediante algoritmo de Bully
- Consenso distribuido con locks centralizados
- Detección de fallos mediante heartbeats UDP
- Replicación de datos entre nodos
- Recuperación automática ante caídas

**Actores clave:**
- `ConnectionManager`: Gestión de conexiones TCP
- `TransactionManager`: Coordinación de transacciones distribuidas
- `ElectionManager`: Elección de líder
- `HeartbeatManager`: Monitoreo de salud de nodos
- `LockManager`: Control de concurrencia
- `RepositoryManager`: Almacenamiento local de datos

---

### 2. [Station](station/README.md)

Estaciones de servicio que actúan como gateway entre los surtidores y el cluster.

**Características principales:**
- Modo online: Validación de pagos con el cluster
- Modo offline: Operación autónoma sin cluster
- Cola de pagos pendientes
- Sincronización automática al reconectar
- Persistencia local de transacciones

**Actores clave:**
- `Station`: Orquestador principal
- `ConnectionEstablisher`: Conexión con cluster
- `PumpConnectionListener`: Escucha de pumps
- `HeartbeatManager`: Detección de caídas del cluster
- `RepositoryManager`: Persistencia local

---

### 3. [Pump](pump/README.md)

Surtidores de combustible que generan solicitudes de pago.

**Características principales:**
- Generación de pagos simulados
- Conexión TCP con estación
- Procesamiento de resultados de transacciones
- Monitoreo de conexión mediante heartbeats

**Actores clave:**
- `Pump`: Generador de pagos
- `ConnectionEstablisher`: Conexión con estación
- `HeartbeatManager`: Monitoreo de conexión

---

### 4. [Enterprise](enterprise/README.md)

Interfaz de gestión para empresas que permite administrar límites y consultar transacciones.

**Características principales:**
- Interfaz CLI para administración
- Actualización de límites de tarjetas y empresas
- Visualización de transacciones en tiempo real
- Reconexión automática al cluster
- Proxy para comunicación con el líder

**Actores clave:**
- `Enterprise`: Lógica de negocio
- `Proxy`: Comunicación con cluster
- `AdminInput`: Interfaz de comandos
- `LeaderFindingManager`: Búsqueda del líder

---

## Características Principales

### Transacciones Distribuidas

El sistema implementa tres tipos de transacciones:

1. **Verificación de Pago**: Valida si un pago puede autorizarse según límites
2. **Actualización Forzada**: Registra pagos procesados offline
3. **Actualización de Límites**: Modifica límites de tarjetas/empresas

Todas las transacciones utilizan un algoritmo de consenso basado en:
- Lock centralizado en el líder
- Consulta a todos los nodos del cluster
- Validación y propagación de cambios
- Liberación del lock

### Tolerancia a Fallos

- **Caída de nodos regulares**: El sistema continúa operando
- **Caída del líder**: Elección automática de nuevo líder
- **Caída de estación**: Pumps pueden reconectarse
- **Pérdida de conectividad**: Estaciones operan en modo offline

### Consistencia de Datos

- Replicación de datos en todos los nodos
- Sincronización al reconectar
- Validación distribuida de transacciones

### Alta Disponibilidad

- Cluster sin punto único de fallo
- Reconexión automática de componentes
- Operación offline de estaciones
- Heartbeats para detección rápida de fallos

---

## Configuración Rápida

### Variables de Entorno

Crear un archivo `.env` en la raíz del proyecto:

```env
# Cluster
NUM_NODES=4
SERVER_PORT_BASE=7000
PROXY_PORT_BASE=6000
STATION_PORT_BASE=8000
HB_MISSES_ALLOWED=3
LOG_LEVEL=Info

# Station
STATION_ID=1
CLUSTER_ADDRESS=127.0.0.1:7001
HOSTNAME=127.0.0.1
BASE_PORT=10000

# Pump
STATION_ADDRESS=127.0.0.1:10000
PING_RECEIVE_BASE_PORT=9000
```

---

## Ejecución

### Levantar el Cluster

```bash
# Terminal 1 - Nodo 1 (será el líder)
cargo run -p cluster -- 1

# Terminal 2 - Nodo 2
cargo run -p cluster -- 2

# Terminal 3 - Nodo 3
cargo run -p cluster -- 3

# Terminal 4 - Nodo 4
cargo run -p cluster -- 4
```

### Levantar Estaciones

```bash
# Terminal 5
cd station
cargo run
```

### Levantar Pumps

```bash
cd pump
cargo run
```

### Levantar Enterprise

```bash
cd enterprise
cargo run enterprise_config.toml
```

---

## Comandos de Administración (Enterprise)

Una vez levantado el enterprise, puedes usar estos comandos:

```bash
# Actualizar límite de empresa
enterprise add 5000
enterprise sub 1000
enterprise set 10000

# Actualizar límite de tarjeta
card add 1 500
card sub 2 100
card set 3 2000

# Ver información
enterprise view
card view 1
```

---

## Miembros del Grupo

1. Mateo Valentin Serrano Godoy (padrón 110912)
2. Jonathan Dominguez (padrón 110057)
3. Dídimo Páez (padrón 98910)

---

## Documentación Adicional

- [README Primera Entrega](README_1ra_entrega.md) - Documentación detallada de la arquitectura inicial
- [Cluster](cluster/README.md) - Documentación del cluster distribuido
- [Station](station/README.md) - Documentación de estaciones
- [Pump](pump/README.md) - Documentación de surtidores
- [Enterprise](enterprise/README.md) - Documentación de gestión empresarial
