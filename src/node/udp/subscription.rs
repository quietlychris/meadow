impl<T: Message> From<Node<Udp, Idle, T>> for Node<Udp, Subscription, T> {
    fn from(node: Node<Udp, Idle, T>) -> Self {
        Self {
            //__interface: PhantomData,
            __state: PhantomData,
            __data_type: PhantomData,
            cfg: node.cfg,
            runtime: node.runtime,
            stream: node.stream,
            name: node.name,
            topic: node.topic,
            socket: node.socket,
            endpoint: node.endpoint,
            subscription_data: node.subscription_data,
            task_subscribe: None,
        }
    }
}