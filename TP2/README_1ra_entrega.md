### Miembros del grupo

Miembros del grupo:
1. Dídidmo Paez (padrón 98910).
2. Jonathan Dominguez (padrón 110057).
3. Martín Saavedra (padrón 109236).
4. Mateo Valentin Serrano Godoy (padrón 110912).


# Entidades del sistema


## Mensajes

- Pago.
- Resultado de verificación de pago.
- Aceptación de pagos pendientes.
- Forzamiento de actualización de pagos.
- Petición de lock.
- Confirmación de lock.
- Liberación de lock.
- Caída de nodo del clúster.
- Reconexión.
- Nueva conexión.
- Actualizar líder.


## Actores

1. Surtidor.
2. Estación.
3. Gerente de conexiones.
4. Conexiones. Tres tipos distintos.
5. Gerente de elección de líder.
6. Gerente de lock.
7. Pingers. Tres tipos distintos.
8. Gerente de transacciones.
9. Verificador de pagos.
10. Forzador de actualización de pagos.
11. Actualizador de topes de gasto.
12. Gerente de conexión con servidor.
13. Gerente de repositorio.
14. Escritor TCP.
15. Lector TCP.


## No actores

1. Tarjeta.
2. Pago.
3. Transacción.
4. Empresa.
5. Proxy.
6. Nodo.


## Desarrollo


### Mensajes

```Rust
pub struct Payment {
    card: Card,
    cost: u32, // ARS
}
```
```Rust
pub struct PaymentVerificationResult {
    result: bool,
}
```
```Rust
pub struct AcceptAllPendingPayments {}
```
```Rust
pub struct PaymentUpdateForce {
    card: Card,
    cost: u32, // ARS
}
```
```Rust
pub struct Lock {
    node_id: u32,
}
```
```Rust
pub struct Locked {
    node_id: u32,
}
```
```Rust
pub struct Unlock {
    node_id: u32,
}
```
```Rust
pub struct NodeFall {
    node_id: u32,
}
```
```Rust
pub struct Reconnected {}
```
```Rust
pub struct NewConnection {
    node_id: u32,
    node_address: SocketAddr,
}
```
```Rust
pub struct UpdateLeader {
    node_id: u32,
}
```
```Rust
pub struct RestartReceivingPings {}
```
```Rust
pub struct Ping {}
```
```Rust
pub struct UpdateProxyLeader {
    node_id: u32,
    socket: TcpStream,
}
```


### Actores


#### Surtidor

La entidad **surtidor** representa un surtidor físico real, a quien los conductores intentan pedir confirmación de pago con su tarjeta. Su función es recibir pedidos de pago y transmitirnos a la estación en la que se encuentra, y luego informar al cliente si el pago fue realizado o rechazado. Este es su estado interno:

```Rust
pub struct Pump {
    id: u32,
    station: Addr<Station>,
}
```

Mensajes que maneja:
1. Envía un PaymentVerification a su Station.
2. Recibe un PaymentVerificationResult. Dependiendo del resultado, informa al cliente de si el pago fue aceptado o rechazado.


#### Estación

La entidad **estación** representa una estación física real, donde los conductores pueden cargar combustible. Recibe pedidos de pago de sus surtidores, y solicita al cluster una verificación o rechazo del pago, para luego informar al surtidor correspondiente. Este es su estado interno:

```Rust
pub struct Station {
    id: u32,
    pumps: HashMap<u32, Addr<TcpWriter>>, // HashMap<id, writer>
    pending_payments: VecDeque<(u32, Payment)>, // VecDeque<(id, pay)>
    writer: Option<Addr<TcpWriter>>,
    reader: Option<Addr<TcpReader>>,
    ping_receiver: Addr<StationPingReceiver>,
}
```

##### Mensajes

