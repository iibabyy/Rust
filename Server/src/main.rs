#[allow(non_snake_case)]
mod Parsing;
mod server;
mod client;
mod traits;
mod request;
mod response;

use tokio::{signal, task::JoinSet};
use server::server::Server;
use tokio_util::sync::CancellationToken;
use Parsing::*;
use std::net::IpAddr;

#[tokio::main]
async fn main() {
	let cancel_token = CancellationToken::new();
	let config = Parsing::get_config("conf.conf".to_owned()).await;
	let servers = match Server::init_servers(config) {
		Ok(vec) => vec,
		Err(e) => {
			eprintln!("Error: {}", e);
			return ;
		}
	};

	tokio::spawn( {
		let cancel_token = cancel_token.clone();
		async  move {
			if let Ok(()) = signal::ctrl_c().await {
				println!(" Server shutdown");
				cancel_token.cancel();
			}
		}
	});

	// println!("--------------------[ CONFIG ]--------------------\n\n{:#?}", servers);
	// println!("--------------------------------------------------\n");

	let mut task = JoinSet::new();
	for serv in &servers {
		task.spawn(serv.to_owned().run(IpAddr::from([127, 0, 0, 1]), cancel_token.clone()));
	}

	while let Some(res) = task.join_next().await {
		match res {
			Err(e) => { println!("----[Error: {e}]----") },
			Ok(_) => {},
		}
	}
}
