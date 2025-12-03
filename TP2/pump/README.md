# Pump Service

El servicio `Pump` simula un surtidor de combustible. Recibe pedidos de pago y los
redirige a una estación asignada de manera concurrente.

## Actores

El servicio cuenta con los siguientes actores:

### Pump

El **actor principal** que simula el comportamiento del surtidor. Recibe pagos y
procesa las respuestas de la estación (para esta implementación, es simplemente
imprimir el resultado obtenido).

#### Estado Interno

```rust
pub struct Pump {
    id: Option<u32>,
    ping_receive_port: u32,
    connection_establisher: Option<ConnectionEstablisher>,
    heartbeat_manager: Option<Addr<HeartbeatManager<Self>>>,
    protocol: Option<TcpProtocol<Self, PumpReceiveMessageType>>,
    pending: VecDeque<Payment>,
}
```

#### Mensajes que Maneja

`Pump` tiene su propio protocolo para mensajes TCP recibidos de su estación asignada,
que le permite manejar los siguientes mensajes.

1. **`SetPumpId`**: Recibe su ID asignado por la estación al conectarse. Almacena el
ID en su estado interno para poder inyectárselo a los nuevos pedidos que lleguen.

2. **`TransactionResult`**: Recibe el resultado de una transacción de pago desde la 
estación.
   - Si el resultado es **aprobado**, registra el pago exitoso.
   - Si es **rechazado**, registra el rechazo.

Además de los mensajes recibidos por TCP, `Pump` maneja otros mensajes:

3. **Payment**: El pump recibe pedidos de validación de pago de los clientes:
   - Envía el `Payment` a la estación a través de un `TcpProtocol`.
   - Espera el `TransactionResult` correspondiente, sin bloquear el procesamiento
   de más mensajes.

4. **ConnectionLost**: Cuando la estación deja de enviar heartbeats al `Pump`,
su `HeartbeatManager` le envía este mensaje al `Pump`, y este da de baja su
protocolo e intenta reconectarse nuevamente. Hasta que el nuevo protocolo sea
establecido, los pedidos de pago recibidos quedarán en espera de envío.

---

### ConnectionEstablisher

El **establecedor de conexión** gestiona la conexión TCP con la estación. No
es un actor, sino que expone métodos asincrónicos para generar conexiones.

#### Funcionamiento

- Intenta conectarse a la estación configurada.
- Si la conexión falla, reintenta periódicamente.
- Una vez establecida la conexión, devuelve un `TcpProtocol` que permite recibir
mensajes del protocolo que maneja el `Pump`.
- El `Pump` toma el protocolo y levanta el `HeartbeatManager` para empezar a recibir
los heartbeats.
- El `Pump` envía todos los pedidos de pago pendientes.

---

### HeartbeatManager

El **gerente de heartbeats** maneja la recepción de heartbeats de la estación para
mantener la conexión activa.

#### Estado Interno

```rust
pub struct HeartbeatManager<T>
where
    T: Actor,
    T::Context: ToEnvelope<T, ConnectionLost>,
    T: Handler<ConnectionLost>,
{
    id: u32,
    port: u32,
    udp_io: Option<Addr<UdpIO>>,
    recipient: Addr<T>, // En este caso, un Addr<Pump>
    ping_receive_interval: Option<SpawnHandle>,
    last_ping_time: Instant,
}
```

#### Funcionamiento

- Escucha heartbeats UDP de la estación.
- Cada cierto tiempo, igual para todos los `HeartbeatManager`, verifica el
instante de recepción del último heartbeat.
- Si el tiempo desde el último heartbeat es mayor a cierto umbral, se da por
caída la estación y se envía `ConnectionLost` al `Pump`.
- Si el tiempo es menor al umbral, no se hace nada.

---

## Funcionamiento del `Pump`

El `Pump` sigue un ciclo de vida sencillo:

### 1. Conexión

- Se conecta vía TCP a la estación configurada mediante el `ConnectionEstablisher`.
- Levanta un `HeartbeatManager` que le avise cuando cae la estación.

### 2. Identificación

- Espera recibir un `SetPumpId` de la estación. Mientras tanto puede recibir
pedidos de pago pero no los enviará.
- La estación asigna un ID único al pump.
- El pump almacena este ID para futuras transacciones.
- Envía todas las transacciones que hubieran estado pendientes.

### 3. Operación

El pump opera en un ciclo continuo:

- Recibe mensajes `Payment` con:
  - Datos de tarjeta (ID de empresa y tarjeta).
  - Costo de la transacción.

- Envía los pedidos de pago: 
  - Le inyecta a los `Payment` su propio ID.
  - Envía los `Payment` a la estación a través del `TcpProtocol`.
  - Puede recibir y reenviar más de un `Payment` concurrentemente.

- Espera y procesa los `TransactionResult`:
  - Independientemente del resultado, lo imprime.
  - Puede recibir más de un `TransactionResult` concurrentemente.

## Configuración

El `Pump` recibe una configuración inicial desde variables de entorno.
Para configurarlo correctamente, se espera un archivo `.env` en el
directorio `pump/` con los siguientes datos:

```env
STATION_ADDRESS=... # Por ejemplo, localhost:8000
PING_RECEIVE_BASE_PORT=... # Por ejemplo, 9000
```

- **`STATION_ADDRESS`**: Dirección IP y puerto TCP de la estación a la que se
conectará.
- **`PING_RECEIVE_BASE_PORT`**: Puerto UDP base donde el pump escuchará los
pings de la estación. El puerto final se calcula como `PING_RECEIVE_BASE_PORT`
\+ id del `Pump`.

---

## Uso

```bash
cd pump
cargo run
```
