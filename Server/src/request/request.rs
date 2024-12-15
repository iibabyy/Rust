

use std::{collections::HashMap, fmt::Display, path::PathBuf, str::Split};

use tokio::{io::AsyncReadExt, net::{TcpSocket, TcpStream}};

use crate::traits::http_message::HttpMessage;

/*------------------------------------------------------------------------------------*/
/*										REQUEST										  */
/*------------------------------------------------------------------------------------*/

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Request {
	method: Method,
	http_version: String,
	path: PathBuf,
	accept: Option<String>,
	host: Option<String>,
	headers: HashMap<String, String>,
	content_length: Option<u64>,
	raw_body: Option<String>,
	raw_header: String,
	state: State,
	keep_connection_alive: bool,
}

impl TryFrom<String> for Request {
	type Error = u16;
	fn try_from(value: String) -> Result<Request, Self::Error> {
		let mut request = Request {
			method: Method::UNDEFINED,
			path: PathBuf::new(),
			http_version: String::new(),
			accept: None,
			host: None,
			headers: HashMap::new(),
			content_length: None,
			raw_body: None,
			raw_header: String::new(),
			keep_connection_alive: true,
			state: State::Undefined,
		};

		request.push(value)?;
		Ok(request)
	}
}

impl HttpMessage for Request {}

impl Request {
	pub fn push(&mut self, request: String) -> Result<(), u16> {
		if self.state == State::OnBody { todo!() }

		let (header, body) = match request.split_once("\r\n\r\n") {
			None => {	// Header not finished
				self.raw_header.push_str(&request);
				return Ok(());
			}
			Some((header, body)) => (header, body)
		};
		// Header complete
		self.raw_header.push_str(header);
		self.state = State::OnHeader;
		if body.is_empty() == false {
			self.raw_body = Some(body.to_owned());
		}

		self.deserialize()?;
		self.raw_header.clear();
		self.state = if self.raw_body.is_some() { State::OnBody } else { State::Finished };

		Ok(())
	}

	fn deserialize(&mut self) -> Result<(), u16> {
		let temp = self.raw_header.clone();
		let mut headers = temp.split("\r\n");

		let first_line = headers.next();
		if first_line.is_none() {
			println!("empty header");
			return Err(400)
		}
	
		self.parse_first_line(first_line.unwrap())?;
		self.parse_other_lines(headers)?;

		Ok(())

	}

	fn parse_other_lines(&mut self, headers: Split<'_, &str>) -> Result<(), u16> {

		for header in headers {
			if header.is_empty() { break; }		// end of header

			let split = header.split_once(":");
			if split.is_none() {
				eprintln!("invalid header: {}", header);
				return Err(400)
			}

			let name =  split.unwrap().0;
			let value = split.unwrap().1;

			match name {
				"Host" => {
					if	self.host.is_none() {
						self.host = Some(value.to_owned())
					} else {
						println!("duplicate header: Host");
						return Err(400)
					}
				}
				"Connection" => {
					if value == "close" {self.keep_connection_alive = false}
				}
				"Accept" => {
					if	self.accept.is_none()	{self.accept = Some(value.to_owned())}	// set if don't exists
					else						{self.accept = Some(format!("{} {value}", self.accept.as_ref().unwrap()))} // concat a space (' ') and the value if already exists 
				}
				_ => {
					self.headers.entry(name.to_owned())	// finding key name
					.and_modify(|val|	// modify if exists
						val.push_str(format!("  {value}").as_str())
					).or_insert(value.to_owned());	// else, insert
				}
			}
		}

		Ok(())
	}
	
	fn parse_first_line(&mut self, line: &str) -> Result<(), u16> {
		let split: Vec<&str> = line.split_whitespace().collect();

		if split.len() != 3 {
			println!("invlid header: first line invalid: [{line}]");
			return Err(400)
		}	// Bad Request
		
		let method = split[0];
		self.method = match Method::try_from(method) {
			Ok(method) => method,
			Err(e) => {
				format!("invlid header: {e}");
				return Err(501)
			}
		};

		self.path = PathBuf::from(split[1]);
		self.http_version = split[2].to_owned();
		Ok(())
	}

	pub fn get(&self, header: String) -> Option<&String> {
		match self.headers.get(&header) {
			None => None,
			Some(value) => Some(value),
		}
	}

