use std::{borrow::Cow, collections::HashMap, fmt::Debug, io::Error, net::{IpAddr, SocketAddr}, os::fd::AsFd, path::PathBuf, sync::Arc};

use tokio::{io::{self, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufStream}, net::{TcpListener, TcpStream}, stream};
use tokio_util::sync::CancellationToken;

use crate::{client::client::Client, request::request::{Request, State}, traits::config::Config, Parsing::*};

/*--------------------------[ SERVER ]--------------------------*/

#[derive(Clone, Debug)]
pub struct Server {
	port: Option<u16>,
	socket: Option<SocketAddr>,
	clients: Vec<Arc<Client>>,
	client_max_body_size: Option<u64>,
	default: bool,
	root: Option<PathBuf>,
	name: Option<Vec<String>>,
	index: Option<String>,
	return_: Option<(u16, Option<String>)>,
	error_pages: HashMap<u16, String>,
	error_redirect: HashMap<u16, (Option<u16>, String)>,
	auto_index: bool,
	methods: Option<Vec<String>>,
	infos: HashMap<String, Vec<String>>,
	locations: HashMap<PathBuf, Location>,
	cgi: HashMap<String, PathBuf>,
}



/*------------------------------------------------------------------------------------------------------*/
/*												SERVER													*/
/*------------------------------------------------------------------------------------------------------*/



impl Server {

	pub async fn run(mut self, ip: IpAddr, cancel_token: CancellationToken) -> Result<(), ()>{
		if self.port.is_none() {
			println!("------[No port to listen -> no bind]------");
			return Ok(())
		}

		self.socket = Some(SocketAddr::new(ip, self.port.unwrap()));
		let listener = match TcpListener::bind(self.socket.unwrap().clone()).await {
			Ok(listener) => listener,
			Err(e) => {
				eprintln!("Server ({}): failed to bind: {e}", self.socket.unwrap());
				return Err(());
			}
		};
		println!("------[Server listening on {ip}::{}]------", self.port.unwrap());
		let server = Arc::new(self);

		loop {
			let cancel = cancel_token.clone();
			tokio::select! {
				Ok((stream, addr)) = listener.accept() => {
					println!("------[Connection accepted: {addr}]------");
					let server_instance = Arc::clone(&server);
					tokio::spawn( async move {
						server_instance.handle_client(stream).await
					});
				}
				_ = cancel.cancelled() => {
					println!("------[Server ({}): stop listening]------", server.socket.unwrap());
					return Ok(());
				}
			}
		}
	}

	async fn handle_client(&self, mut stream: TcpStream) {
		
		//	getting request
		loop {
			let buffer = match Self::read_from(&mut stream).await {
				Ok(buf) => buf,
				Err(e) => {
					println!("Server {}: failed to read from client: {e}", self.socket.unwrap());
					return ;
				}
			};
			let mut request = match Request::try_from(buffer) {
				Ok(request) => request,
				Err((code, str)) => {
					println!("Error: {str}");
					stream.write(format!("HTTP/1.1 {code} OK\r\n\r\n{str}\r\n").as_bytes()).await.expect("failed to send response");
					return ;
				}
			};

			while request.state().is_not(State::OnHeader) {
				let buffer = match Self::read_from(&mut stream).await {
					Ok(buf) => buf,
					Err(e) => {
						println!("Server {}: failed to read from client: {e}", self.socket.unwrap());
						return ;
					}
				};

				match request.push(buffer) {
					Ok(_) => (),
					Err((code, msg)) => {
						self.send_error_response_to(stream, code, msg).await;
						return ;
					}
				}
			}

			println!("Request received:\n{:#?}", request);

			//	sending RESPONSE
			if let Err(err) = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 19\r\n\r\nHello from server !\r\n").await {
				println!("Server ({}): failed to send response: {err}", self.socket.unwrap());
				break;
			}
			println!("Response Send !");
			
			if request.keep_connection_alive() == false {
				break;
			}
		}
	}

}


/*---------------------------------------------------------------*/
/*--------------------------[ UTILS ]--------------------------*/
/*---------------------------------------------------------------*/


#[allow(dead_code)]
impl Server {

