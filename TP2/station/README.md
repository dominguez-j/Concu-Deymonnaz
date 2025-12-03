# Station Service

El servicio `Station` representa una estación de servicio YPF. Actúa como un
intermediario entre los surtidores (`Pumps`) y el clúster, gestionando la
autorización de pagos y manteniendo la operatividad  incluso en situaciones de
desconexión temporal.

## Actores

El servicio se compone de los siguientes actores principales:

### Station

El **actor principal** que orquesta toda la lógica de la estación. Maneja los
mensajes de los `Pumps`, se comunica con el clúster y coordina a los demás actores.

#### Estado Interno

```rust
pub struct Station {
   id: u32,
   counter: u32,
   heartbeat_manager: Addr<HeartbeatManager<Self>>,
   connection_establisher: Addr<ConnectionEstablisher>,
   _pump_connection_listener: PumpConnectionListener,
   heartbeat_sender: Addr<HeartbeatSender>,
   repository_manager: Addr<RepositoryManager>,
   pumps_protocols: HashMap<u32, TcpProtocol<Self, Payment>>,
   node_protocol: Option<TcpProtocol<Self, ProtocolMessage>>,
}
```

#### Mensajes que Maneja

La `Station` se comunica por TCP con dos tipos de actores distintos: los `Pump`
y los nodos del clúster.

- **Mensajes con `Pump`**:

1. `Payment`: 
   - Si está online, guarda el pedido de verificación de pago y lo envía al cluster 
   con un `TcpProtocol`.
   - Si está offline, guarda el pedido y envía un `TransactionResult` al pump con estado
   **aceptado** (como política, siempre se aceptan los pagos cuando se está offline).

2. `TransactionResult`:
   - Cuando el clúster le envía el resultado de un pedido de pago a la `Station`,
   o cuando se entra en modo offline, se envían `TransactionResult` a los `Pump`
   que hubieran pedido la verificación del pedido de pago.
   - En el caso de que la `Station` se vaya offline, se envía un resultado de
   **aceptado** por todas las transacciones registradas hasta el momento a los
   `Pump`, y se dejan de enviar pedidos al clúster hasta que se haya reconectado.

- **Mensajes con el clúster**:

1. `Transaction`:
   - El clúster acepta transacciones envueltas en el tipo `Transaction` de su
   protocolo general. La `Station` envuelve los `Payment` en este tipo y las
   envía al clúster si está online.
   - Si el nodo del clúster al que la `Station` se conecta se levanta luego
   de haberse caído, la `Sation` le enviará todos los pedidos de pagos guardados
   mientras estuvo offline en modo `ForcePayment`, ya que no pueden ser rechazados.

2. `TransactionResult`: 
   - Recibe los resultados de las verificaciones de pago no forzadas.
   - Busca el pedido de verificación de pago en su repositorio y lo elimina.
   - Envía el resultado al `Pump`.

3. `FirstMsg`:
   - Recibe un mensaje de incialización del clúster. Para `Station`, no hace nada.

- **Mensajes que maneja con el `RepositoryManager`**:

El `RepositoryManager` es un actor que se encarga del almacenamiento de los
`Payment` que le llegan a la `Station`. Si bien para `Station` el modo de
almacenamiento es irrelevante, para esta implementación se decidió persistir
los `Payment` en disco, por lo que `RepositoryManager` trabaja sobre un archivo
de `tokio` para lectura y escritura asincrónica.

`Station` maneja los siguientes mensajes con el `RepositoryManager`:

1. `Payment`:
   - Envía los `Paymet` que le llegan al `RepositoryManager`, previamente 
   inyectándoles su propio ID.

2. `Remove`:
   - Si llega un resultado de transacción del clúster, `Station` envía `Remove`
   al `RepositoryManager` para que elimine la transacción.

3. `GetAll`:
   - La `Station` le puede pedir al `RepositoryManager` todos los `Payment`
   que tiene guardados. Esto lo hace cuando se cae el nodo del clúster al que
   se conecta, para enviar los resultados como aceptados al `Pump` correspondiente,
   y cuando el nodo se reconecta, para enviarle los `ForcePayment`.