	pub fn state(&self) -> &State {
        &self.state
    }

	pub fn method(&self) -> &Method {
        &self.method
    }

	pub fn path(&self) -> &PathBuf {
        &self.path
    }

	pub fn accept(&self) -> Option<&String> {
        self.accept.as_ref()
    }

	pub fn host(&self) -> Option<&String> {
        self.host.as_ref()
    }

	pub fn content_length(&self) -> Option<u64> {
        self.content_length
    }

	pub fn keep_connection_alive(&self) -> bool {
        self.keep_connection_alive
    }

pub fn http_version(&self) -> &str {
        &self.http_version
    }

}

/*------------------------------------------------------------------------------------*/
/*										METHOD										  */
/*------------------------------------------------------------------------------------*/

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Method {
	UNDEFINED,
	GET,
	POST,
	DELETE,
	OPTIONS,
	HEAD,
	PUT,
	PATCH,
	TRACE,
	CONNECT,
	UNKNOWN,
}

impl Default for Method {
    fn default() -> Self {
        Self::UNDEFINED
    }
}

impl From<&str> for Method {
	fn from(method: &str) -> Self {
		match method {
			"GET" => Method::GET,
			"POST" => Method::POST,
			"DELETE" => Method::DELETE,
			"OPTIONS" => Method::OPTIONS,
			"HEAD" => Method::HEAD,
			"PUT" => Method::PUT,
			"CONNECT" => Method::CONNECT,
			"PATCH" => Method::PATCH,
			"TRACE" => Method::TRACE,
			_ => Method::UNKNOWN,
		}
	}
}

impl <'a>TryInto<&'a str> for Method {
	type Error = ();
	fn try_into(self) -> Result<&'a str, Self::Error> {
		match self {
			Method::GET => Ok("GET"),
			Method::POST => Ok("POST"),
			Method::DELETE => Ok("DELETE"),
			Method::HEAD => Ok("HEAD"),
			Method::PUT => Ok("PUT"),
			Method::CONNECT => Ok("CONNECT"),
			Method::PATCH => Ok("PATCH"),
			Method::TRACE => Ok("TRACE"),
			Method::OPTIONS => Ok("OPTIONS"),
			Method::UNDEFINED | Method::UNKNOWN => Err(()),
		}
		
	}
}

/*------------------------------------------------------------------------------------*/
/*										STATE										  */
/*------------------------------------------------------------------------------------*/

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum State {
	Undefined,
	OnHeader,
	OnBody,
	Finished,
}

impl State {
	pub fn is(&self, other: Self) -> bool {
		self.eq(&other)
	}
	pub fn is_not(&self, other: Self) -> bool {
		self.eq(&other)
	}
}

/*-----------------ERROR CODES-----------------*/

