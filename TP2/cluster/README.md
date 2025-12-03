# Cluster Service

El servicio `cluster` representa el compendio de nodos, que juntos conformarán básicamente la base de datos distribuida.
Cada nodo es responsable de mantener una réplica de los datos, participar en el consenso para las transacciones y
coordinarse con otros nodos para asegurar la consistencia y disponibilidad del sistema.

La idea de iplementar un cluster sería el garanrizar que los datos de las empresas y sus respectivas tarjetas se mantengan
consistentes y actualizados en tiempo real, con lo cual se tendrá una interacción constante entre las ordenes enviadas por
parte de los administradores de empresas (set, add, sub) ya sea para actualizar los límites de cada uno de estos elementos
o simplemente generando requests para poder saber cuál es el estado actual de cada uno de ellos (límite de tarjetas,
límites de empresas, consumo actual específico para una tarjeta o consumo global para la empresa). A su vez, el cluster
interactuará con las estaciones de la red de YPF recibiendo distintas transacciones por cada consumo que quiera realizar
una tarjeta (conductor) en un cierto pump de estación específico, con lo cual se tendrá que el cluster se encargará de
realizar la respectiva verificación para cada una de dichas operaciones y emitir un resultado de operación o no de la
transacción.

Se considera que es una manera adecuada para llevar a cabo la solución al problema planteado, ya que al tener un cluster
distribuido la información no se perderá en caso de que algún nodo llegase a caerse (resiliencia a fallos), pero además
es funcional en el caso de que cada nodo puede estar distribuido de forma conveniente por ubicación geográfica, facilitando
el acceso a los datos desde las distintas estaciones.


## Actores

### ServerPeer:
```rust
pub struct ServerPeer {
    pub(crate) id: u32,
    pub(crate) role: Option<Role>,
    pub(crate) writer: Addr<TcpWriter>,
    pub(crate) connection_manager: Addr<ConnectionManager>,
    pub(crate) transaction_manager: Addr<TransactionManager>,
}
```
Este es el actor que maneja las conexiones con los distintos tipos de clientes que se conectan con el respectivo nodo,
de esta forma se plantea un distinto tipo de "Role" para cada distinto tipo de conexión que se podrá realizar, de esta
forma se tendrán:

``` Rust
pub enum Role {
    Internode,
    Station,
    Proxy,
}
```
Cada uno de estos tipos caracteriza el tipo de ServerPeer que se tendrá y por ende la distinta funcionalidad que manejará.

Cada vez que se genera una conexión TCP entre alguna entidad de este tipo de roles y el respectivo nodo, lo que ocurrirá
será que se instanciará un nuevo SrverPeer con conexión/comunicación directa con tal entidad.

Por medio del ServerPeer se enviarán las requests internodos, se obtendrán sus respuestas, se recibirán las requests desde
los clientes (estaciones y proxies de empresas), pedido de locks y respuesta a los mismos, etc.

Es en el ServerPeer en donde confluye toda la recepción de mensajes enviados por TCP desde y hacia los distintos clientes
y mensajes internodos, es en su Handler **StreamHandler** en donde se producirá toda el manejo e interpretación de los
mensajes recibidos según el protocolo inscripto en el enum **ProtocolMessage**, generando toda la multiplexación de los
mismo hacia los respectivos actores a quienes les compete dicho mensaje. Por ejemplo se realizan acciones como las
siguientes:

1. Recibe el mensaje `StartUp` el cual es el Handshake que se realiza con cada una de las entidades que quieren generar
   conexión con el cluster, identificando de esta manera qué tipo de ServerPeer deberá de ser asignado para que meneje
   dicha conexión.
2. Reenvía las transacciones que llegan de las estaciones (`Station`) al `TransactionManager`.
3. Reenvía las actualizaciones que llegan de los proxies (`Proxy`) al `TransactionManager`.
4. Recibe/envía las requests internodos en el caso de que la información necesitada no se encuentre en el nodo actual.


El nodo se compone de varios actores (Managers) que manejan responsabilidades específicas:

### ConnectionManager

