use std::{sync::Arc, time::Duration};

use resp::Value;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

mod resp;
mod storage;

struct InnerServer {
    listener: TcpListener,
    storage: Mutex<storage::Storage>,
}

pub struct Server {
    inner: Arc<InnerServer>,
}

impl Server {
    pub async fn new() -> Self {
        Server {
            inner: Arc::new(InnerServer {
                listener: TcpListener::bind("0.0.0.0:6379").await.unwrap(),
                storage: Mutex::new(storage::Storage::new()),
            }),
        }
    }

    pub async fn listen(&self) {
        loop {
            let stream = self.inner.listener.accept().await;
            match stream {
                Ok((stream, _)) => {
                    println!("accepted new connection");
                    let inner = self.inner.clone();
                    tokio::spawn(async move { inner.handle_conn(stream).await });
                }
                Err(e) => {
                    println!("error: {}", e);
                }
            }
        }
    }
}

impl InnerServer {
    async fn handle_conn(&self, stream: TcpStream) {
        let mut handler = resp::RespHandler::new(stream);

        println!("Starting read loop");

        loop {
            let value = handler.read_value().await.unwrap();

            println!("Got value {:?}", value);

            let mut storage = self.storage.lock().await;
            let response = match value {
                Some(v) => {
                    let (command, args) = v.extract_command().unwrap();
                    match command.as_str() {
                        "ping" => Value::SimpleString("PONG".to_string()),
                        "echo" => args.first().unwrap().clone(),
                        "set" => {
                            match (args.get(2), args.get(3)) {
                                (Some(Value::BulkString(px)), Some(Value::BulkString(expires))) if px.to_lowercase() == "px" => storage.set(
                                    args[0].unpack_bulk_str().unwrap(),
                                    args[1].unpack_bulk_str().unwrap(),
                                    Some(Duration::from_millis(expires.parse().unwrap())),
                                ),
                                _ => storage.set(
                                    args[0].unpack_bulk_str().unwrap(),
                                    args[1].unpack_bulk_str().unwrap(),
                                    None,
                                )
                            }
                        }
                        "get" => storage.get(args[0].unpack_bulk_str().unwrap()),
                        c => panic!("Cannot handle command {}", c),
                    }
                }
                _ => break,
            };

            println!("Sending value {:?}", response);
            handler.write_value(response).await.unwrap();
        }
    }
}
