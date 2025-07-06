use redb::{Database, ReadableTable, TableDefinition};

use crate::host::GenericStore;
use crate::msg::GenericMsg;
use crate::Error;
use std::sync::Arc;

impl GenericStore for Arc<Database> {
    #[tracing::instrument]
    fn insert_generic(&mut self, msg: GenericMsg) -> Result<(), crate::Error> {
        let bytes = msg.as_bytes()?;
        let write_txn = self.begin_write().unwrap();

        let definition = TableDefinition::<&[u8], &[u8]>::new(&msg.topic);
        let mut tree = write_txn.open_table(definition).unwrap();
        tree.insert(&msg.timestamp.to_string().as_bytes(), bytes.as_slice())
            .unwrap();
        Ok(())
    }

    #[tracing::instrument]
    fn get_generic(
        &self,
        topic: impl Into<String> + std::fmt::Debug,
    ) -> Result<GenericMsg, crate::Error> {
        self.get_generic_nth(topic, 0)
    }

    #[tracing::instrument]
    fn get_generic_nth(
        &self,
        topic: impl Into<String> + std::fmt::Debug,
        n: usize,
    ) -> Result<GenericMsg, crate::Error> {
        let topic: String = topic.into();
        let definition = TableDefinition::<&[u8], &[u8]>::new(&topic);
        let read_txn = self.begin_read().unwrap();

        let table = read_txn.open_table(definition).unwrap();
        if let Ok(table) = read_txn.open_table(definition) {
            if let Ok((_k, v)) = table.iter().unwrap().nth_back(1).unwrap() {
                //println!("nth_back: {:?}, {:?}",k.value(),v.value());
                let v: GenericMsg = postcard::from_bytes(&v.value()[..])?;
                return Ok(v);
            } else {
                return Err(Error::NoNthValue);
            }
        } else {
            Err(Error::NonExistentTopic(topic))
        }
    }
}

use crate::host::Store;

// impl Store for Database {

// }
