/// Configuration for TCP listener
#[derive(Debug)]
pub struct TcpConfig {
    pub interface: String,
    pub socket_num: usize,
    pub max_buffer_size: usize,
    pub max_name_size: usize,
}

impl TcpConfig {
    pub fn default(interface: impl Into<String>) -> Self {
        TcpConfig {
            interface: interface.into(),
            socket_num: 25_000,
            max_buffer_size: 10_000,
            max_name_size: 100,
        }
    }

    pub fn set_socket_num(mut self, socket_num: usize) -> TcpConfig {
        self.socket_num = socket_num;
        self
    }

    pub fn set_max_buffer_size(mut self, max_buffer_size: usize) -> TcpConfig {
        self.max_buffer_size = max_buffer_size;
        self
    }

    pub fn set_max_name_size(mut self, max_name_size: usize) -> TcpConfig {
        self.max_name_size = max_name_size;
        self
    }
}
