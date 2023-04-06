use server::Server;

mod server;
mod args;

#[tokio::main]
async fn main() {
    let args = args::Args::parse();

    let mut server = Server::new(&args.host).await;

    println!("Start");
    server.listen().await;
}
