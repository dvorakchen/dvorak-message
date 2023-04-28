use dctor::server;

mod args;
mod dctor;

#[tokio::main]
async fn main() {
    let args = args::Args::parse();

    // let mut server = Server::new(&args.host).await;
    let mut server = server::Server::new(&args.host).await;

    println!("Start");
    server.listen().await;
}