Mensajes que maneja:
1. Recibe un Payment de cada Pump. Si está online, acola el pedido de verificación de pago y lo envía al TcpWriter del nodo, si la cola solo tiene ese elemento. Si está offline, acola el pedido de verificación de pago y envía un PaymentVerificationResult al TcpWriter del Pump con un estado aceptado.
2. Recibe un PaymentVerificationResult del TcpReader. Reenvía ese resultado al TcpWriter del Pump que lo pidió, descarta el top de su cola, y envía el siguiente Payment al TcpWriter del nodo, si la cola no quedó vacía.
3. Recibe un LostConnection de su StationPingReceiver cuando el nodo con el que está conectada está caído. Elimina su TcpWriter y TcpReader con el nodo, y se envía a sí misma un AcceptAllPendingPayments.
4. Recibe un AcceptAllPendingPayments de sí misma. Envía un PaymentVerificationResult a cada TcpWriter de Pump que pidió verificar un pago con un estado aceptado (no muta la cola de pedidos pendientes).
5. Recibe un Reconnected de su StationPingReceiver cuando recuperó la conexión. Reconstruye la conexión TCP con el nodo, levanta su TcpWriter y TcpReader con el nodo, y envía un PaymentUpdateForce al TcpWriter por cada pago pendiente que tuviera en su cola, y luego la vacía.


#### Gerente de conexiones.

La entidad **gerente de conexiones** se encarga de registrar las conexiones realizadas por y con el nodo, y redirigir los mensajes a los actores que fuera necesario si le llega uno. Este es su estado interno:

```Rust
pub struct ConnectionManager {
    leader_id: Option<u32>,
    node_id: u32,
    station_connections: Vec<Addr<StationConnection>>,
    internode_connections: Vec<Addr<InternodeConnection>>,
    proxy_connections: Vec<Addr<ProxyConnection>>,
    leader_election_manager = Addr<LeaderElectionManager>,
    lock_manager: Addr<LockManager>,
    stations_pinger: Addr<StationPinger>,
    internodes_pinger: Addr<InternodePinger>,
    proxies_pinger: Addr<ProxyPinger>,
    transaction_manager: Addr<TransactionManager>,
}
```

##### Mensajes

Mensajes que maneja:
1. Reenvía los PaymentVerification y PaymentUpdateForce que llegan de los StationConnection al TransactionManager.
2. Reenvía los UpdateMaxSpending que llegan de los ProxyConnection al TransactionManager.
3. Recibe los NodeFall que llegan del InternodePinger. Verifica si el nodo caído es el líder: si lo es, cambia a None el id del líder, y envía el mensaje LeaderFall al LockManager, al TransactionManager y al LeaderElectionManager; si no lo es, solo reenvía el mensaje NodeFall al LockManager y al TransactionManager. En ambos casos también elimina al actor que manejaba la conexión TCP con el nodo caído.
4. Recibe conexiones nuevas de sus tasks de escucha, cuando se le conecta una estación, proxy o nodo, y las almacena en su registro de conexiones TCP establecidas. Luego, si la conexión fue de un nodo, envía un NewConnection al InternodePinger, al TransactionManager y al LeaderElectionManager.
5. Recibe conexiones nuevas de su task de conexión con otros nodos. Las añade a su registro interno, y hace que los demás actores la registren: si es una conexión de nodo, envía NewConnection al InternodePinger, al TransactionManager y al LeaderElectionManager; si es una conexión de Proxy, envía NewProxy al ProxyPinger; si es una conexión de estación, envía NewStation al StationPinger.
6. Recibe un UpdateLeader del LeaderElectionManager. Actualiza el id del líder y reenvía el mensaje al LockManager, al TransactionManager, y a todas las ProxyConnections. Además, si el id del nuevo líder coincide con el id del nodo, envía StartPinging al ProxyPinger.
7. Reenvía los mensajes Lock y Unlock que llegan de los InternodeConnection al LockManager, y reenvía el mensaje Unlocked que llega del LockManager al InternodeConnection correspondiente.


#### Conexión con estación.

La entidad **conexión con estación** se crea cuando una estación se conecta a un nodo, y maneja el envío y recepción de mensajes. Este es su estado interno:

```Rust
pub struct StationConnection {
    id: u32,
    writer: Addr<TcpWriter>,
    connection_manager: Addr<ConnectionManager>,
}
```

Mensajes que maneja:
1. Reenvía todos los mensajes que recibe por socket TCP al ConnectionManager.
2. Reenvía todos los mensajes que el ConnectionManager le envía a su TcpWriter.

##### Mensajes

Mensajes que maneja:
1. Un Payment de la estación con la que está conectado. Se lo envía al ConnectionManager para que pueda redirigir la transacción, junto con su propio address para que puedan devolverle el resultado.
2. Un PaymentVerificationResult del ConnectionManager. Se lo envía a su TcpWriter.


