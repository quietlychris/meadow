use crate::Error;
use crate::*;

use chrono::Utc;

use postcard::*;
use quinn::Connection as QuicConnection;
use std::result::Result;
use tracing::*;

impl<T: Message + 'static> Node<Quic, Active, T> {
    #[tracing::instrument(skip(self))]
    pub fn publish(&self, val: T) -> Result<(), Error> {
        let val_vec: Vec<u8> = match to_allocvec(&val) {
            Ok(val_vec) => val_vec,
            Err(_e) => return Err(Error::Serialization),
        };

        let packet = GenericMsg {
            msg_type: MsgType::SET,
            timestamp: Utc::now(),
            name: self.name.to_string(),
            topic: self.topic.to_string(),
            data_type: std::any::type_name::<T>().to_string(),
            data: val_vec.to_vec(),
        };

        let packet_as_bytes: Vec<u8> = match to_allocvec(&packet) {
            Ok(packet) => packet,
            Err(_e) => return Err(Error::Serialization),
        };

        let server_addr = self.cfg.network_cfg.host_addr;
        let endpoint = self.endpoint.clone().unwrap();

        self.runtime.block_on(async {
            // let mut buf = vec![0; 1_000];

            let connection = endpoint
                .connect(server_addr.clone(), "localhost")
                .unwrap()
                .await
                .unwrap();
            let (mut send, mut _recv) = connection.open_bi().await.unwrap();

            // let msg = format!("test message");
            send.write_all(&packet_as_bytes).await.unwrap();
            send.finish().await.unwrap();

            Ok(())
        })
    }
}
