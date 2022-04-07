/// Configuration for Tokio UDP socket
#[derive(Debug)]
pub struct UdpConfig {
    pub interface: String,
    pub socket_num: usize,
    pub max_buffer_size: usize,
    pub max_name_size: usize,
}

impl UdpConfig {
    pub fn default(interface: impl Into<String>) -> Self {
        UdpConfig {
            interface: interface.into(),
            socket_num: 25_000,
            max_buffer_size: 10_000,
            max_name_size: 100,
        }
    }

    pub fn set_socket_num(mut self, socket_num: usize) -> UdpConfig {
        self.socket_num = socket_num;
        self
    }

    pub fn set_max_buffer_size(mut self, max_buffer_size: usize) -> UdpConfig {
        self.max_buffer_size = max_buffer_size;
        self
    }

    pub fn set_max_name_size(mut self, max_name_size: usize) -> UdpConfig {
        self.max_name_size = max_name_size;
        self
    }
}
