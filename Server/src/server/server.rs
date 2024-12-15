/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   server.rs                                          :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ibaby <ibaby@student.42.fr>                +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/12/15 05:34:36 by ibaby             #+#    #+#             */
/*   Updated: 2024/12/15 06:07:33 by ibaby            ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

use std::{borrow::Cow, collections::HashMap, fmt::Debug, io::Error, net::{IpAddr, SocketAddr}, os::fd::AsFd, path::{Path, PathBuf}, sync::Arc};

use tokio::{fs::File, io::{self, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufStream}, net::{TcpListener, TcpStream}, stream};
use tokio_util::sync::CancellationToken;

use crate::{client::client::Client, request::request::{Method, Request, State}, response::response::{Response, ResponseCode}, traits::config::Config, Parsing::*};

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
	methods: Option<Vec<Method>>,
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

	async fn handle_client(&self, mut stream: TcpStream) -> Result<(), Error> {

		//	getting request
		loop {

			let request = match self.read_until_header_complete(&mut stream).await {
				Ok(request) => request,
				Err(err) => {
					if err.is_none() {	// Request Parsing Error
						eprintln!("invalid header !");
						self.send_error_response_to(&mut stream, ResponseCode::new(400)).await?;
					} else {			// i/o Error
						let err = err.unwrap();
						eprintln!("failed to read header !");
						self.send_error_response_to(&mut stream, ResponseCode::from_error(err.kind())).await?;
					}
					continue ;
				}
			};

			let _ = match self.parse(&request) {
				Ok(()) => (),
				Err(err) => {
					eprintln!("request refused !");
					self.send_error_response_to(&mut stream, err).await?;
					continue ;
				}
			};

			if let Err(err) = self.send_response(&request, &mut stream).await {
				eprintln!("failed to send response: {} !", err.to_string());
				self.send_error_response_to(&mut stream, err).await?;
				continue;
			}


			if request.keep_connection_alive() == false {
				break;
			}
		}

		Ok(())
	}

	async fn read_until_header_complete(&self, mut stream: &mut TcpStream) -> Result<Request, Option<Error>> {
		let buffer = Self::read_from(&mut stream).await?;
		let mut request = match Request::try_from(buffer) {
			Ok(request) => request,
			Err(_) => {
				println!("Error: bad request");
				// self.send_error_response_to(&mut stream);
				return Err(None)
			}
		};

		while request.state().is(State::OnHeader) {
			let buffer = Self::read_from(&mut stream).await?;

			match request.push(buffer) {
				Ok(_) => (),
				Err(_) => {
					println!("Error: bad request");
					return Err(None);
				}
			}
		}

		Ok(request)
	}

	async fn send_response(&self, request: &Request, stream: &mut TcpStream) -> Result<(), ResponseCode> {
		match request.method() {
			&Method::GET => { self.send_get_response(&request, stream).await },
			// &Method::POST => {},
			// &Method::DELETE => {},
			_ => { Err(ResponseCode::new(501)) },		// not implemented
		}
	}

	async fn send_get_response(&self, request: &Request, stream: &mut TcpStream) -> Result<(), ResponseCode> {
		
		// if self.is_cgi(&request){
		// 	// handle CGI GET methods
		// 	todo!();
		// }

		eprintln!("----SENDING RESPONSE----");

		// match self.consume_body(stream).await {
		// 	Ok(_) => (),
		// 	Err(err) => { return Err(ResponseCode::from_error(err.kind())) }
		// }

		eprintln!("root: {}, path: {}", self.root.as_ref().unwrap().display(), request.path().display());
		let mut path = PathBuf::from(format!("{}{}", self.root.clone().unwrap().display(), request.path().display()));
	
		if path.is_dir() {
			path = path.join(PathBuf::from(self.index.as_ref().unwrap()));
		}
	
		eprintln!("----Bro want {}----", path.display());

		let mut response = Response::from_file(200, path.as_path()).await?;

		match response.send_to(stream).await {
			Ok(_) => Ok(()),
			Err(err) => Err(ResponseCode::from_error(err.kind())),
		}

	}

	async fn send_error_response_to(&self, stream: &mut TcpStream, code: ResponseCode ) -> Result<(), Error> {
		todo!()
	}

	async fn consume_body(&self, stream: &mut TcpStream) -> Result<(), Error> {
		let mut buffer = [0; 65536];

		let n = 1;
		loop {
			match stream.try_read(&mut buffer) {
				Ok(0) => break,
				Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
					println!("would block !");
					continue;
				}
				Err(e) => {
					println!("failed to read: {e}");
					return Err(e);
				}
				_ => (),
			}
		}
		Ok(())
	}

}


/*---------------------------------------------------------------*/
/*							   PARSING							 */
/*---------------------------------------------------------------*/

impl Server {
	fn parse(&self, request: &Request) -> Result<(), ResponseCode> {
		self.parse_method(request)?;

		if self.client_max_body_size < request.content_length() {
			return Err(ResponseCode::new(413))
		}

		Ok(())
	}

	fn parse_method(&self, request: &Request) -> Result<(), ResponseCode> {
		if self.methods.is_none() { return Err(ResponseCode::new(405)) }	// No method allowed
		if self.methods.as_ref().unwrap().contains(request.method()) { return Ok(()) }	// Ok

		return match request.method() {
			&Method::UNKNOWN => { Err(ResponseCode::new(405)) },	// Unknown methhod
			_ => { Err(ResponseCode::new(501)) },		// Not implemented
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

	fn is_cgi(&self, request: &Request) -> bool {
		todo!()
	}

}


/*---------------------------------------------------------------*/
/*----------------------[ CONFIG PARSING ]-----------------------*/
/*---------------------------------------------------------------*/


impl Server {
	fn add_directive(&mut self, name: String, infos: Vec<String>) -> Result<(), String> {
		match name.as_str() {
			"root" => {		// ROOT
				self.root = Some(Self::extract_root(infos)?);
			} "listen" => {		// LISTEN
				(self.port, self.default) = Self::extract_listen(infos)?;
			} "server_name" | "server_names" => {		// SERVER_NAME
				if infos.len() < 1 { return Err("invalid field: server_name".to_owned()) }
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
				if infos.len() < 1 { return Err("invalid field: allowed_methods".to_owned()) }
				if self.methods.is_none() { self.methods = Some(Vec::new()) }

				self.methods.as_mut().unwrap().append(&mut infos.iter().map(|method| Method::from(&method[..])).collect());
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
	fn is_method_allowed(&self, method: Method) -> bool { self.methods.as_ref().is_some() && self.methods.as_ref().unwrap().contains(&method) }
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
	methods: Option<Vec<Method>>,
	redirect: Option<String>,
	cgi: HashMap<String, PathBuf>,
}

impl Config for Location {
	fn root(&self) -> &Option<PathBuf> { &self.root }
	fn auto_index(&self) -> bool { self.auto_index }
	fn cgi_path(&self, extension: String) -> Option<&PathBuf> { self.cgi.get::<String>(&extension) }
	fn index(&self) -> &Option<String> { &self.index }
	fn is_method_allowed(&self, method: Method) -> bool { self.methods.as_ref().is_some() && self.methods.as_ref().unwrap().contains(&method) }
	fn port(&self) -> Option<u16> { None }
}

#[allow(dead_code)]
impl Location {
	fn new(location: LocationBlock) -> Result<Self, String> {
		let mut new_location = Location {
			path: PathBuf::from(location.path),
			exact_path: (location.modifier == Some("=".to_owned())),
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
	
					new_location.methods.as_mut().unwrap().append(&mut infos.iter().map(|method| Method::from(&method[..])).collect());
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


