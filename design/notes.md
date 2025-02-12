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
8. Using the `MsgType` of the `GenericMsg`, the `Host` performs an action, then may or may not send a reply to the `Node`. 
    1. `MsgType::Set`
       - Action: Insert this message into the database using the `topic` as the tree and the `timestamp` as the key.
       - Reply: No reply needed    
    2. `MsgType::Get`
       - Should be equivalent to a `MsgType::GetNth(0)` operation   
       - Action: Retrieve the last message in the database on the `topic` and send it to the requester
       - Reply: Sends the retrieved message
    3. `MsgType::Subscribe`
       - Typically derived from a strongly-typed `Msg<Duration>`.
       - Action: Begin a `Host`-side blocking loop that will retrieve the last message on the `topic` and send it to the subscribed `Node` at a given `rate`. 
       - Reply: Stream of messages at the specified rate 
    4. `MsgType::GetNth`
       - Action: Retrieve the n'th message back in the database log on the `topic` and send it to the requester
       - Reply: Sends the retrieved message
    5. `MsgType::Topics`
       - Action: Create a list of all available topics, format them into `Vec<String>`, then into a `Msg<Vec<String>>` and then into `GenericMsg`.  
       - Reply: Send the created message
  
At any point during these operations, a failure can be had, which will be in the form of `meadow::Error` enum. This error type is serializable, and so can be included in `Msg` types. As a result, a failure of any of the `Host`-side actions will result in a `MsgType::Error(e)`-based `GenericMsg` being sent back to the `Node`, which is responsible for propagating this message.  
