use std::sync::Arc;

use tokio::net::TcpStream;

use crate::request::request::Request;


#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Client {
	stream: Arc<TcpStream>,
	request: Option<Arc<Request>>,

}

#[allow(dead_code)]
impl Client {
    pub fn new(stream: TcpStream, request: Request) -> Self {
		Self {
			stream: Arc::new(stream),
			request: Some(Arc::new(request)),
		}
    }

    pub fn stream(&self) -> Arc<TcpStream> {
        self.stream.clone()
    }

    pub fn request(&mut self) -> Option<Arc<Request>> {
		self.request.clone()
    }
	


}