	pub fn new(config: ServerBlock) -> Result<Self, String> {
		let mut serv = Server {
			port: None,
			socket: None,
			root: None,
			clients: Vec::new(),
			client_max_body_size: None,
			index: None,
			methods: None,
			return_: None,
			auto_index: false,
			error_pages: HashMap::new(),
			error_redirect: HashMap::new(),
			infos: HashMap::new(),
			locations: HashMap::new(),
			cgi: HashMap::new(),
			default: false,
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

	async fn read_from(mut stream: impl AsyncRead + Unpin) -> Result<String, Error> {
		let mut buffer = [0;65536];

		match stream.read(&mut buffer).await {
			Ok(n) => Ok(String::from_utf8_lossy(&buffer[..n]).into_owned()),
			Err(e) => Err(e),
		}
	}

	async fn send_error_response_to(&self, mut stream: impl AsyncWrite + Unpin, error_code: u16, error_msg: String) {
		todo!()
	}

	// async fn create_request_from(&mut self, stream: &mut TcpStream) -> Result<Request, ()> {
	// 	let mut buffer = [0;65536];

	// 	let buffer = match stream.read(&mut buffer).await {
	// 		Ok(n) => String::from_utf8_lossy(&buffer[..n]).into_owned(),
	// 		Err(_) => return Err(()),
	// 	};

	// 	match Request::try_from(buffer) {
	// 		Ok(request) => Ok(request),
	// 		Err(_) => Err(())
	// 	}

	// }

	fn new_client_from(&mut self, stream: TcpStream) -> &Arc<Client> {
		let client = Arc::new(Client::new(stream));
		self.clients.push(client);
		self.clients.last().unwrap()
	}

}


/*---------------------------------------------------------------*/
/*--------------------------[ PARSING ]--------------------------*/
/*---------------------------------------------------------------*/


impl Server {
	fn add_directive(&mut self, name: String, infos: Vec<String>) -> Result<(), String> {
		match name.as_str() {
			"root" => {		// ROOT
				self.root = Some(Self::extract_root(infos)?);
			} "listen" => {		// LISTEN
				(self.port, self.default) = Self::extract_listen(infos)?;
			} "server_name" | "server_names" => {		// SERVER_NAME
				if infos.len() < 1 { return Err("invalid field: server_name".to_string()) }
				else {
					if self.name.is_none() { self.name = Some(Vec::new()) }

					self.name.as_mut().unwrap().append(&mut infos.clone());
				}
			} "index" => {
				self.index = Some(Self::extract_index(infos)?);
			} "auto_index" => {
				self.auto_index = Self::extract_auto_index(infos)?;
			} "client_max_body_size" => {
				self.client_max_body_size = Some(Self::extract_max_body_size(infos)?);
			} "cgi" => {
				let (extension, path) = Self::extract_cgi(infos)?;
				self.cgi.insert(extension, path);
			} "allowed_methods" => {
				if infos.len() < 1 { return Err("invalid field: allowed_methods".to_string()) }
				if self.methods.is_none() { self.methods = Some(Vec::new()) }

				self.methods.as_mut().unwrap().append(&mut infos.clone());
			} "return" => {
				self.return_ = Some(Self::extract_return(infos)?);
			} "error_page" => {
				let (pages, redirect) = Self::extract_error_page(infos)?;
				let hash = &mut self.error_pages;
				if pages.is_some() {pages.unwrap().iter().map(|(code, url)| hash.insert(code.to_owned(), url.to_owned())).last();}
				let hash = &mut self.error_redirect;
				if redirect.is_some() {redirect.unwrap().iter().map(|(code, url)| hash.insert(code.to_owned(), url.to_owned())).last();}
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
}


/*---------------------------------------------------------------*/
/*----------------------[ GETTER / SETTER ]----------------------*/
/*---------------------------------------------------------------*/


#[allow(dead_code)]
impl Server {
	pub fn is_default(&self) -> bool {
        self.default
    }

	pub fn port(&self) -> Option<u16> {
        self.port
    }

	pub fn client_max_body_size(&self) -> Option<u64> {
        self.client_max_body_size
    }

	pub fn root(&self) -> Option<&PathBuf> {
        self.root.as_ref()
    }

	pub fn name(&self) -> Option<&Vec<String>> {
        self.name.as_ref()
    }

	pub fn get(&self, info: String) -> Option<String> {
		Some(self.infos.get(&info)?.join(" "))
	}

	pub fn index(&self) -> Option<&String> {
        self.index.as_ref()
    }

	pub fn auto_index(&self) -> bool {
        self.auto_index
    }

}

impl Config for Server {
	fn root(&self) -> &Option<PathBuf> { &self.root }
	fn auto_index(&self) -> bool { self.auto_index }
	fn cgi_path(&self, extension: String) -> Option<&PathBuf> { self.cgi.get::<String>(&extension) }
	fn index(&self) -> &Option<String> { &self.index }
	fn is_method_allowed(&self, method: String) -> bool { self.methods.as_ref().is_some() && self.methods.as_ref().unwrap().contains(&method) }
	fn port(&self) -> Option<u16> { self.port }
}

/*-------------------------------------------------------------------------------------------------------*/

/*---------------------------------------------------------------*/
/*-------------------------[ LOCATIONS ]-------------------------*/
/*---------------------------------------------------------------*/

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct Location {
	root: Option<PathBuf>,
	path: PathBuf,
	exact_path: bool,
	return_: Option<(u16, Option<String>)>,
	index: Option<String>,
	auto_index: bool,
	error_pages: HashMap<u16, String>,
	error_redirect: HashMap<u16, (Option<u16>, String)>,
	client_max_body_size: Option<u64>,
	infos: HashMap<String, Vec<String>>,
	methods: Option<Vec<String>>,
	redirect: Option<String>,
	cgi: HashMap<String, PathBuf>,
}

impl Config for Location {
	fn root(&self) -> &Option<PathBuf> { &self.root }
	fn auto_index(&self) -> bool { self.auto_index }
	fn cgi_path(&self, extension: String) -> Option<&PathBuf> { self.cgi.get::<String>(&extension) }
	fn index(&self) -> &Option<String> { &self.index }
	fn is_method_allowed(&self, method: String) -> bool { self.methods.as_ref().is_some() && self.methods.as_ref().unwrap().contains(&method) }
	fn port(&self) -> Option<u16> { None }
}

#[allow(dead_code)]
impl Location {
	fn new(location: LocationBlock) -> Result<Self, String> {
		let mut new_location = Location {
			path: PathBuf::from(location.path),
			exact_path: (location.modifier == Some("=".to_string())),
			error_pages: HashMap::new(),
			error_redirect: HashMap::new(),
			client_max_body_size: None,
			return_: None,
			root: None,
			index: None,
			methods: None,
			redirect: None,
			auto_index: false,
			infos: HashMap::new(),
			cgi: HashMap::new(),
		};

		for (name, infos) in location.directives {
			match name.as_str() {
				"root" => {		// ROOT
					let root = Self::extract_root(infos);
					match root {
						Err(e) => return Err(format!("location ({}) : {}", new_location.path.display(), e)),
						Ok(path) => new_location.root = Some(path),
					}

				} "index" => {
					let index = Self::extract_index(infos);
					match index {
						Err(e) => return Err(format!("location ({}) : {}", new_location.path.display(), e)),
						Ok(index) => new_location.index = Some(index),
					}
				} "auto_index" => {
					let auto_index = Self::extract_auto_index(infos);
					match auto_index {
						Err(e) => return Err(format!("location ({}) : {}", new_location.path.display(), e)),
						Ok(is_true) => new_location.auto_index = is_true,
					}
				} "client_max_body_size" => {
					let max_body_size = Self::extract_max_body_size(infos);
					match max_body_size {
						Err(e) => return Err(format!("location ({}) : {}", new_location.path.display(), e)),
						Ok(max_size) => new_location.client_max_body_size = Some(max_size),
					}
				} "cgi" => {
					let (extension, path) = match Self::extract_cgi(infos) {
						Err(e) => return Err(format!("location ({}) : {}", new_location.path.display(), e)),
						Ok(cgi) => cgi,
					};
					new_location.cgi.insert(extension, path);
				} "allowed_methods" => {
					if infos.len() < 1 { return Err(format!("location ({}) : invalid field: allowed_methods", new_location.path.display())) }
					if new_location.methods.is_none() { new_location.methods = Some(Vec::new()) }
	
					new_location.methods.as_mut().unwrap().append(&mut infos.clone());
				} "redirect" => {
					if infos.len() != 1 { return Err(format!("location ({}) : invalid field: redirect", new_location.path.display())) }
					new_location.redirect = Some(infos[0].clone());
				} "return" => {
					new_location.return_ = match Self::extract_return(infos) {
						Err(e) => return Err(format!("location ({}) : {}", new_location.path.display(), e)),
						Ok(res) => Some(res),
					}
				} "error_page" => {
					let (pages, redirect) = Self::extract_error_page(infos)?;
					let hash = &mut new_location.error_pages;
					if pages.is_some() {pages.unwrap().iter().map(|(code, url)| hash.insert(code.to_owned(), url.to_owned())).last();}
					let hash = &mut new_location.error_redirect;
					if redirect.is_some() {redirect.unwrap().iter().map(|(code, url)| hash.insert(code.to_owned(), url.to_owned())).last();}
				} _ => {
					new_location.infos.insert(name, infos);
				}
			}
		}

		Ok(new_location)
	}
	pub fn find(&self, name: String) -> Option<&Vec<String>> {
		self.infos.get(&name)
	}
	pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