El **gerente de conexiones** se encarga de registrar las conexiones realizadas por y con el nodo, de esta forma tendrá
conocimiento de cada uno de los tipos de conexiones ya sea con los clientes (estaciones y proxies) o de las comunicaciones
internas (ServerPees de tipo Internode), también actúa como moderador para el manejo de funciones importantes como el
Heartbeat gestionado por el HeartbeatManager, el cual le avisa al ConnectionManager cuándo se detecta un nodo "Down", o
cuándo por medio de un discovery inicial se detecta que hay un nuevo nodo en el cluster, con lo cual el ConnectionManager
informará a las entidades específicas para que tomen decisiones con base en esto, y saque o ingrese a nuevas entidades a
su estado interno. Tiene relación directa con otras entidades vitales para el funcionamiento del cluster, como son el
ElectionManger y el TransactionManager, con el primero se vincula enviandole la orden de que se genere una nueva llamada
a elección "CallElection", en el caso de que se haya detectado un nodo "Down" y este haya sido el líder, entonces el
ConnectionManager será el encargado de hacer el llamado a elección enviando la orden al ElectionManager para que realice
directamente el CallElection; por el lado del TransactionManager, tiene relación directa con este manteniéndolo informado
de con qué entidades que le competen se sigue teniendo conexión activa, ya sea caía o nueva actividad con ServerPeers de
tipo Internode, cuando se genera una nueva conexión con los ServerPeers de tipo Proxy o Station, cuándo hay un nuevo
líder electo o cuándo cae un nodo.


#### Estado Interno

```rust
pub struct ConnectionManager {
    pub id: u32,
    pub leader: Option<u32>,
    pub cfg: Config,
    pub hb: Option<Addr<HeartbeatManager>>,
    pub em: Option<Addr<ElectionManager>>,
    pub tm: Option<Addr<TransactionManager>>,
    pub active_peers: HashMap<u32, Addr<ServerPeer>>,
    pub active_stations: HashMap<u32, Addr<ServerPeer>>,
    pub active_proxies: HashMap<u32, Addr<ServerPeer>>,
}
```

#### Mensajes que Maneja

1. Envía y actualiza información de conexiones al `TransactionManager` y al `ElectionManager`
3. Recibe notificaciones de caída de nodos del `HeartbeatManager`.
4. Recibe conexiones nuevas de las tareas de escucha TCP cuando se conecta una estación, proxy o nodo, y las almacena
   en su registro de conexiones.
5. Recibe actualizaciones de líder del `ElectionManager` y actualiza su estado interno, reenviando la información al
   `TransactionManager`.

---


### TransactionManager

Coordina las transacciones distribuidas utilizando un algoritmo de consenso basado en locks centralizados. Maneja las
peticiones de lectura y escritura en la "base de datos" a la cual denominamos `Repository`, asegurando la consistencia
de los datos a través del cluster mediante el mecanismo de que si la data que existe en cada nodo es correcta o de lo
contrario simplemente no existe información al respecto.

#### Estado Interno

```rust
pub struct TransactionManager {
pub(crate) id: u32,
pub(crate) leader: Option<u32>,
pub(crate) repository_manager: Addr<RepositoryManager>,
pub(crate) selects: HashMap<String, Transaction>,
pub(crate) updates: HashMap<String, Transaction>,
pub(crate) select_counter: u32,
pub(crate) update_counter: u32,
pub(crate) lock_manager: LockManager,
pub(crate) conn_manager: Addr<ConnectionManager>,
pub(crate) active_peers: HashMap<u32, Addr<ServerPeer>>,
}
```

#### Funcionamiento

El `TransactionManager` procesa dos tipos principales de transacciones:

1. Updates, `Update`, los cuales a su vez tienen distintas vertientes:
    - `EnterpriseLimitUpdate`: El cual es una orden de actualización del límite para una empresa específica (proveniente
      del proxy de una empresa).
    - `CardLimitUpdate`: El cual es una orden de actualización del límite para una tarjeta de una empresa específica
      (proveniente del proxy de una empresa).
    - `Payment`: Es el anuncio de la realización del pago, ya sea del tipo `PaymentVerification` o `ForcePayment`, en
      donde el primero es una transacción "normal" en curso, que espera verificación/confirmación para la realización de
      dicha operación por parte del cluster; y la segunda es una orden de escritura de datos directa al clustes, dicho
      caso ocurre únicamente cuando la estación haya tenido que registrar cobros de tarjeta de manera offline, en esta
      eventualidad la transacción al no poder verificarse directamente con los datos del cluster, se procede entonces a
      ejecutarla y a registrarla directamente en la base de datos distribuida. Esto es solamente llevado a cabo por parte
      de las distintas estaciones.

