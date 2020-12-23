use crate::simple_sublist::SimpleSubList;
use std::error::Error;
use rmq::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("server start...");
    let s: Server<SimpleSubList> = Server::default();
    s.start().await
}