#### Conexión con nodos.

La entidad **conexión con nodos** se crea cuando un nodo establece comunicación con otro, ya sea porque se le conectó o porque se le conectaron, y maneja el intercambio de mensajes con ese nodo. Este es su estado interno:

```Rust
pub struct InternodeConnection {
    id: u32,
    writer: Addr<TcpWriter>,
    connection_manager: Addr<ConnectionManager>,
}
```

##### Mensajes

Mensajes que maneja:
1. Reenvía todos los mensajes que le llegan del nodo al que está conectado al ConnectionManager.
2. Reenvía todos los mensajes que le envía el ConnectionManager al TcpWriter.


#### Gerente de elección de líder.

La entidad **gerente de elección de líder** maneja la transmisión de mensajes en el proceso de elección de líder por el algoritmo de Bully. Este es su estado interno:

```Rust
pub struct LeaderElectionManager {
    id: u32,
    leader_id: Option<u32>,
    socket: UdpSocket,
    peers: HashMap<u32, SocketAddr>,
    connection_manager: Addr<ConnectionManager>,
}
```

##### Mensajes

Mensajes que maneja:
1. Recibe NewConnection del ConnectionManager. Añade la nueva conexión a sus peers e inicia el proceso de elección de líder: levantará una task de envío y una de escucha, ambas por UDP; la lógica seguirá el algoritmo de Bully (se envía "E" a los nodos de id mayor, y todos los nodos que reciban ese mensaje enviarán "K", finalmente el líder será el de mayor id, quien no habrá podido enviar mensajes a nadie o no le habrán respondido, y se anunciará por la task de envío con un "C").
2. Recibe "C" de su task de escucha del nuevo nodo líder. Envía un UpdateLeader al ConnectionManager.


#### Gerente de lock.

La entidad de **gerente de lock** maneja el acceso al lock centralizado del líder durante las transacciones. Este es su estado interno:

```Rust
pub struct LockManager {
    pending: VecDeque<u32>, // Internode ids
    connection_manager: Addr<ConnectionManager>,
}
```

##### Mensajes
Mensajes que maneja:
1. Recibe un Lock del ConnectionManager. Añade a su lista de pedidos de lock al nodo que lo pidió; si la cola solo tiene esa petición, envía un Locked al ConnectionManager.
2. Recibe un Unlock del ConnectionManager. Remueve la petición de lock del tope de su lista; si la cola no quedó vacía, le envía un Locked al ConnectionManager.
3. Recibe un NodeFall del ConnectionManager. Elimina de su lista de petición de lock al nodo caído según el id del mensaje; si el nodo caído era el primer elemento de la lista, luego de eliminarlo envía un Lock al ConnectionManager con el siguiente nodo.


#### Pinger de estaciones.

La entidad **pinger de estación** se encarga de constantemente *pingear* a las estaciones conectadas al nodo. Este es su estado interno:

``` Rust
struct StationPinger {
    id: u32,
    peers: Vec<UdpSocket>,
    connection_manager: Addr<ConnectionManager>
}
```


#### Receptor de ping en estación.

La entidad **receptor de ping en estación** se encarga de recibir los pings que manda el nodo al que la estación se conectó. Este es su estado interno:

``` Rust
struct StationPingReceiver {
    node: UdpSocket,
}
```

##### Mensajes

Mensajes que maneja:
1. Cuando detecta que el nodo deja de hacer ping, envía el mensaje LostConnection a Station.
2. Cuando detecta que el nodo volvió a hacer ping, envía el mensaje Reconnected a Station.


#### Pinger de nodos.

La entidad **pinger de nodos** se encarga de constantemente *pingear* a los demás nodos del clúster y de recibir los pings de los demás nodos. Esto se hace para registrar los nodos vivos y saber cuándo alguno cae. Este es su estado interno:

``` Rust
struct InternodePinger {
    id: u32,
    socket: Rc<UdpSocket>,
    peers: HashMap<u32, State>,
    connection_manager: Addr<ConnectionManager>
}
struct State {
    udp_addr: SocketAddr,
    misses: u32,
}
```

