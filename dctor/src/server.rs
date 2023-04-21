use super::supervisor::ClientSupervisor;

/// representing the server,
/// listening the incoming client and io,
/// start and terminal the whole application
/// 
/// # example
/// ```
/// let server = Server::new();
/// server.listen();
/// ```
pub struct Server {
    client_supervisor: ClientSupervisor,
}

impl Server {
    /// construct a Server
    pub fn new() -> Self {
        Server {
            client_supervisor: ClientSupervisor::new()
        }
    }

    pub fn listen(&mut self) {
        
    }
}