4. `Clear`:
   - La `Station` le envía este mensaje al `RepositoryManager` cuando tiene que limpiar
   todos los `Payment` guardados. Esto solo lo hace cuando el nodo se reconecta.

5. `SavedPayments`:
   - Es la respuesta a `GetAll`. Incluye un vector con todos los `Payment`
   guardados.
   - Cuando se recibe, `Station` enviará los resultados como aceptados a los `Pump`
   o enviará los `ForcePayment` al nodo, dependiendo de si está online u offline.

- **Otros mensajes manejados**:

1.`ConnectionLost`: 
   - Lo envía el `HeartbeatManager` cuando se cae el nodo del clúster al que se conecta
   la `Station`.
   - Se elimina el `TcpProtocol` con el nodo.
   - Se le pide al `RepositoryManager` todos los `Payment` guardados.
   - Se levanta el `ConnectionEstablisher` para buscar reconectarse.

2.`ConnectionEstablished`: 
   - Lo envía el `ConnectionEstablisher` cuando logra conectarse al nodo del clúster.
   - Se construye el `TcpProtocol` con el nodo.
   - Se levanta el `HeartbeatManager`.
   - Se le pide al `RepositoryManager` todos sus `Payment` y luego se lo limpia.

3. `Deploy`:
   - Lo envía al `ConnectionEstablisher` cuando pierde la conexión con el nodo.
   - Lo envía al `HeartbeatManager` cuando se conecta al nodo.

4`PumpConnected`:
   - Lo envía el `PumpConnectionListener` cuando un `Pump` se conecta.
   - Se crea un nuevo `TcpProtocol` para ese `Pump`.
   - Se le envía un `SetPumpId`.
   - Se almacena el protocolo.
   - Se registra en el `HeartbeatSender` al nuevo `Pump` para empezar a enviarle,
   heartbeats, enviándole el mensaje `RegisterHeartbeat`.

---

### ConnectionEstablisher

El **establecedor de conexión** es el actor encargado de establecer la conexión TCP
con el nodo del clúster.

#### Estado interno

```rust
pub struct ConnectionEstablisher {
    station: Addr<Station>,
    socket_addr: String,
    connect_interval: Option<SpawnHandle>,
}
```

#### Funcionamiento

- Solo intenta conectarse cuando `Station` le envía el mensaje `Deploy`.
- En intervalos de tiempo intenta conectarse al nodo una vez.
- Si la conexión falla, reintenta luego del intervalo.
- Una vez establecida la conexión, se notifica a sí mismo con el mensaje
`ConnectionEstablished`.
- Cuando le llega su propia notificación, cierra el intervalo de conexión y
notifica a `Station` con el mismo mensaje.


---

### PumpConnectionListener

El **escuchador de conexiones de pumps** acepta nuevas conexiones TCP entrantes
desde los `Pump`. No es un actor, sino un *struct* que maneja una tarea asincrónica,
por lo que `Station` nunca invoca sus métodos o le envía mensajes realmente.

#### Estado interno

```rust
pub struct PumpConnectionListener {
    _task: JoinHandle<()>,
}
```

#### Funcionamiento

- Escucha conexiones TCP en un puerto dado.
- Cuando un pump se conecta, acepta la conexión.
- Notifica a `Station` con el mensaje `PumpConnected`.

---

### HeartbeatSender

El **enviador de heartbeats** envía heartbeats vía UDP a los pumps conectados
para que se den cuenta cuando la `Station` cae.

#### Estado interno

```rust
pub struct HeartbeatSender {
   id: u32,
   base_port: u32,
   udp_io: Option<Addr<UdpIO>>,
   addresses: Vec<SocketAddr>,
}
```

#### Funcionamiento