Los peers tendrán asociados **estados**, que almacenan su dirección de socket y la cantidad de veces que no se registró un ping de ellos, así, si esa cantidad supera cierto umbral, se asume que el nodo cayó.


##### Mensajes

Mensajes que maneja:
1. Cuando detecta que un nodo dejó de hacerle ping, envía el mensaje NodeFall al ConnectionManager, y lo elimina de los peers.
2. Recibe el mensaje NewConnection del ConnectionManager, y añade al nuevo peer a su estado interno.


#### Pinger de proxies.

La entidad **pinger de proxies** se encarga de constantemente *pingear* a los proxies conectados al nodo. Esto solo lo hace el nodo líder, que es a quien los proxies se conectan. Este es su estado interno:

``` Rust
struct ProxyPinger {
    proxies: Vec<UdpSocket>,
    connection_manager: Addr<ConnectionManager>
}
```

##### Mensajes

Mensajes que maneja:
1. Recibe NewProxy del ConnectionManager. Añade al proxy a su estado interno, pero no comienza a pinguear aún.
2. Recibe StartPinging del ConnectionManager. Se envía a sí mismo el mensaje Ping.
3. Recibe Ping de sí mismo. Envía un ping a todos los proxies que registró.


#### Receptor de ping en proxies.

La entidad **receptor de ping en proxies** se encarga de recibir los pings que el nodo líder le manda al proxy. Este es su estado interno:

``` Rust
struct ProxyPingReceiver {
    active: bool,
    pinger: UdpSocket,
}
```

##### Mensajes

Mensajes que maneja:
1. Cuando deja de recibir pings, enviará LeaderFall al LeaderFindingManager para que busque al nuevo líder del clúster. Luego, cambiará su active a falso, y seguirá esperando pings para que no se acumulen.
2. Recibe RestartReceivingPings del LeaderFindingManager cuando se ha encontrado al nuevo líder. Cambiará su active a true, y si deja de recibir pings de nuevo entonces se reinicia la lógica de (1.).


#### Gerente de transacciones.

La entidad **gerente de transacciones** se encarga de registrar las transacciones pendientes e iniciarlas cuando es posible. No realiza las transacciones manualmente, sino que invoca a otros actores que se especializan en realizar cada tipo de transacción puntual. Este es su estado interno:

```Rust
pub struct TransactionManager {
    pending: Queue<(PeerConnection, Transaction)>,
    payment_verificator: Addr<PaymentVerificator>,
    payment_update_forcer: Addr<PaymentUpdateForcer>,
    max_spending_updater: Addr<MaxSpendingUpdater>,
    connection_manager: Addr<ConnectionManager>,
    repository_manager: Addr<RepositoryManager>,
}
```

Los mensajes que maneja este actor y los demás agentes dedicados de transacciones se desarrollan en la siguiente sección.


#### Verificador de pagos.

La entidad **verificador de pagos** maneja la lógica de transacción para las transacciones del tipo *verificación de pago*. Este es su estado interno:

```Rust
pub struct PaymentVerificator {
    expected_number_of_pieces: u32,
    spending_info_pieces: Vec<(u32, Option<SpendingInfo>)>,
    transaction_manager: Addr<TransactionManager>,
    connection_manager: Addr<ConnectionManager>,
}
```


#### Forzador de actualización de pagos.

La entidad **forzador de actualización de pagos** maneja la lógica de transacción para las transacciones del tipo *forzar actualización de pago*. Este es su estado interno:

```Rust
pub struct PaymentUpdateForcer {
    expected_number_of_pieces: u32,
    spending_info_pieces: Vec<(u32, Option<SpendingInfo>)>,
    transaction_manager: Addr<TransactionManager>,
    connection_manager: Addr<ConnectionManager>,
}
```


#### Actualizador de topes de gasto.

La entidad **actualizador de topes de gasto** maneja la lógica de transacción para las transacciones del tipo *actualizar tope de gasto de un empleado*. Este es su estado interno:

```Rust
pub struct MaxSpendingUpdater {
    expected_number_of_pieces: u32,
    max_spending_info_pieces: Vec<(u32, Option<MaxSpendingInfo>)>,
    transaction_manager: Addr<TransactionManager>,
    connection_manager: Addr<ConnectionManager>,
}
```


#### Gerente de búsqueda de líder.

