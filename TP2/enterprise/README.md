# Enterprise Service

El servicio `enterprise` representa una empresa en sí, permite registrar por pantalla todas las transacciones que se realizan con sus tarjetas. También provee una interfaz de administración para gestionar el balance de la empresa y los límites de las tarjetas.

## Actores

- Enterprise
- Proxy
- AdminInput

## Funcionamiento

### Enterprise

Este actor es el núcleo del servicio. Mantiene el estado del balance de la empresa y los límites de crédito de cada tarjeta. Procesa los comandos de administración y las transacciones que llegan desde el cluster.

### Proxy

Actúa como un puente de comunicación entre el servicio `enterprise` y el cluster de nodos. Su principal responsabilidad es reenviar los mensajes del `Enterprise` al cluster.

- **Conexión y Reconexión**: Al iniciarse, el `Proxy` intenta conectarse a uno de los nodos del cluster de forma rotativa. Si la conexión con un nodo se pierde, el `Proxy` intentará reconectarse automáticamente a otro nodo para mantener la comunicación y asegurar la disponibilidad del servicio.

- **Reenvío de Mensajes**: Una vez conectado, el `Proxy` reenvía los `ProtocolMessage` que recibe del actor `Enterprise` hacia el nodo del cluster.

- **Monitoreo de Conexión**: El `Proxy` utiliza un `HearthbeatManager` para monitorear la salud de la conexión. El `HearthbeatManager` escucha los *heartbeats* que envía el nodo del cluster. Si deja de recibir estos *heartbeats*, asume que la conexión se ha perdido y notifica al `Proxy` para que inicie el proceso de reconexión.

- **Componentes de Red**: Internamente, el `Proxy` utiliza un `TcpProtocol` para manejar la lectura y escritura de datos en el socket TCP, facilitando la comunicación con el cluster.

### AdminInput

Provee una interfaz de línea de comandos (CLI) para que un administrador pueda interactuar con el sistema. A través de esta interfaz, se pueden realizar operaciones como actualizar el balance de la empresa o modificar el límite de una tarjeta.

## Mensajes

La comunicación entre los actores se realiza a través de un sistema de mensajes bien definido:

### Comandos de Administración (`AdminCommand`)

Estos mensajes son generados por la interfaz de `AdminInput` y procesados por el actor `Enterprise`:

- **`UpdateEnterpriseLimit { limit }`**: Actualiza el limite de la empresa.
- **`UpdateCardLimit { card_id, limit }`**: Actualiza el límite de una tarjeta específica.
- **`ViewCard { card_id }`**: Solicita ver el límite y el consumo de una tarjeta.
- **`ViewEnterprise { }`**: Solicita ver el balance y el consumo de la empresa.

### Mensajes del Protocolo del Cluster (`ProtocolMessage`)

Estos mensajes se intercambian con el cluster a través del `Proxy`:

- **Mensajes Enviados por Enterprise**:
  - `StartUp`: Notifica al cluster que el servicio se ha iniciado.
  - `InitialState`: Envía el estado inicial (balance y límites de tarjetas) al cluster.
  - `EnterpriseLimitUpdate`: Informa de una actualización en el limite de la empresa.
  - `CardLimitUpdate`: Informa de una actualización en el límite de una tarjeta.
  - `CardView`: Pide al cluster el limite o el consumo de una tarjeta.
  - `EnterpriseView`: Pide al cluster el limite o el consumo de la empresa.

- **Mensajes Recibidos por Enterprise**:
  - `Transaction`: Recibe la notificación de una nueva transacción realizada.
  - `CardViewResponse`: Recibe el limite o el consumo de una tarjeta.
  - `EnterpriseViewResponse`: Recibe el limite o el consumo de la empresa.

### Mensajes Internos

- **`SetProxyAddr`**: El `Proxy` envía este mensaje al `Enterprise` al iniciarse para que este último pueda enviarle mensajes.

## Configuración

El servicio `enterprise` requiere de dos tipos de configuración:

### 1. Archivo de Configuración (TOML)

La configuración principal del enterprise se especifica en un archivo TOML. Este archivo debe contener el `id` del enterprise, su `balance` inicial y una lista de `cards` con su `id` y `limit`.

Ejemplo (`cfg.toml`):

```toml
id = 1
balance = 10000
cards = [
    { id = 1, limit = 1000 },
    { id = 2, limit = 500 },
]
```

### 2. Variables de Entorno (.env)

La configuración del proxy se realiza a través de variables de entorno. Se recomienda crear un archivo `.env` en la raíz del enterprise para gestionarlas.

Ejemplo (`.env`):

```env
HOST=localhost
PROXY_PORT_BASE=8080
INTERNODE_PORT_BASE=7000
NUM_NODES=4
```

- `HOST`: La dirección IP donde corren los nodos del cluster.
- `PROXY_PORT_BASE`: El puerto base para el proxy.
- `INTERNODE_PORT_BASE`: El puerto base para la comunicación entre nodos del cluster.
- `NUM_NODES`: La cantidad de nodos en el cluster.

## Uso

Para levantar el servicio, se debe ejecutar el binario `enterprise` pasándole como argumento la ruta al archivo de configuración TOML.

```bash
cargo run -p enterprise <path_to_config.toml>
```

Una vez que el servicio está corriendo, se pueden utilizar los siguientes comandos en la terminal para administrar el sistema:

- **Actualizar límite de la empresa**:

  ```bash
  enterprise (add/sub/set) <monto>
  ```

- **Actualizar límite de una tarjeta**:

  ```bash
  card (add/sub/set) <id_tarjeta> <limite>
  ```

- **Ver límite y consumo de la empresa**:

  ```bash
  enterprise view
  ```

- **Ver límite y consumo de una tarjeta**:

  ```bash
  card view <id_tarjeta>
  ```
