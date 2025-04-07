use std::{future::Future, thread::sleep, time::Duration};

use tokio::{
    io::{self, AsyncReadExt},
    net::{TcpListener, TcpStream, UdpSocket},
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
};

use crate::state::{bootstrap::NodeId, messaging::Message};

pub fn host() -> String {
    hostname::get().unwrap().into_string().unwrap()
}

pub async fn attempt_op<F, Fut, Socket>(op: F, host: &str, port: u64) -> Socket
where
    F: Fn(String) -> Fut,
    Fut: Future<Output = std::io::Result<Socket>>,
{
    let addr = format!("{}:{}", host, port);
    loop {
        match op(addr.clone()).await {
            Ok(s) => break s,
            Err(_) => sleep(Duration::from_secs(2)),
        }
    }
}
async fn check_node(mut stream: TcpStream, send: UnboundedSender<Message>) {
    let mut buffer = [0; 1024];
    loop {
        if let Ok(bytes_read) = stream.read(&mut buffer).await {
            let msg: Message = bincode::deserialize(&buffer[..bytes_read]).expect("Full message");
            send.send(msg).expect("Send was successful");
        }
    }
}

pub fn bootstrap_comms(
    listener: TcpListener,
) -> (UnboundedSender<Message>, UnboundedReceiver<Message>) {
    let (send, recv) = unbounded_channel::<Message>();
    let sc = send.clone();
    tokio::spawn(async move {
        loop {
            if let Ok((stream, _)) = listener.accept().await {
                let send_clone = sc.clone();
                tokio::spawn(async move { check_node(stream, send_clone).await });
                //println!("Set up a connection");
            }
        }
    });

    (send, recv)
}

/// UdpSocket wrapper designed for communicating with the ring's members
pub struct NodeCaster(UdpSocket);
impl NodeCaster {
    const NODE_PORT: u64 = 6970;

    pub async fn new() -> Self {
        Self(attempt_op(UdpSocket::bind, &host(), Self::NODE_PORT).await)
    }

    /// Sends a message to the specified node ID
    pub async fn tell_node(&mut self, msg: Message, nid: NodeId) -> io::Result<()> {
        let encoded = bincode::serialize(&msg).expect("Is gud");
        let addr = format!("n{nid}:{}", Self::NODE_PORT);
        self.0.send_to(&encoded, addr).await?;
        Ok(())
    }

    /// Awaits to hear any message
    pub async fn hear(&mut self) -> io::Result<Message> {
        let mut buf = [0; 1024];
        let (bytes_read, _) = self.0.recv_from(&mut buf).await?;
        Ok(bincode::deserialize(&buf[..bytes_read]).expect("Should be deserializable"))
    }
}