La entidad **gerente de búsqueda de líder** permite que un proxy se conecte con el líder del clúster, y permite conectarse cuando hay un líder definido, y pedir que se le indique el nuevo líder cuando el anterior se cayó. Este es su estado interno:

```Rust
pub struct LeaderFindingManager {
    proxy_id: u32,
    nodes_addresses: HashMap<u32, SocketAddr>, // HashMap<node_id, address>
    ping_receiver: Addr<ProxyPingReceiver>,
    proxy: Addr<Proxy>,
}
```

##### Mensajes

Mensajes que maneja:
1. Recibe LeaderFall del ProxyPingReceiver cuando el líder del clúster cayó. Levanta una task para conectarse con los nodos del clúster y guarda las conexiones.
2. Cuando algún nodo le envía un mensaje diciéndole el id del nuevo líder, corta todos los sockets menos el del líder, y se lo envía al Proxy con el mensaje UpdateProxyLeader. Luego, envía el mensaje RestartReceivingPings al ProxyPingReceiver.


#### Gerente de repositorio.

La entidad **gerente de repositorio** operar sobre un repositorio de datos dentro del clúster. Este es su estado interno:

```Rust
pub struct RepositoryManager {
    data: HashMap<u32, (HashMap<...>, ...,)>, // mapa de tablas indexadas por id de empresa, al estilo SQL
    connection_manager: Addr<ConnectionManager>,
}
```

Permite hacer operaciones GET y UPDATE para los distintos tipos de transacciones.

##### Mensajes

Mensajes que maneja:
1. Recibe LeaderFall del ProxyPingReceiver cuando el líder del clúster cayó. Levanta una task para conectarse con los nodos del clúster y guarda las conexiones.
2. Cuando algún nodo le envía un mensaje diciéndole el id del nuevo líder, corta todos los sockets menos el del líder, y se lo envía al Proxy con el mensaje UpdateProxyLeader. Luego, envía el mensaje RestartReceivingPings al ProxyPingReceiver.


#### Escritor TCP.

La entidad *escritor TCP* maneja una *conexión TCP* con un peer, y permite enviar un mensaje a través de esa conexión. Este es su estado interno:

```Rust
pub struct TcpWriter {
    writer: Option<WriteHalf<TcpStream>>,
}
```

Reenvía todos los mensajes que recibe a través de su stream de escritura.


#### Lector TCP.

La entidad *lector TCP* maneja una *conexión TCP* con un peer, y permite recibir datos a través de esa conexión. Es solo usada por Station. Este es su estado interno:

```Rust
pub struct TcpReader {
    station: Addr<Station>,
}
```

Reenvía todos los datos recibidos a la Station en forma de mensajes.


### No actores


#### Tarjeta

La entidad **tarjeta** representa una tarjeta física real, usada por los conductores cuando quieren pagar para cargar combustible en un surtidor. Almacena solo datos estáticos, que no cambian en el tiempo; este es su estado interno:

```Rust
pub struct Card {
    id: u32,
    company_id: u32,
}
```


#### Pago

La entidad **pago** representa una solicitud de pago que efectúa un cliente con una tarjeta a un surtidor. Este es su estado interno:

```Rust
pub struct Payment {
    card: Card,
    cost: u32, // ARS
}
```


#### Transacción

La entidad **transacción** representa un procedimiento que realiza el clúster y es *triggereado* por un mensaje de una estación o proxy. Esta es su definición:

```Rust
pub enum Transaction {
    PaymentVerification(PaymentVerification),
    PaymentUpdateForce(PaymentUpdateForce),
    MaxSpendingUpdate(MaxSpendingUpdate),
}
```


#### Conexión con peer

La entidad **conexión con peer** representa una conexión con un peer. Esta es su definición:

```Rust
pub enum PeerConnection {
    StationConnection(Addr<StationConnection>),
    ProxyConnection(Addr<ProxyConnection>),
}
```


#### Empresa

La entidad **empresa** representa un punto de conexión con cada empresa suscrita al servicio de YPF Ruta. Una empresa puede pedirle al servicio datos en particular, o modificar el tope máximo permitido para algún empleado, y constantemente recibe información en vivo de las transacciones realizadas por sus empleados.
Una empresa no se comunica directamente con el clúster o ni siquiera con el líder del mismo, sino que invoca a un intermediario llamado **proxy**, que maneja la conexión, reconexión, y envío y recepción de mensajes con el cluster. De este modo, a los ojos de la empresa, el proxy es el líder del cluster, y para el cluster, el proxy es la empresa.

