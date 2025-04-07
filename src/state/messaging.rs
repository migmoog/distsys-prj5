use serde::{Deserialize, Serialize};
use tokio::io::{self, AsyncWriteExt};

use crate::setup::hostsfile::{ClientId, ObjectId};

use super::bootstrap::NodeId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operation {
    STORE,
    RETRIEVE,
}

pub type RequestId = u64;
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum Message {
    Join(NodeId),
    NewPredecessor(NodeId),
    NewSuccessor(NodeId),

    // Object manipulation
    REQUEST(RequestId, ObjectId, ClientId, Operation),
    OBJ_STORED(ObjectId, ClientId, NodeId),
    OBJ_RETRIEVED(ObjectId, ClientId, NodeId),
    NOT_FOUND(ObjectId),
}
impl Message {
    pub async fn send<T: AsyncWriteExt + Unpin>(&self, to: &mut T) -> io::Result<()> {
        let encoded = bincode::serialize(self).expect("Serializeable");
        to.write(&encoded).await?;
        Ok(())
    }

    pub fn is_obj_response(&self) -> bool {
        matches!(
            self,
            Self::OBJ_STORED(_, _, _) | Self::OBJ_RETRIEVED(_, _, _) | Self::NOT_FOUND(_)
        )
    }
}
