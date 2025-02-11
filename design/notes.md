## Meadow Design Notes

- Meadow currently relies on transport-layer   


Each message is serialized into two types of messages:
1. `Msg<T>`, which is a strongly-typed variant that is primarily used in the user-facing APIs. 
2. `GenericMsg`, which has structural overlap with `Msg<T>`, but carries its data payload as an vector of bytes `Vec<u8>`.

Meadow assumes that the `Host` process is reliable, and can not be crashed by actions on the `Node`. Data flow is generally considered to be push-pull from the `Node`-side; the `Node` should not be receiving data from a `Host` unless it has asked for it, and can not have data arbitrarily pushed to it by any other `Node` on the network. The `Host` is responsible for both logging and exchanging data. 

For all message types, the `Node` operates by: 

1. Creating a strongly-typed message `Msg<T>`, which carries both a data payload and a requested `Host`-side operation, s denoted by the `MsgType` field. 
2. Converting that message to a `GenericMsg`.
3. Converting the `GenericMsg` to a vector of bytes `Vec<u8>` via `postcard`.
4. Creating a connection the `Host` located at a given address, which provides both the `Host` with address information of sender. 
5. Sending the `Vec<u8>` over the connection to the host
6. The `Host` receives a vector of bytes `Vec<u8>`
7. The `Host` attempts to deserialize that `Vec<u8>` into a `GenericMsg`.
    - If successful: 
    - If not successful: 
8. Using the `MsgType` of the `GenericMsg`, the `Host` performs an operation
    1. `MsgType::Set`
       - Insert this message into the database using the `topic` as the tree and the `timestamp` as the key.
       -     
    2. `MsgType::Get`
       - Retrieve the last message in the database on the `topic` and send it to the requester
  
    3. `MsgType::Subscribe`
       - Begin a `Host`-side blocking loop that will retrieve the last message on the `topic` and send it to the subscribed `Node` at a given `rate`.
       - Typically derived from a strongly-typed `Msg<Duration>`. 
    4. `MsgType::GetNth`
       - Retrieve the n'th message back in the database log on the `topic` and send it to the requester
       - 
    5. `MsgType::Topics`
       - Create a list of all available topics, format them into `Vec<String>`, then into a `Msg<Vec<String>>` and then into `GenericMsg`.  

At any point during these operations, a failure can be had, which will be in the form of `meadow::Error` enum. 
