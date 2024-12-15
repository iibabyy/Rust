use std::{collections::HashMap, fmt::format, io::{Error, ErrorKind}, path::PathBuf};

use nom::InputIter;
use tokio::net::TcpStream;

use crate::{
    request::request::{Method, Request},
    response::response::{Response, ResponseCode},
    server::location::Location,
};

fn is_redirect_status_code(code: u16) -> bool {
    code == 301 || code == 302 || code == 303 || code == 307
}

#[allow(dead_code)]
pub trait Config {
    async fn send_response(
        &self,
        request: &Request,
        stream: &mut TcpStream,
    ) -> Result<(), ResponseCode> {
        let _ = match self.parse(&request) {
            Ok(()) => (),
            Err(err) => return Err(err),
        };

        match request.method() {
            &Method::GET => self.send_get_response(&request, stream).await,
            // &Method::POST => {},
            // &Method::DELETE => {},
            _ => Err(ResponseCode::new(501)), // not implemented
        }
    }

    async fn send_get_response(
        &self,
        request: &Request,
        stream: &mut TcpStream,
    ) -> Result<(), ResponseCode> {
        let mut path = match self.get_final_path(request.path()) {
			Some(path) => path,
			None => return Err(ResponseCode::new(404)),
		};

        if path.is_dir() {
            // send_get_response_directory()
            eprintln!("index: {}", self.index().as_ref().unwrap());
            path = path.join(PathBuf::from(self.index().as_ref().unwrap()));
        }

        // if self.is_cgi(&request){
        // 	// handle CGI GET methods
        // 	todo!();
        // }

        eprintln!("----SENDING RESPONSE----");

        // match self.consume_body(stream).await {
        // 	Ok(_) => (),
        // 	Err(err) => { return Err(ResponseCode::from_error(err.kind())) }
        // }

        let mut response = match Response::from_file(ResponseCode::new(200), path.as_path()).await {
			Err(err) => return Err(ResponseCode::from_error(err)),
			Ok(response) => response,
		};

        match response.send_to(stream).await {
            Ok(_) => Ok(()),
            Err(err) => Err(ResponseCode::from_error(err.kind())),
        }
    }

    fn get_final_path(&self, request: &PathBuf) -> Option<PathBuf> {
        let path = if self.root().is_some() {
			let mut path = self.root().unwrap().to_owned().to_str().unwrap().to_owned();
			path = format!("{}/{}", path, request.to_str().unwrap());
			eprintln!("path: {}", path);
			path
		} else if self.alias().is_some() {
			todo!()
		} else {
			return None
		};

		let mut path = PathBuf::from(path);

		if path.exists() == false {
			return None
		} else if path.is_dir() && self.index().is_some() {
			path.push(self.index().unwrap());
		}

		Some(path)

    }

    async fn send_error_response_to(
        &self,
        stream: &mut TcpStream,
        mut code: ResponseCode,
    ) -> Result<(), ErrorKind> {
        let error_page = match self.error_pages().get(&code.code()) {
            Some(page) => PathBuf::from(page),
            None => {
				code = ResponseCode::new(400);
				PathBuf::from("error_pages/404.html") // TODO: need to have one default page by error (cgi ?)
			}
		};

        let response = self.get_final_path(&error_page);

		let mut response = match response {
			Some(path) => {
				Response::from_file(code, path.as_path()).await?
			}
			None => {
				Response::from_file(ResponseCode::new(400), PathBuf::from("error_pages/404.html").as_path()).await?		// final path not found
			}
		};

		match response.send_to(stream).await {
			Ok(_) => Ok(()),
			Err(err) => Err(err.kind()),
		};

		Ok(())

    }

    fn parse(&self, request: &Request) -> Result<(), ResponseCode> {
        self.parse_method(request)?;

        if self.max_body_size() < request.content_length().as_ref() {
            return Err(ResponseCode::new(413));
        }

        Ok(())
    }

    fn extract_root(value: Vec<String>) -> Result<PathBuf, String> {
        if value.len() != 1 {
            return Err("invalid field: root".to_owned());
        }

        let mut path = PathBuf::from(&value[0]);
        if path.is_dir() == false {
            return Err(value[0].clone() + ": invalid root directory");
        }

        Ok(path)
    }

    fn extract_alias(value: Vec<String>) -> Result<PathBuf, String> {
        if value.len() != 1 {
            return Err("invalid field: root".to_owned());
        }

        let path = PathBuf::from(&value[0]);

        if path.to_str().unwrap().iter_elements().last() != Some('/') {
            return Err(value[0].clone() + ": alias must ends with '/'");
        }

        Ok(path)
    }

    fn extract_max_body_size(value: Vec<String>) -> Result<u64, String> {
        if value.len() != 1 {
            return Err("invalid field: client_max_body_size".to_owned());
        }

        let num = value[0].parse::<u64>();

        return match num {
            Ok(num) => Ok(num),
            Err(e) => Err(format!("invalid field: client_max_body_size: {e}")),
        };
    }

