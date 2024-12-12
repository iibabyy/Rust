#[allow(non_snake_case)]
mod Server;
#[allow(non_snake_case)]
mod Parsing;
mod traits;

use Server::server::Server as server;
use Parsing::*;
use std::net::IpAddr;


#[tokio::main]
async fn main() {
	let config = Parsing::get_config("src/conf.conf".to_string()).await;
	let server = match server::init_servers(config) {
		Ok(vec) => vec,
		Err(e) => {
			println!("Error: {}", e);
			return ;
		}
	};
	println!("Config:\n{:#?}", server);

	server.first().unwrap().run(IpAddr::from([127, 0, 0, 1])).await.unwrap();
}
