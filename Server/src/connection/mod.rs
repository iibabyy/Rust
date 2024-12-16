mod connection {
    use std::sync::{Arc, Mutex};

    use tokio::{io::BufReader, net::TcpStream};

	pub struct Connection {
		stream: Arc<Mutex<TcpStream>>,
	}

	impl Connection {
		pub fn new(stream: TcpStream) -> Self {
			Connection {
				stream: Arc::new(Mutex::new(stream)),
			}
		}

	}

}

mod listener {
    use std::{fmt::Debug, io, net::IpAddr, sync::{Arc, Mutex}};

    use nom::FindToken;
    use tokio::{io::{AsyncBufReadExt, AsyncReadExt}, net::{TcpListener, TcpStream}};
    use tokio_util::sync::CancellationToken;

    use crate::{request::{self, request::Request}, response::response::Response, server::server::Server};

    use super::{connection::Connection, find_in_u8};

	pub struct Listener {
		listener: TcpListener,
		servers: Vec<Arc<Server>>,
		cancel_token: CancellationToken,
	}

	impl Listener {
		pub async fn new(addr: IpAddr, port: u16, servers: Vec<Server>, cancel_token: CancellationToken) -> io::Result<Self> {
			let socket = format!("{}:{}", addr, port);

			let listener = match TcpListener::bind(socket).await {
				Ok(listener) => listener,
				Err(err) => return Err(err),
			};

			let servers: Vec<Arc<Server>> = servers.iter()
			.map(|serv| Arc::new(serv.to_owned()))
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
							Self::hande_connection(stream, server_instance);
						});
					}
					_ = cancel.cancelled() => {
						println!("------[listener ({:#?}): stop listening]------", self.listener.local_addr());
						return Ok(());
					}
				}
			}
		}

		async fn hande_stream(mut stream: TcpStream, servers: Vec<Arc<Server>>) -> anyhow::Result<()> {
			let mut buffer = [0; 65536];
			let mut raw = String::new();

			loop {
				let n = match stream.read(&mut buffer).await {
					Err(err) => {
						return Ok(eprintln!("Error: {} -> closing conection", err));
					}
					Ok(n) => n,
				};
				
				if n == 0 {
					return Ok(eprintln!("End of stream: closing conection"));
				}	// end of connection

				raw.push_str(std::str::from_utf8(&buffer[..n])?);

				while let Some((header, raw)) = raw.split_once("\r\n\r\n") {

				}



			}

		}

		async fn extract_header(&self, header: &mut String) {
			let response = Response::from
		}

	}
}

fn find_in_u8(big: &[u8], litte: &[u8]) -> bool {
	if litte.len() == 0 { return true }

	big.windows(litte.len()).position(|window| window == litte).is_some()
}