- Envía a cada `Pump` registrado un mensaje por UDP a su puerto de escucha,
que se calcula como `base_port` = `Pump` ID.
- Envía los heartbeats en un intervalo de tiempo.
- Permite registrar nuevos `Pump` cuando le llega el mensaje `RegisterHeartbeat`.

---

### HeartbeatManager

El **gerente de heartbeats** gestiona la recepción de heartbeats del clúster para
detectar caídas.

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
   recipient: Addr<T>, // En este caso Addr<Station>
   ping_receive_interval: Option<SpawnHandle>,
   last_ping_time: Instant,
}
```

#### Funcionamiento

- Escucha heartbeats UDP del clúster.
- Cada cierto tiempo, igual para todos los `HeartbeatManager`, verifica el
  instante de recepción del último heartbeat.
- Si el tiempo desde el último heartbeat es mayor a cierto umbral, se da por
  caído el nodo del clúster y se envía `ConnectionLost` a la `Station`.
- Si el tiempo es menor al umbral, no se hace nada.

---

### RepositoryManager

El **gerente de repositorio** administra el almacenamiento local de transacciones,
permitiendo la persistencia cuando se entra en modo offline y si se reinicia
la `Station` durante ese tiempo.

#### Estado interno

```rust
pub struct RepositoryManager {
    station: Addr<Station>,
    repository_name: String,
    repository: Option<File>, // En modo escritura
}
```

#### Funcionamiento

- Guarda los pagos pendientes en un archivo local. Esto lo hace  cuando le
llega el mensaje `Payment`.
- Permite recuperar el estado de la estación al reiniciar. Esto se hace
a través del mensaje `GetAll`.
- Permite remover y limpiar todos los datos, con los mensajes `Remove` y
`Clear`.

---

## Funcionamiento de la `Station`

La estación opera como un nexo entre `Pump` y clúster:

### Conexión con Pumps

- Acepta conexiones de múltiples pumps.
- Asigna IDs únicos a cada pump.
- Procesa solicitudes de pago de forma concurrente.

### Conexión con Clúster

- Se conecta a un nodo del clúster para validar transacciones.
- Mantiene un registro de pagos pendientes.
- Si el nodo cae, los pagos se almacenan hasta que se reconecte.
- Cuando el nodo se reconecta, se envían todos los pagos guardados.
- Si `Station` se reinicia, los datos siguen persistidos en disco.
La excepción a esto es que mientras `Station` está caída, los pedidos
realizados por los `Pump` se pierden en ese intervalo.
- Si `Station` está offline, los pedidos de pago se almacenan y se
responden automáticamente como **aceptado**.


---

## Configuración

La `Station` recibe una configuración inicial por variables de entorno.

Para configurarla correctamente, se espera un archivo `.env` en el directorio
`station/` con los siguientes datos.

```env
STATION_ID=... # Por ejemplo, 1
REPOSITORY_NAME=... # Por ejemplo, payments.jsonl
STATION_UDP_LISTENING_PORT=... # Por ejemplo, 8000
PUMPS_UDP_LISTENING_PORT=... # Por ejemplo, 11000
CLUSTER_ADDRESS=... # Por ejemplo, localhost:7001
HOSTNAME=... # Por ejemplo, localhost
BASE_PORT=... # Por ejemplo, 10000
```

- **`STATION_ID`**: ID de la estación.
- **`REPOSITORY_NAME`**: Nombre del archivo para la persistencia local de datos.
Generalmente un JSONL.
- **`STATION_UDP_LISTENING_PORT`**: Puerto UDP donde la estación escucha heartbeats
del clúster.
- **`PUMPS_UDP_LISTENING_PORT`**: Puerto UDP base donde la estación envía heartbeats
a los pumps.
- **`CLUSTER_ADDRESS`**: Dirección IP y puerto TCP del nodo del clúster al que se conectará.
- **`HOSTNAME`**: Dirección IP o hostname donde la estación escuchará conexiones de pumps.
- **`BASE_PORT`**: Puerto TCP base para escuchar conexiones de pumps.

---

## Uso

```bash
cd station
cargo run
```
