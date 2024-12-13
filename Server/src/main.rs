#[allow(non_snake_case)]
mod Server;
#[allow(non_snake_case)]
mod Parsing;
mod traits;
mod request;

use tokio::task::JoinSet;
use Server::server::Server as server;
use Parsing::*;
use std::net::IpAddr;


#[tokio::main(flavor = "current_thread")]
async fn main() {
	let config = Parsing::get_config("src/conf.conf".to_string()).await;
	let servers = match server::init_servers(config) {
		Ok(vec) => vec,
		Err(e) => {
			println!("Error: {}", e);
			return ;
		}
	};

	println!("--------------------[ CONFIG ]--------------------\n\n{:#?}", servers);
	println!("--------------------------------------------------\n");

	let mut task = JoinSet::new();
	for serv in &servers {
		task.spawn(serv.to_owned().run(IpAddr::from([127, 0, 0, 1])));
	}
	while let Some(res) = task.join_next().await {
		match res {
			Err(e) => { eprintln!("----[Error: {e}]----") },
			Ok(_) => {},
		}
	}
}
