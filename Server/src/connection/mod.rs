mod connection {
    use std::sync::{Arc, Mutex};

    use tokio::{io::BufReader, net::TcpStream};

	pub struct Connection {
		stream: TcpStream,
		reader: Option<Arc<Mutex<BufReader<TcpStream>>>>,
	}

	impl Connection {
		pub fn new(stream: TcpStream) -> Self {
			Connection {
				stream: stream,
				reader: None,
			}
		}

	pub fn reader(&self) -> BufReader<TcpStream> {
		if self.reader.is_none() {
			self.reader = Some(BufReader::new(self.stream));
		}

		let reader = self.reader.unwrap().clone().lock().unwrap();

		if reader.

	}

	pub async fn readable(&self) -> std::io::Result<()> {
        self.stream.readable().await
    }

}

mod listener {
    use std::{fmt::Debug, io, net::IpAddr, sync::{Arc, Mutex}};

    use tokio::{net::{TcpListener, TcpStream}};
    use tokio_util::sync::CancellationToken;

    use crate::{request::{self, request::Request}, server::server::Server};

    use super::{connection::Connection, Connection};

	pub struct Listener {
		listener: TcpListener,
		servers: Vec<Arc<Mutex<Server>>>,
		cancel_token: CancellationToken,
	}

	impl Listener {
		pub async fn new(addr: IpAddr, port: u16, servers: Vec<Server>, cancel_token: CancellationToken) -> io::Result<Self> {
			let socket = format!("{}:{}", addr, port);

			let listener = match TcpListener::bind(socket).await {
				Ok(listener) => listener,
				Err(err) => return Err(err),
			};

			let servers: Vec<Arc<Mutex<Server>>> = servers.iter()
			.map(|serv| Arc::new(Mutex::new(serv.to_owned())))
			.collect();

			Ok(
				Listener {
					servers,
					listener,
					cancel_token,
				}
			)
		}

		pub async fn listen(self) -> io::Result<()> {

			loop {
				let cancel = self.cancel_token.clone();
				tokio::select! {
					Ok((stream, addr)) = self.listener.accept() => {
						println!("------[Connection accepted: {addr}]------");
						let server_instance = self.servers.clone();
						tokio::spawn( async move {

						});
					}
					_ = cancel.cancelled() => {
						println!("------[listener ({:#?}): stop listening]------", self.listener.local_addr());
						return Ok(());
					}
				}
			}
		}

		async fn hande_connection(stream: TcpStream, servers: Vec<Arc<Mutex<Server>>>) {
			let stream = Connection::new(stream);
			
			loop {

			}
		}

	}

	async fn read_header_from(mut stream: &mut Connection) -> io::Result<Vec<String>> {
		let mut headers = vec![];
		let mut size = 0;

		while size < 4096 {
			stream.readable().await;
			let reader = stream.reader();
			let mut lines = reader.lines();
			
			while let Some(line) = lines.next_line().await? {
				if line.is_empty() { return Ok(headers) }
				size += line.as_bytes().len();
				headers.push(line);
			}
		}

		Err(io::Error::new(io::ErrorKind::FileTooLarge, "header too large: expected less than 4096 bytes"))

	}

}