2. Selects, `Select`, en general se tienen de dos tipos, y ambos realizados por parte de la empresa: `CardView` y
   `EnterpriseView`, con los cuales lo que se busca es obtener el dato específico tanto de límites como de consumo actual
   por parte de una tarjeta específica de una empresa, o de una empresa como tal en el segundo caso.

---

### ElectionManager

Implementa el algoritmo de Bully para elegir un líder entre los nodos del cluster.

#### Estado Interno

```rust
pub struct ElectionManager {
    pub(crate) id: u32,
    pub(crate) leader: Option<u32>,
    pub(crate) election_status: ElectionStatus,
    pub(crate) round: u64,
    pub(crate) udp_io: Addr<UdpIO>,
    pub(crate) cm: Addr<ConnectionManager>,
    pub(crate) peers: HashMap<u32, SocketAddr>,
}
```

#### Funcionamiento

- Cuando se detecta la caída del líder o cuando un cierto nodo ingresa al cluster se inicia un proceso de elección.
- Informará al ConnectionManager siempre que se haya terminado una elección y se haya elegido a un nuevo líder.
- Recibirá información por parte del ConnectionManager para la iniciación de una nueva elección de líder por medio del
  mensaje `CallElection`

La operación de elección de líder se llevará a cabo por medio de mensajes UDP.

---

### HeartbeatManager

Monitorea el estado de los nodos del cluster mediante el envío periódico de mensajes UDP.

#### Estado Interno

```rust
pub struct HeartbeatManager {
    pub(crate) id: u32,
    cfg: Config,
    discovery_rounds_left: u32,
    seed_peers: Vec<SocketAddr>,
    udp_io: Addr<UdpIO>,
    pub(crate) peers: HashMap<u32, PeerState>,
    pub(crate) stations: HashMap<u32, SocketAddr>,
    pub(crate) proxies: HashMap<u32, SocketAddr>,
    pub(crate) cm: Addr<ConnectionManager>,
}

pub struct PeerState {
    pub(crate) last_seen: Instant,
    pub(crate) misses: u32,
    pub(crate) udp_addr: SocketAddr,
}
```

#### Funcionamiento

- Envía heartbeats periódicos a todos los nodos conocidos.
- Escucha heartbeats de otros nodos.
- Incrementa un contador de "misses" siempre que envía un heartbeat a otro nodo, y lo descuenta siempre que reciba una
  respuesta por parte de dicho nodo.
- Se contempla un threshold que será el umbral para saber cuántos misses permintidos se contemplan antes de considerar a
  un nodo como "Down" (`HB_MISSES_ALLOWED`), en cuyo caso notifica al `ConnectionManager` que el nodo ha caído.
- Cuando un nodo vuelve a enviar heartbeats reconoce su actividad, y de no llegar a tenerlo dentro de su lista de
  active_peers lo notificará al ConnectionManager, el cual si se trata de un nodo con Id menor intentará generar conexión
  TCP con el mismo y lo ingresará en el HashMap de active_peers

---

### RepositoryManager

Administra el almacenamiento local de datos en memoria.

#### Estado Interno

```rust
pub struct RepositoryManager {
    pub enterprises: HashMap<u32, Addr<Repository>>, 
    pub transaction_manager: Option<Addr<TransactionManager>>,
}

```

#### Funcionamiento

- Ejecuta operaciones de lectura (SELECT) para consultar datos.
- Ejecuta operaciones de escritura (UPDATE) para modificar datos.
- Mantiene la consistencia local de los datos según las instrucciones del `TransactionManager`.


## Otras estructuras importantes:

### LockManager

Administra el acceso exclusivo a los recursos durante las transacciones, implementando un lock centralizado en el nodo líder.

#### Estado Interno

```rust
pub struct LockManager {
    pub pending: VecDeque<u32>, // IDs de nodos esperando el lock
}
```

### Repository

Funciona como una especie de base de datos que almacena la información de cada una de las empresas que se encuentra haciendo
uso del servicio del programa.

