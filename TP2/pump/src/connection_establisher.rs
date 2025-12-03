use std::time::Duration;
use tokio::net::TcpStream;

const INTERVAL_PERIOD: Duration = Duration::from_secs(3);

pub struct ConnectionEstablisher {
    socket_addr: String,
}

impl ConnectionEstablisher {
    pub fn new(socket_addr: String) -> Self {
        Self { socket_addr }
    }
    pub async fn generate_connection(&self) -> TcpStream {
        loop {
            println!("Trying to connect to station...");
            if let Ok(result) = TcpStream::connect(self.socket_addr.clone()).await {
                println!("Connection successful!");
                return result;
            }
            println!("Couldn't connect to station, sleeping...");
            tokio::time::sleep(INTERVAL_PERIOD).await;
        }
    }
}