El proxy tiene el siguiente estado interno:

```Rust
pub struct Proxy {
    id: u32,
    writer: Addr<TcpWriter>,
    leader_finding_manager: Addr<LeaderFindingManager>,
    ping_receiver: Addr<ProxyPingReceiver>,
}
```

Cuando el LeaderFindingManager le envía LeaderFall al Proxy, éste droppea su writer y deja de recibir mensajes por socket. Cuando el mismo le envía UpdateProxyLeader, reabre la conexión y levanta el TcpWriter.


#### Nodo

La entidad **nodo** representa un nodo del clúster, que puede o bien ser un nodo físico real (estar ubicado en alguna parte del país) o un nodo lógico (uno de varios actores que corren en una misma computadora). Tiene múltiples responsabilidades:
1. Recibir pedidos de pago de estaciones y enviarles el resultado de la validación.
2. Actualizar su estado con pagos registrados por las estaciones cuando estaban offline.
3. Validar pedidos de pago a través de un algoritmo de transacciones con la estrategia de *mutex centralizado*.
4. Participar en el proceso de elección de líder a través del *algoritmo de Bully*.

Además, si el nodo es el líder del clúster, tiene algunas responsabilidades adicionales:
1. Recibir pedidos de datos o de actualización de los mismos de las empresas.
2. Notificar en tiempo real todos los pagos registrados a cada empresa.
3. Administrar el acceso a un lock centralizado.

Un nodo es una instancia del siguiente struct:

```Rust
pub struct Node {
    id: u32,
    client_port: u32,
    internode_port: u32,
    proxy_port: u32,
    cfg: Config,
    connection_manager: Addr<ConnectionManager>,
}
```

Aunque esta instancia no realiza acciones manualmente, sí mantendrá los demás actores vivos mientras viva. Estas son las funciones que cumplirá:

1. Recibir conexiones de las estaciones.
2. Manejar envío de mensajes con cada estación.
3. Encolar y realizar las transacciones pendientes.
5. Recibir las conexiones de otros nodos.
6. Manejar envío de mensajes con cada otro nodo del clúster.
7. Recibir las conexiones de las empresas.
8. Manejar el envío de mensajes con cada empresa.


## Establecimiento de conexiones TCP

Para establecer una conexión TCP, no se usarán actores, sino *tareas* de Actix, por simplicidad. Así, se correrán dentro del nodo una tarea que escuchará conexiones de estaciones, una para conexiones de otros nodos, y una para conexiones de proxies, y una tarea para conectarse a otros nodos, todo por TCP.


## Reconexión de un nodo

Cuando un nodo ingresa al clúster (ya sea por primera vez o porque se reconectó), empezará a *pingear* a todos los demás nodos, que conocerá a partir de su configuración  inicial (esto durante un periodo 'de descubrimiento', que duraría solo algunas iteraciones), luego se conectaría a los de id menor y escucharía conexiones de los nodos restantes. Finalmente, pasado el período de descubrimiento, solo se pingearía con los nodos con los que efectivamente pudo sostener una conexión TCP, ya sea porque pudo conectarse o porque se le conectaron.


## Transacciones

Manejamos tres tipos de transacciones:

1. Pedido de **validación de pago**. Una estación se comunica con algún nodo del clúster (el que le es más cercano físicamente) para pedir que verifique si un intento de pago efectuado por un cliente con su tarjeta es válido. A este tipo de transacción la llamamos PaymentVerification.
2. Pedido de **actualización de estado con pagos registrados en tiempo *offline***. Cuando una estación se queda *offline* (cuando el nodo con el que se comunica cae, y no tiene a quien pedirle verificaciones), no pide verificación de pedidos de pago, sino que los da a todos por válido y los registra para notificárselo al clúster cuando esté *online* otra vez; en ese caso, no es posible rechazar el pago, ya que ya fue realizado, así que se fuerza al clúster a actualizar su estado con estos datos. A este tipo de transacción la llamamos PaymentUpdateForce.
3. Pedido de **actualización de tope de consumo de un empleado**. Cuando un administrador de empresa quiere cambiar el tope máximo de consumo de un empleado. A estas las llamamos MaxSpendingUpdate.