```rust
pub struct Repository {
id: u32,
current_usage: u32,
limit: Option<u32>,
cards: HashMap<u32, SpendingInfoRegister>,
repository_manager: Addr<RepositoryManager>,
}
```


## Transacciones Distribuidas

### Flujo de una Transacción de Verificación de Pago


En general el siguiente es el orden de mensajes para una transacción de tipo update proveniente de una estación (sin
embargo el flujo es muy similar para los uptades realizados desde la empresa e incluso para los distintos selects,
exceptuando el pedido del lock y el almacenamiento en el Repository)

![Update Transaction Flow](https://imgur.com/TpJr8gd.png)

1. El determinado ServerPeer en el cluster recibe la request y decodifica de qué tipo de request se trata
2. El TransactionManager almacena la nueva transacción, pide el lock al lider de ser necesario (sólo para operaciones
   "Update", pues con "Select" no sería necesario tener un lock), y cuando obtiene el lock la resuelve
3. Pedido del lock al líder para que registre la transacción en la respectiva queue del líder para  los locks de las
   empresas y lo otorgue cuando corresponda.
   Si la Request original es un Select, NO hace falta que se pida un lock, simplemente con devolver la respuesta con la
   base de datos propia sería correcto, o de no haber datos relacionados en el Repository propio, se resolverá con la
   respuesta del primer nodo que haya respondido a la Request Interna de dicho Select.
4. Lock otorgado
5. Se verificará si hay datos disponibles para verificar la respectiva transacción, de no tenerlos localmente se deberá
   de hacer un broadcast a todos los nodos realizando el Select de dicha data (InterSelect), se utilizará la primer
   respuesta obtenida, el resto será descartado.

5.1. En el caso de que la operación sea un update y se tengan los datos de manera local para llevar a cabo la operación
se pueden tener alguno de los siguientes casos: `EnterpriseLimitUpdate`, `CardLimitUpdate` o `UpdatePayment`, cada
uno de los cuales generará una modificación distinta en el Repository, sin embargo, dicha modificación aún no es
realizada, pues primero se almacena dicha información en una estructura `SetOwnData` que contendrá los datos del
con la información actualizada y que luego será confirmada para escritura pero una vez que se pueda llevar a cabo
el broadcast hacia el resto de los nodos del cluster confirmando los cambios realizados y verificando que el líder
actual sea el mismo que el líder cuando se inició dicha transacción, en este punto se realizará la modificación de
manera simultánea tanto en el el propio nodo como en el cluster en general

5.1 De no llegar a tener la información en el Repository local, lo que se hará será generar una requeset internodo para
poder llevar a cabo la validación de la transacción con los datos otorgados por algún otro nodo del cluster, esto
estará dado por el mensaje `RequestData` y la respectiva respuesta desde los otros nodos será dado por el mensaje
`InternodeResponse`
5.2 En este punto se anuncian los datos de la respectiva actualización al `TransactionManager` por medio del mensaje
`SendBroadcastOfSetOwnData`, verificándose el tipo de información a devolver y a qué cliente (estación o proxy, por
medio del mensaje `TransactionResponse`), y si en definitiva se debe de confirmar la escritura de datos tanto local
como el envío de dicha orden en broadcast para que el resto de nodos hagan lo propio.
6. Es el mensaje a partir del cual se informa la orden de sobre escritura de los datos en el Repository local
7. Una vez que se haya finalizado la transacción se generará la respectivo `ReleaseLock`, anunciándole al nodo líder que
   puede otorgar el lock a otra transacción de la misma empresa.


### Manejo de locks:
La estrategia que se lleva a cabo para garantizar la concurrencia y consistencia de datos del cluster para las operaciones
de los distintos clientes del cluster será por medio del uso de **N** locks manejados de manera centralizada por el nodo
líder, donde **N** es la cantidad de empresas que estarán haciendo uso del servicio del cluster.

Siempre que se inicia una nueva transacción en el cluster para una determinada tarjeta y/o empresa, y si esta es de tipo
Update, la misma deberá de ser informada al nodo líder para que este la almacene en la respectiva cola que le correspode
a tal empresa, con lo cual, será la transacción que se encuentra en el front de dicha cola la que le corresponda el lock
en dicho momento, mientras las otras transacciones para la misma empresa quedan encoladas en estado pendiente.

Una vez que termina una transacción en el nodo de "origen" (el que recibió inicialmente dicha operación), este deberá de
enviar el mensaje del respectivo `ReleaseLock` al líder para que este pueda sacar de la cola de **pendings** a tal
transacción y otorgar el lock a la siguiente transacción.

Básicamente la idea de tal operatoria es la representada en las siguientes imagenes:

#### Estado inicial (llegada de transacciones a los distintos nodos):
![Imgur](https://imgur.com/C7bNyWy.png)

#### Petición de Locks:
![Imgur](https://imgur.com/vxbY1mt.png)

En la anterior imagen se observa que todas las transaciones de tipo Update son encoladas en la respectiva cola que le
corresponde a cada empresa en el estado interno del líder. También se tiene que son las transacciones "C", "A" y "D",
las que tienen acceso al lock y por ende se estarán llevando a cabo en ese momento, y serán los respectivos nodos
"propietarios" de dichas transacciones los que serán anunciados para que lleven a cabo la operatoria de dichas
transacciones.


### Manejo de Caídas de Nodos

#### Caída de un Nodo Regular

- El `HeartbeatManager` detecta la caída y notifica al `ConnectionManager`.
- El `TransactionManager`, `HeartbeatManager` y `ConnectionManager` eliminan el nodo de sus peers activos.
- Las transacciones en curso continúan sin problema sin esperar respuesta del nodo caído.
- Si el nodo caído es el dueño de la transacción, la estación se dará cuenta de que dicha transacción aún no ha recibido
  respuesta y que dicho nodo caído es el que la estaba gestionando, con lo cual la estación la sigue manteniendo almacenada
  resolviendola de manera offline, y una vez que el nodo vuelva a estar activo, dichas transacciones serán enviadas
  pero esta vez siendo de tipo `ForcePayment`, con lo cual quedarán actualizadas en la base de datos sin importar si los
  límites eran validos o no.

#### Caída del Líder

- El `HeartbeatManager` detecta la caída y notifica al `ConnectionManager`.
- El `ConnectionManager` notifica al `ElectionManager`.
- El `ElectionManager` inicia un proceso de elección.
- Las transacciones en curso hacen una especie de rollback y se vuelven a enviar al nuevo líder como petición de lock
- Las transacciones que habían iniciado con un líder y se van a finalizar habiendo cambiado de líder en el transcurso de la
  las mismas serán ignoradas, y volverán a ser enviadas como petición de lock al nuevo lider

#### Caída del Nodo con Lock

- El líder detecta la caída a través del `HeartbeatManager`.
- El `LockManager` elimina el nodo del HashMap de `current_transactions` las cuales son las transacciones del current node
  que tienen el lock acutalmente.
- Si el nodo caído tenía el lock, el líder se dará cuenta de esto y procederá a otorgar el lock automáticamente al
  siguiente en la cola.

---

## Configuración

La configuración se realiza principalmente a través de variables de entorno (o archivo `.env`):

```env
NUM_NODES=4
LOG_LEVEL=Trace
SERVER_PORT_BASE=7000
PROXY_PORT_BASE=6000
STATION_PORT_BASE=8000
MAX_DISCOVERY_ROUNDS=2
HB_MISSES_ALLOWED=3
```

- **`NUM_NODES`**: Cantidad total de nodos en el cluster.
- **`SERVER_PORT_BASE`**: Puerto base para comunicación entre nodos (puerto = base + id).
- **`PROXY_PORT_BASE`**: Puerto base para conexiones de proxies.
- **`STATION_PORT_BASE`**: Puerto base para conexiones de estaciones.
- **`MAX_DISCOVERY_ROUNDS`**: Rondas máximas para descubrimiento de peers.
- **`HB_MISSES_ALLOWED`**: Cantidad de heartbeats perdidos antes de considerar un nodo caído.
- **`LOG_LEVEL`**: Nivel de detalle de los logs (Info, Debug, Trace, etc.).

---

## Uso

Para levantar un nodo del cluster, se debe ejecutar el binario pasando su ID como argumento:

```bash
  cargo run -p cluster -- <node_id>
```

El nodo iniciará automáticamente:
- La escucha de conexiones TCP en los puertos configurados.
- El envío y recepción de heartbeats UDP.
- El proceso de descubrimiento y conexión con otros nodos.
- El proceso de elección de líder.

