// Tokio for async
// use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

use postcard::*;
use serde::{de::DeserializeOwned, Serialize};
// use std::thread::{self, JoinHandle};

use std::sync::{Arc, Mutex};
// use tokio::sync::Mutex;

use std::error::Error;
use std::result::Result;

use crate::msg::*;

#[derive(Debug)]
pub struct Host {
    listener: Arc<Mutex<TcpListener>>,
    store: sled::Db,
    thread_start: Option<JoinHandle<()>>,
}

#[derive(Debug)]
pub struct HostConfig {
    ip: String,
    socket_num: usize,
    store_name: String,
}

impl HostConfig {
    pub fn new(ip: String) -> HostConfig {
        HostConfig {
            ip,
            socket_num: 25_000,
            store_name: "rhiza_store".into(),
        }
    }

    pub fn socket_num(mut self, socket_num: usize) -> HostConfig {
        self.socket_num = socket_num;
        self
    }

    pub async fn build(&self) -> Result<Host, Box<dyn Error>> {
        // self.validate();
        //let socket = UdpSocket::bind(self.ip.clone() + &self.socket_num.to_string())?;
        let listener = Arc::new(Mutex::new(TcpListener::bind(self.ip.clone() + &self.socket_num.to_string()).await?));

        let store: sled::Db = sled::open(&self.store_name)?;

        let host = Host {
            listener,
            store,
            thread_start: None,
        };

        Ok(host)
    }
}

impl Host {
    pub async fn default(store_name: &str) -> Result<Host, Box<dyn Error>> {
        let ip = "127.0.0.1:"; // Defaults to localhost

        //let socket = UdpSocket::bind(ip.to_owned() + "25000")?;
        let listener = Arc::new(Mutex::new(
            TcpListener::bind(ip.to_owned() + "25000").await?,
        ));

        let config = sled::Config::default().path(store_name).temporary(true);
        let store: sled::Db = config.open()?;

        let host = Host {
            listener,
            store,
            thread_start: None,
        };

        Ok(host)
    }

    pub fn get<T: Serialize + DeserializeOwned + std::fmt::Debug>(
        &mut self,
        query: &str,
    ) -> Option<T> {
        // let store = self.store.lock().unwrap();
        println!("Asking for '{}' from the store", &query);
        let val: Option<T> = match self.store.get(query).ok()? {
            Some(bytes) => {
                let v: T = from_bytes(&bytes[..]).unwrap();
                Some(v)
            }
            None => None,
        };
        /*println!(
            "From the kv store using query '{}', we got {:?}",
            query, bytes
        );*/
        //let v: T = from_bytes(&bytes[..]).unwrap();
        //v
        val
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn Error>> {
        
        loop {
            // The second item contains the IP and port of the new connection.
            let listener = self.listener.lock().unwrap();
            let (stream, _) = listener.accept().await.unwrap();
            let db = self.store.clone();
            tokio::spawn(async move {
                // Wait for the socket to be readable
                // stream.readable().await.unwrap();

                // Creating the buffer **after** the `await` prevents it from
                // being stored in the async task.
                // let mut buf = [0; 4096];
                let mut buf = [0u8; 4096];
                // Try to read data, this may still fail with `WouldBlock`
                // if the readiness event is a false positive.
                loop {
                    stream.readable().await.unwrap();

                    match stream.try_read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            let bytes = &buf[..n];
                            let msg: GenericRhizaMsg = from_bytes(bytes).unwrap();
                            dbg!(&msg);

                            match msg.msg_type {
                                Msg::SET => {
                                    println!(
                                        "received {} bytes, to be assigned to: {}",
                                        n, &msg.name
                                    );
                                    db.insert(msg.name.as_bytes(), bytes).unwrap();
                                }
                                Msg::GET => loop {
                                    println!(
                                        "received {} bytes, asking for reply on topic: {}",
                                        n, &msg.name
                                    );

                                    stream.writable().await.unwrap();
                                    let return_bytes = db.get(&msg.name).unwrap().unwrap();
                                    match stream.try_write(&return_bytes) {
                                        Ok(n) => {
                                            println!("Successfully replied with {} bytes", n);
                                            break;
                                        }
                                        Err(ref e)
                                            if e.kind() == std::io::ErrorKind::WouldBlock =>
                                        {
                                            continue;
                                        }
                                        Err(e) => {
                                            println!("Error: {:?}", e);
                                            continue;
                                            //return Err(e.into());
                                        }
                                    }
                                },
                            }

                            //self.store.insert(msg.name.as_bytes(), bytes).unwrap();
                            //let decoded: RhizaMsg<Pose> = from_bytes(bytes).unwrap();
                            //dbg!(&decoded);
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            // println!("Error::WouldBlock: {:?}", e);
                            continue;
                        }
                        Err(e) => {
                            println!("Error: {:?}", e);
                            // return Err(e.into());
                        }
                    }
                }
            });
        }

        
    }

    pub fn stop(&mut self) -> Result<(), Box<dyn Error>> {
        panic!("unimplemented!");
        Ok(())
    }
}

#[test]
fn test_default() {
    let host = Host::default("store");
}
