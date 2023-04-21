use server::Server;

mod args;
mod server;

#[tokio::main]
async fn main() {
    let args = args::Args::parse();

    let mut server = Server::new(&args.host).await;

    println!("Start");
    server.listen().await;
}