    fn extract_error_page(
        value: Vec<String>,
    ) -> Result<
        (
            Option<HashMap<u16, String>>,
            Option<HashMap<u16, (Option<u16>, String)>>,
        ),
        String,
    > {
        if value.is_empty() {
            return Err(format!("invalid field: error_page: empty"));
        }

        let mut pages = HashMap::new();
        let mut redirect = HashMap::new();

        let mut it = value.iter();
        while let Some(str) = it.next() {
            let code = match str.parse::<u16>() {
                Ok(num) => num,
                Err(e) => return Err(format!("invalid field: error_page: {str}: {e}")),
            };

            let str = match it.next() {
                Some(str) => str,
                None => {
                    return Err(format!(
                        "invalid field: error_page: {} have no corresponding page",
                        code
                    ))
                }
            };

            if str.starts_with("=") {
                let redirect_code = if str.len() > 1 {
                    match str.as_str()[1..].parse::<u16>() {
                        Ok(num) => Some(num),
                        Err(e) => return Err(format!("invalid field: error_page: {str}: {e}")),
                    }
                } else {
                    None
                };

                let str = match it.next() {
                    Some(str) => str,
                    None => {
                        return Err(format!(
                            "invalid field: error_page: {} have no corresponding redirect",
                            code
                        ))
                    }
                };

                let url = str.to_owned();

                redirect.insert(code, (redirect_code, url));
            } else {
                pages.insert(code, str.clone());
            }
        }

        Ok((
            if pages.is_empty() { None } else { Some(pages) },
            if redirect.is_empty() {
                None
            } else {
                Some(redirect)
            },
        ))
    }

    fn extract_return(value: Vec<String>) -> Result<(u16, Option<String>), String> {
        if value.len() < 1 || value.len() > 2 {
            return Err("invalid field: return".to_owned());
        }

        let status_code = match value[0].parse::<u16>() {
            Ok(num) => num,
            Err(e) => return Err(format!("invalid field: return: {e}")),
        };

        let url = if value.len() == 2 {
            match is_redirect_status_code(status_code) {
                true => Some(value[1].clone()),
                false => {
                    println!(
                        "'return' field: not redirect code, url ignored ({status_code} {})",
                        value[1]
                    );
                    None
                }
            }
        } else {
            None
        };

        Ok((status_code, url))
    }

    fn extract_listen(value: Vec<String>) -> Result<(Option<u16>, bool), String> {
        if value.len() < 1 || value.len() > 2 {
            return Err("invalid field: port".to_owned());
        }

        let default = value.len() == 2 && value[1] == "default";

        let port = value[0].parse::<u16>();

        return match port {
            Ok(num) => Ok((Some(num), default)),
            Err(err) => Err(format!("invalid field: port: {}", err)),
        };
    }

    fn extract_index(value: Vec<String>) -> Result<String, String> {
        if value.len() != 1 {
            return Err("invalid field: index".to_owned());
        }

        Ok(value[0].clone())
    }

    fn extract_auto_index(value: Vec<String>) -> Result<bool, String> {
        if value.len() != 1 {
            return Err("invalid field: auto_index".to_owned());
        }

        match &value[0][..] {
            "on" => Ok(true),
            "off" => Ok(false),
            _ => Err(format!(
                "invalid field: auto_index: expected 'on' or 'off', found {}",
                value[0]
            )),
        }
    }

    fn extract_cgi(value: Vec<String>) -> Result<(String, PathBuf), String> {
        if value.len() != 2 {
            return Err("invalid field: cgi".to_owned());
        }

        let extension = value[0].clone();
        let path = PathBuf::from(&value[1]);

        if path.is_file() == false {
            return Err(format!("invalid field: cgi: invalid path: {}", value[1]));
        }
        Ok((extension, path))
    }

    //		GETTERS		//
    fn auto_index(&self) -> bool;
    fn root(&self) -> Option<&PathBuf>;
    fn alias(&self) -> Option<&PathBuf>;
    fn port(&self) -> Option<&u16>;
    fn index(&self) -> Option<&String>;
    fn max_body_size(&self) -> Option<&u64>;
    fn name(&self) -> Option<&Vec<String>>;
    fn path(&self) -> &PathBuf;
    fn methods(&self) -> Option<&Vec<Method>>;
    fn cgi(&self) -> &HashMap<String, PathBuf>;
    fn error_pages(&self) -> &HashMap<u16, String>;
    fn error_redirect(&self) -> &HashMap<u16, (Option<u16>, String)>;
    fn locations(&self) -> Option<&HashMap<PathBuf, Location>>;
    fn return_(&self) -> Option<&(u16, Option<String>)>;
    fn internal(&self) -> bool;
    fn is_location(&self) -> bool;

    //		CHECKERS	//

    fn parse_method(&self, request: &Request) -> Result<(), ResponseCode> {
        let methods = self.methods();
		if methods.is_none() {
			eprintln!("NO METHODS ALLOWED !");
            return Err(ResponseCode::new(405));
        } // No method allowed
        if !methods.as_ref().unwrap().contains(request.method()) {
			eprintln!("METHOD NOT ALLOWED !");
            return Err(ResponseCode::new(405));
        } // Ok

        return match request.method() {
            // check if implemented (wip)
            &Method::GET => Ok(()),
            _ => Err(ResponseCode::new(501)), // Not implemented
        };
    }

    fn is_general_field(&self, field: String) -> bool {
        let general = vec![
            "cgi",
            "index",
            "auto_index",
            "allowed_methods",
            "root",
            "listen",
        ];

        return general.contains(&field.as_str());
    }
}
