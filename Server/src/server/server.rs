use std::{collections::HashMap, net::{IpAddr, SocketAddr}, os::fd::AsFd, path::PathBuf, sync::Arc};

use tokio::{io::{self, AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream}};

use crate::{client::client::Client, request::request::Request, traits::config::Config, Parsing::*};

/*--------------------------[ SERVER ]--------------------------*/

#[derive(Clone, Debug)]
pub struct Server {
	pub(self) port: Option<u16>,
	pub(self) clients: HashMap<u16, Arc<Client>>,
	pub(self) client_max_body_size: Option<u64>,
	pub(self) default: bool,
	pub(self) root: Option<PathBuf>,
	pub(self) name: Option<Vec<String>>,
	pub(self) index: Option<String>,
	pub(self) return_: Option<(u16, Option<String>)>,
	pub(self) error_pages: HashMap<u16, String>,
	pub(self) error_redirect: HashMap<u16, (Option<u16>, String)>,
	pub(self) auto_index: bool,
	pub(self) methods: Option<Vec<String>>,
	pub(self) infos: HashMap<String, Vec<String>>,
	pub(self) locations: HashMap<PathBuf, Location>,
	pub(self) cgi: HashMap<String, PathBuf>,
}

impl Server {

	pub async fn run(self, ip: IpAddr) -> Result<(), io::Error>{
		if self.port.is_none() {
			println!("------[No port to listen -> no bind]------");
			return Ok(())
		}

		let listener = TcpListener::bind(SocketAddr::new(ip, self.port.unwrap())).await?;
		println!("------[Server listening on {ip}::{}]------", self.port.unwrap());
		let server = Arc::new(self);
		loop {
			let (mut stream, _) = listener.accept().await?;
			println!("------[Connection accepted]------");
			let server_instance = Arc::clone(&server);
			println!("fd: {:#?}", stream.as_fd().try_clone_to_owned().unwrap());
			tokio::spawn( async move {
				server_instance.handle_client(&mut stream).await
			});
		}
	}

	async fn handle_client(&self, stream: &mut TcpStream) {
		let response_code = 200;
		let mut buffer = [0; 65536];
		// let client = 

		//	getting request
		stream.read(&mut buffer).await.expect("failed to receive request !");
		let buffer = String::from_utf8_lossy(&buffer[..]);
		let request = match Request::deserialize(buffer.into_owned()) {
			Ok(request) => request,
			Err((code, str)) => {
				println!("Error: {str}");
				stream.write(format!("HTTP/1.1 {code} OK\r\n\r\n{str}\r\n").as_bytes()).await.expect("failed to send response");
				return ;
			}
		};
		println!("Request received: {:#?}", request);
		
		//	sending RESPONSE
		stream.write(format!("HTTP/1.1 {response_code} OK\r\n\r\nHello from server !\r\n").as_bytes()).await.expect("failed to send response");
	}

}


/*--------------------------[ UTILS ]--------------------------*/

#[allow(dead_code)]
impl Server {

	pub fn init_servers(configs: Vec<ServerBlock>) -> Result<Vec<Self>, String> {
		let mut servers = Vec::new();

		for server_config in configs {
			servers.push(Self::new(server_config)?);
		}

		Ok(servers)
	}

	pub fn new(config: ServerBlock) -> Result<Self, String> {
		let mut serv = Server {
			client_max_body_size: None,
			index: None,
			methods: None,
			return_: None,
			auto_index: false,
			error_pages: HashMap::new(),
			clients: HashMap::new(),
			error_redirect: HashMap::new(),
			infos: HashMap::new(),
			locations: HashMap::new(),
			cgi: HashMap::new(),
			port: None,
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

/*--------------------------[ LOCATIONS ]--------------------------*/

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


