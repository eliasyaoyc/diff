use crate::client::*;
use crate::simple_sublist::SubListTrait;
use bitflags::_core::sync::atomic::AtomicUsize;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

#[derive(Default, Default)]
pub struct Server<T: SubListTrait> {
    state: Arc<Mutex<ServerState<T>>>,
}

#[derive(Default, Debug)]
pub struct ServerState<T: SubListTrait> {
    clients: HashMap<u64, Arc<Mutex<ClientMessageSender>>>,
    pub sublist: T,
    pub gen_cid: u64,
}

impl<T: SubListTrait + Send + 'static> Server<T> {
    pub async fn start(&self) -> Result<(), Box<dyn Error>> {
        let addr = "127.0.0.1:4222";

        let mut listener = TcpListener::bind(addr).await?;
        println!("listening on:{}", addr);
        loop {
            let (mut socket, _) = listener.accept().await?;
            self.new_client(socket).await;
        }
    }

    async fn new_client(&self, conn: TcpStream) {
        let cid = {
            let mut state = self.state.lock().await;
            state.gen_cid += 1;
            state.gen_cid
        };
        let c = Client::process_connection(cid, self.state.clone(), conn);
        let mut state = self.state.lock().await;
        state.clients.insert(cid, c);
    }
}