Todas son bastante similares en cuanto a envío de mensajes, por lo que describiremos mayormente el caso de la primera, y repasaremos por encima el resto.

Dentro del nodo funcionarán actores que permitirán llevar a cabo la transacción:

1. El actor encargado de manejar el intercambio de mensajes con la estación, el StationConnection.
2. El actor encargado de administrar las conexiones del nodo, el ConectionManager.
3. El actor encargado de administrar las transacciones, el TransactionManager.
4. El actor encargado de realizar las transacciones PaymentVerification, el PaymentVerificator.
5. El actor encargado de realizar las transacciones PaymentUpdateForce, el PaymentUpdateForcer.
6. El actor encargado de enviar mensajes a otros nodos, el InternodeConnection.
7. El actor encargado de actualizar los datos dentro del nodo, el RepositoryManager.


### Descripción del flujo de transacción dentro de un nodo

Todos los tipos de transacciones se realizan utilizando la estrategia de **mutex distribuido**. Cuando se quiere realizar una transacción, se le pide al líder del clúster exclusividad para operar.

#### Flujo de una **PaymentVerification**

Un nodo recibe un pedido de pago de una estación a través de un StationConnection. Este mensaje es reenviado al ConnectionManager, quien luego lo vuelve a reenviar al TransactionManager, quien encola el pedido en una cola de transacciones. Hay dos circunstancias en las que el TransactionManager *triggerea* el inicio de la transacción en el PaymentVerificator:

1. Cuando el ConnectionManager le envía una transacción PaymentVerification  y el único elemento de la cola es esa transacción enviada.
2. Cuando el PaymentVerficator le envía un mensaje al TransactionManager indicándo que ya terminó la transacción que estaba haciendo y pidiéndole otra, si es que la cola no quedó vacía.

Los elementos de la cola del TransactionManager solo se *poppean* cuando el PaymentVerificator envía el mensaje de que terminó la transacción; hasta entonces, el elemento en el *top* de la cola es la transacción que se está realizando en ese momento.

Con esto, el TransactionManager puede devolverle el resultado al ConnectionManager, y este sabe a qué StationConnection debe notificar el resultado de la transacción.

Cuando el PaymentVerificator recibe el mensaje del TransactionManager, inicia la transacción pidiéndo exclusividad al líder. Dado que implementamos un sistema de ping constante y el líder permite exclusividad por orden de acceso, no corremos riesgo de starvation, y como el único recurso restringido es el líder, no hay deadlocks.
El PaymentVerificator envía el mensaje `Lock` al líder, y queda a la espera. El líder, eventualmente, responde con `Locked`.

Ante el mensaje Locked, el PaymentVerificator envía un mensaje a todos los nodos del clúster pidiéndole los datos necesarios para resolver la transacción, con esta forma:

```Rust
pub struct GetSpendingInfo {
    card_id: u32,
    company_id: u32,
}
```

Los nodos responderán con los datos correspondientes al *get*, de esta forma:

```Rust
pub struct SpendingInfo {
    employee_current_spending: f32, // ARS
    employee_max_spending: f32, // ARS
    company_current_spending: f32, // ARS,
    company_max_spending: f32, // ARS
}
```

Cuando recibe un mensaje de este tipo, incrementa un contador hasta la cantidad de nodos del cluster menos uno (porque ya tiene sus propios datos, naturalmente). Cuando recibe el último mensaje, comienza la verificación. En este punto, pueden suceder dos situaciones:

1. Todos los estados recibidos coinciden en que el pago es válido.
2. Alguno de los estados recibidos (o todos) marca que es inválido.

Si se da el primer caso, el PaymentVerificator actualizará los datos del nodo para reflejar la verificación exitosa, y le envía un mensaje a todos nodos del clúster para que actualizen su estado con las modificaciones del pago, con la siguiente forma:

```Rust
pub struct UpdateSpendingInfo {
    card_id: u32,
    company_id: u32,
    employee_current_spending: f32, // ARS
    company_current_spending: f32, // ARS
}
```

