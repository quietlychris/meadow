mod active;
mod idle;
// mod subscription

use crate::msg::{GenericMsg, Message, Msg};
use std::convert::TryInto;
use tokio::net::UdpSocket;

use tracing::*;

use crate::Error;

/// Send a `GenericMsg` of `MsgType` from the Node to the Host
/* pub async fn send_msg(socket: &mut &UdpSocket, packet_as_bytes: Vec<u8>) -> Result<(), Error> {
    println!("Checking if UDP socket is writeable");
    match socket.writable().await {
        Ok(_) => (),
        Err(_e) => return Err(Error::AccessSocket),
    };
    println!("Yes UDP socket is writeable");

    // Write the request
    // TO_DO: This should be a loop with a maximum number of attempts
    for _ in 0..self.{
        println!("Trying to send");
        match socket.send(&packet_as_bytes) {
            Ok(_n) => {
                // debug!("Node successfully wrote {}-byte request to host",n);
                break;
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {}
                continue;
            }
        }
    }
    Ok(())
} */

pub async fn await_response<T: Message>(
    socket: &mut &UdpSocket,
    max_buffer_size: usize,
) -> Result<Msg<T>, Error> {
    // Read the requested data into a buffer
    // TO_DO: Having to re-allocate this each time isn't very efficient
    let mut buf = vec![0u8; max_buffer_size];
    // TO_DO: This can be made cleaner
    loop {
        socket.readable().await.unwrap();
        match socket.try_recv(&mut buf) {
            Ok(0) => continue,
            Ok(n) => {
                let bytes = &buf[..n];
                match postcard::from_bytes::<GenericMsg>(bytes) {
                    Ok(generic) => {
                        if let Ok(msg) = TryInto::<Msg<T>>::try_into(generic) {
                            return Ok(msg);
                        } else {
                            return Err(Error::Deserialization);
                        }
                    }
                    Err(_e) => return Err(Error::Deserialization),
                }
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {}
                debug!("Would block");
                continue;
            }
        }
    }
}
