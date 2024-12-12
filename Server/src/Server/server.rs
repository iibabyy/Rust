use std::{collections::HashMap, net::{IpAddr, SocketAddr}, path::PathBuf, sync::Arc};

use tokio::{io::{self, AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream}};

use crate::Parsing::*;

#[derive(Clone, Debug)]
pub struct Location {
	root: Option<PathBuf>,
	path: PathBuf,
	infos: HashMap<String, Vec<String>>,
	cgi: HashMap<String, PathBuf>,
}

#[derive(Clone, Debug)]
pub struct Server {
	port: u16,
	default: bool,
	root: Option<PathBuf>,
	name: Option<String>,
	infos: HashMap<String, Vec<String>>,
	locations: HashMap<PathBuf, Location>,
	cgi: HashMap<String, PathBuf>,
}

impl Location {
	fn new(location: LocationBlock) -> Result<Self, String> {
		let mut new_location = Location {
			path: PathBuf::from(location.path),
			root: None,
			infos: HashMap::new(),
			cgi: HashMap::new(),
		};

		for directive in location.directives {
			match directive.0.as_str() {
				"root" => {		// ROOT
					if directive.1.is_empty() { new_location.root = None; }
					else if directive.1.len() > 1 { return Err("invalid location field: root".to_string()); }
					else { new_location.root = Some(PathBuf::from(directive.1.first().unwrap().clone())); }
				} _ => {
					new_location.infos.insert(directive.0, directive.1);
				}
			}
		}

		new_location.cgi = location.cgi;

		Ok(new_location)
	}
}

impl Server {

	pub fn new(config: ServerBlock) -> Result<Self, String> {
		let mut serv = Server {
			infos: HashMap::new(),
			locations: HashMap::new(),
			cgi: HashMap::new(),
			port: 0,
			default: false,
			root: None,
			name: None,
		};

		for directive in config.directives {
			serv.add_directive(directive.0, directive.1)?;
		}

		for location in config.locations {
			serv.add_location(location.1)?;
		}

		serv.cgi = config.cgi;

		Ok(serv)
	}

	pub fn init_servers(configs: Vec<ServerBlock>) -> Result<Vec<Self>, String> {
		let mut servers = Vec::new();

		for server_config in configs {
			servers.push(Self::new(server_config)?);
		}

		Ok(servers)
	}

	pub async fn run(&self, ip: IpAddr) -> Result<(), io::Error>{
		let listener = TcpListener::bind(SocketAddr::new(ip, self.port)).await?;
		println!("------Server listening on [{}]------", ip);
		let server = Arc::new(self.clone());
		loop {
			let (stream, _) = listener.accept().await?;
			println!("------Connection accepted------");
			let server_instance = Arc::clone(&server);
			tokio::spawn( async move {
				server_instance.handle_client(stream).await
			});
		}
	}
	fn add_directive(&mut self, name: String, infos: Vec<String>) -> Result<(), String> {
		match name.as_str() {
			"root" => {		// ROOT
				if infos.is_empty() { self.root = None; }
				else if infos.len() > 1 { return Err("invalid field: root".to_string()); }
				else { self.root = Some(PathBuf::from(infos.first().unwrap().clone())); }
			} "listen" => {		// LISTEN
				if infos.is_empty() || infos.len() > 2 { return Err("invalid field: listen".to_string()) }
				else {
					self.port = match infos.first().unwrap().parse::<u16>() {
						Ok(num) => num,
						Err(_) => return Err("invalid field: listen (0 <= port <= 65535)".to_string()),
					}
				}
				if infos.len() == 2 && infos[1] == "default" { self.default = true; }
			} "server_name" => {		// SERVER_NAME
				if infos.len() != 1 { return Err("invalid field: server_name".to_string()) }
				else { self.name = Some(infos[0].clone()) }
			} _ => {
				self.infos.insert(name, infos);
			}
		}
		Ok(())
	}

	fn add_location(&mut self, location: LocationBlock) -> Result<(), String> {
		let new_location = Location::new(location)?;

		self.locations.insert(new_location.path.clone(), new_location);

		Ok(())
	}

	async fn handle_client(&self, mut stream: TcpStream) {
		let mut buffer = [0; 1024];

		//	getting request
		stream.read(&mut buffer).await.expect("failed to receive request !");
		let buffer = String::from_utf8_lossy(&buffer[..]);
		println!("Request received: {}", buffer);
		//	sending RESPONSE
		let response = b"HTTP/1.1 200 OK\r\n\r\nHello from server !\r\n";
		stream.write(response).await.expect("failed to send response");
	}

}
