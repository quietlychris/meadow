mod active;
mod idle;
mod subscription;

use crate::msg::{GenericMsg, Message, Msg, MsgType};
use std::convert::TryInto;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::Mutex as TokioMutex;

use tracing::*;

use crate::error::Error;
use std::io::{Error as IoError, ErrorKind};
use std::net::SocketAddr;

/* #[inline]
#[tracing::instrument(skip(buffer))]
pub async fn await_response<T: Message>(
    socket: &UdpSocket,
    buffer: Arc<TokioMutex<Vec<u8>>>,
) -> Result<Msg<T>, Error> {
    socket.readable().await?;
    loop {
        let mut buf = buffer.lock().await;

        match socket.recv(&mut buf).await {
            Ok(0) => {
                info!("await_response received zero bytes");
                continue;
            }
            Ok(n) => {
                // info!("await_response received {} bytes", n);
                let bytes = &buf[..n];

                let generic = postcard::from_bytes::<GenericMsg>(bytes)?;
                match generic.msg_type {
                    MsgType::Result(result) => {
                        if let Err(e) = result {
                            return Err(e);
                        }
                    }
                    _ => {
                        let msg: Msg<T> = generic.try_into()?;
                        return Ok(msg);
                    }
                }
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    error!("Would block");
                }
                continue;
            }
        }
    }
} */

#[inline]
#[tracing::instrument(skip(buffer))]
pub async fn await_response(
    socket: &UdpSocket,
    buffer: Arc<TokioMutex<Vec<u8>>>,
) -> Result<GenericMsg, Error> {
    socket.readable().await?;
    loop {
        let mut buf = buffer.lock().await;

        match socket.recv(&mut buf).await {
            Ok(0) => {
                info!("await_response received zero bytes");
                continue;
            }
            Ok(n) => {
                // info!("await_response received {} bytes", n);
                let bytes = &buf[..n];
                let msg = postcard::from_bytes::<GenericMsg>(bytes)?;
                return Ok(msg);
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    error!("Would block");
                }
                continue;
            }
        }
    }
}

#[inline]
async fn send_msg(
    socket: &UdpSocket,
    packet: Vec<u8>,
    host_addr: SocketAddr,
) -> Result<usize, Error> {
    socket.writable().await?;
    // NOTE: This used to be done 10 times in a row to make sure it got through
    let n = socket.send_to(&packet, host_addr).await?;
    Ok(n)
}