// codes_responses[100] = "Continue";
// codes_responses[101] = "Switching Protocols";
// codes_responses[102] = "Processing";
// codes_responses[103] = "Early Hints";
// codes_responses[200] = "OK";
// codes_responses[201] = "Created";
// codes_responses[202] = "Accepted";
// codes_responses[203] = "Non-Authoritative Information";
// codes_responses[204] = "No Content";
// codes_responses[205] = "Reset Content";
// codes_responses[206] = "Partial Content";
// codes_responses[207] = "Multi-Status";
// codes_responses[208] = "Already Reported";
// codes_responses[210] = "Content Different";
// codes_responses[226] = "IM Used";
// codes_responses[300] = "Multiple Choices";
// codes_responses[301] = "Moved Permanently";
// codes_responses[302] = "Found";
// codes_responses[303] = "See Other";
// codes_responses[304] = "Not Modified";
// codes_responses[305] = "Use Proxy";
// codes_responses[307] = "Temporary Redirect";
// codes_responses[308] = "Permanent Redirect";
// codes_responses[310] = "Too many Redirects";
// codes_responses[400] = "Bad Request";
// codes_responses[401] = "Unauthorized";
// codes_responses[402] = "Payment Required";
// codes_responses[403] = "Forbidden";
// codes_responses[404] = "Not Found";
// codes_responses[405] = "Method Not Allowed";
// codes_responses[406] = "Not Acceptable";
// codes_responses[407] = "Proxy Authentication Required";
// codes_responses[408] = "Request Time-out";
// codes_responses[409] = "Conflict";
// codes_responses[410] = "Gone";
// codes_responses[411] = "Length Required";
// codes_responses[412] = "Precondition Failed";
// codes_responses[413] = "Request Entity Too Large";
// codes_responses[414] = "Request-URI Too Long";
// codes_responses[415] = "Unsupported Media Type";
// codes_responses[416] = "Requested range unsatisfiable";
// codes_responses[417] = "Expectation failed";
// codes_responses[418] = "I'm a teapot";
// codes_responses[419] = "Page expired";
// codes_responses[421] = "Bad mapping / Misdirected Request";
// codes_responses[422] = "Unprocessable entity";
// codes_responses[423] = "Locked";
// codes_responses[424] = "Method failure";
// codes_responses[425] = "Too Early";
// codes_responses[426] = "Upgrade Required";
// codes_responses[427] = "Invalid digital signature";
// codes_responses[428] = "Precondition Required";
// codes_responses[429] = "Too Many Requests";
// codes_responses[431] = "Request Header Fields Too Large";
// codes_responses[449] = "Retry With";
// codes_responses[450] = "Blocked by Windows Parental Controls";
// codes_responses[451] = "Unavailable For Legal Reasons";
// codes_responses[456] = "Unrecoverable Erstatus()";
// codes_responses[500] = "Internal Server Error";
// codes_responses[501] = "Method Not Implemented";
// codes_responses[505] = "HTTP Version not supported";
// codes_responses[506] = "Variant Also Negotiates";
// codes_responses[507] = "Insufficient storage";
// codes_responses[508] = "Loop detected";
// codes_responses[509] = "Bandwidth Limit Exceeded";
// codes_responses[510] = "Not extended";
// codes_responses[511] = "Network authentication required";
// codes_responses[520] = "Unknown Error";
// codes_responses[521] = "Web Server Is Down";
// codes_responses[522] = "Connection Timed Out";
// codes_responses[523] = "Origin Is Unreachable";
// codes_responses[524] = "A Timeout Occurred";
// codes_responses[525] = "SSL Handshake Failed";
// codes_responses[526] = "Invalid SSL Certificate";
// codes_responses[527] = "Railgun Error";


/*-----------------CONTENT-TYPES-----------------*/

// contentTypes[".txt"] = "text/plain";
// contentTypes[".html"] = "text/html";
// contentTypes[".htm"] = "text/html";
// contentTypes[".css"] = "text/css";
// contentTypes[".js"] = "application/javascript";
// contentTypes[".json"] = "application/json";
// contentTypes[".xml"] = "application/xml";
// contentTypes[".pdf"] = "application/pdf";
// contentTypes[".zip"] = "application/zip";
// contentTypes[".gz"] = "application/gzip";
// contentTypes[".doc"] = "application/msword";
// contentTypes[".docx"] = "application/vnd.openxmlformats-officedocument.wordprocessingml.document";
// contentTypes[".xls"] = "application/vnd.ms-excel";
// contentTypes[".xlsx"] = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";
// contentTypes[".ppt"] = "application/vnd.ms-powerpoint";
// contentTypes[".pptx"] = "application/vnd.openxmlformats-officedocument.presentationml.presentation";
// contentTypes[".jpg"] = "image/jpeg";
// contentTypes[".jpeg"] = "image/jpeg";
// contentTypes[".png"] = "image/png";
// contentTypes[".gif"] = "image/gif";
// contentTypes[".webp"] = "image/webp";
// contentTypes[".svg"] = "image/svg+xml";
// contentTypes[".mp3"] = "audio/mpeg";
// contentTypes[".ogg"] = "audio/ogg";
// contentTypes[".wav"] = "audio/wav";
// contentTypes[".mp4"] = "video/mp4";
// contentTypes[".webm"] = "video/webm";
// contentTypes[".ogv"] = "video/ogg";
// contentTypes[".tar"] = "application/x-tar";
// contentTypes[".7z"] = "application/x-7z-compressed";
// contentTypes[".rar"] = "application/x-rar-compressed";
// contentTypes[".md"] = "text/markdown";
// contentTypes[".rtf"] = "application/rtf";
// contentTypes[".sh"] = "application/x-sh";
// contentTypes[".py"] = "application/x-python";
// contentTypes[".jar"] = "application/x-java-archive";