Si se da el segundo caso, el PaymentVerificator va a hacer una **reparación** de su estado interno, según si los datos recibidos son válidos (por ejemplo, los datos enviados por un nodo son vacíos, porque el nodo no procesó nunca un pago de ese empleado/empresa, por lo que no debería afectar al resultado final), y si los datos efectivamente confirman una incongruencia, actualizará su estado interno con las correcciones y enviará el mismo mensaje a los demás nodos.

Independientemente del caso, termina por enviar el mensaje `Unlock` al líder y un mensaje al TransactionManager avisándole que la verificación resultó válida y pidiéndole una nueva transacción. Luego, el TransactionManager le envía el resultado al ConnectionManager, y una nueva transacción se dispara si es posible.

#### Flujo de las otras transacciones

La lógica de las demás transacciones es igual, la única diferencia es que para las transacciones del tipo PaymentUpdateForce actúa el PaymentUpdateForcer y para las de tipo MaxSpendingUpdate actúa el MaxSpendingUpdater.

Por otro lado, ninguno de estos tipos de transacciones pueden rechazar la petición, aunque en ambas se realiza una reparación del estado interno del nodo si es necesario.


### Actualización del estado interno en cada nodo

Cuando un nodo recibe un UpdateSpendingInfo (el nodo que hace la transacción también se envía a sí mismo este mensaje), lo redirigirá al RepositoryManager. Para este proyecto, dada su simplicidad, el RepositoryManager manejará solo datos en RAM, aunque podría manejar una base de datos y la lógica sería similar.
Cuando el RepositoryManager recibe el update, registra esa actualización sin hacer verificaciones adicionales.


### Reacción ante caídas de nodos

El flujo descrito es relativamente simple, pero cuando se introduce el problema de caídas de otros nodos o del líder se vuelve más complejo.

Cuando un nodo cae, el Pinger que está dentro de cada nodo envía un mensaje al ConnectionManager, indicándole el id del nodo caído. Hay tres escenarios:
1. Se cae un nodo que no es el líder ni quien tomó el lock del mismo.
2. Se cae el líder.
3. Se cae quien tomó el lock.

Si quien cae es un nodo que no es el líder ni quien tomó el lock del líder, el ConnectionManager enviará un mensaje al TransactionManager, que luego enviará uno a cada manejador de transacción de su estado para indicarles que no lo tengan en cuenta para realizar la transacción.
Si quien cayó fue el líder, el TransactionManager les enviará un mensaje para decirles que hagan un *rollback* de la transacción (es decir, que cancelen lo que estaban haciendo hasta que se reelija el líder); cuando el nuevo líder sea elegido, el LeaderElectionManager se lo indicará al ConnectionManager, quien luego avisa al TransactionManager, y entonces enviará el pedido de transacción en el top de su cola (con este enfoque, si el líder cae, no se preserva el orden de peticiones de lock necesariamente, pero sigue garantizando ausencia de starvation y deadlock).
Si quien cae es quien tomó el lock, todos los otros nodos lo sabrán en cuanto el Pinger le envíe el mensaje al ConnectionManager; el líder, en particular, eliminará de su lista de pedidos de lock la petición del nodo caído (esto lo hace aunque el nodo caído no sea quien tenía el lock) usando su LockManager, y le avisará al siguiente nodo en la lista que le fue concedido el lock.

Cuando el TransactionManager envía el mensaje de que un nodo (que no es el líder) cayó durante la transacción, los manejadores de transacción mutarán su estado interno para evitar esperar por la respuesta del nodo caído, si es que la estaban esperando, o descartarán los datos obtenidos si ya los habían recibido. Seguidamente, ellos mismos se enviarán un mensaje para indicarse que evalúen si, habiendo descartado ese nodo, pueden seguir con la transacción. Si ya hubieran recibido todos los datos necesarios antes de que el TransactionManager les avisara que cayó el nodo, entonces simplemente usarán esos datos y seguirán con la transacción normalmente.

Si el TransactionManager les envía el mensaje de que el nodo caído fue el líder, entonces hacemos la misma distinción: si sucedió antes de terminar de recibir los datos de todos los nodos, eliminarán al líder de sus 'nodos esperados' y borrarán todos los datos recibidos; si sucedió luego de recibir todos los datos, la transacción se llevará a cabo con normalidad.
