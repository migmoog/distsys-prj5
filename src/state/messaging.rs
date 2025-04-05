use serde::{Deserialize, Serialize};
use tokio::{
    io::{self, AsyncWriteExt},
    net::TcpStream,
};

use super::bootstrap::NodeId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    Join(NodeId),
}
impl Message {
    pub async fn send(&self, to: &mut TcpStream) -> io::Result<()> {
        let encoded = bincode::serialize(self).expect("Serializeable");
        to.write(&encoded).await?;
        println!("Sent the message");
        Ok(())
    }
}
