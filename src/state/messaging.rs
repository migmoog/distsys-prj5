use serde::{Deserialize, Serialize};
use tokio::io::{self, AsyncWriteExt};

use super::bootstrap::NodeId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    Join(NodeId),
    NewPredecessor(NodeId),
    NewSuccessor(NodeId),
}
impl Message {
    pub async fn send<T: AsyncWriteExt + Unpin>(&self, to: &mut T) -> io::Result<()> {
        let encoded = bincode::serialize(self).expect("Serializeable");
        to.write(&encoded).await?;
        Ok(())
    }
}
