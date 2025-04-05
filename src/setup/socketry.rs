use std::{future::Future, thread::sleep, time::Duration};

use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream},
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
};

use crate::state::messaging::Message;

pub fn host() -> String {
    hostname::get().unwrap().into_string().unwrap()
}

pub async fn attempt_op<F, Fut, Socket>(op: F, host: &str) -> Socket
where
    F: Fn(String) -> Fut,
    Fut: Future<Output = std::io::Result<Socket>>,
{
    let addr = format!("{}:{}", host, "6969");
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

pub fn bootstrap_comms(listener: TcpListener) -> UnboundedReceiver<Message> {
    let (send, recv) = unbounded_channel::<Message>();
    tokio::spawn(async move {
        loop {
            if let Ok((stream, _)) = listener.accept().await {
                let send_clone = send.clone();
                tokio::spawn(async move { check_node(stream, send_clone).await });
                //println!("Set up a connection");
            }
        }
    });

    recv
}
