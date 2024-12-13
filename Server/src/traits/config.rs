use std::{collections::HashMap, path::PathBuf};

fn is_redirect_status_code(code: u16) -> bool {
	code == 301 || code == 302 || code == 303 || code == 307
}

#[allow(dead_code)]
pub trait Config {
	fn extract_root(value: Vec<String>) -> Result<PathBuf, String> {
		if value.len() != 1 { return Err("invalid field: root".to_string()) }

		let path = PathBuf::from(&value[0]);
		if path.is_dir() == false { return Err(value[0].clone() + ": invalid root directory") }

		Ok(path)

	}

	fn extract_max_body_size(value: Vec<String>) -> Result<u64, String> {
		if value.len() != 1 { return Err("invalid field: client_max_body_size".to_string()) }

		let num = value[0].parse::<u64>();
		
		return match num {
			Ok(num) => Ok(num),
			Err(e) => Err(format!("invalid field: client_max_body_size: {e}")),
		}
	
	}

	fn extract_error_page(value: Vec<String>) -> Result<(Option<HashMap<u16, String>>, Option<HashMap<u16, (Option<u16>, String)>>), String> {
		if value.is_empty() { return Err(format!("invalid field: error_page: empty")) }

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
				None => { return Err(format!("invalid field: error_page: {} have no corresponding page", code)) },
			};

			if str.starts_with("=") {
				let redirect_code = if str.len() > 1 {
					match str.as_str()[1..].parse::<u16>() {
						Ok(num) => Some(num),
						Err(e) => return Err(format!("invalid field: error_page: {str}: {e}")),
					}
				} else { None };

				let str = match it.next() {
					Some(str) => str,
					None => { return Err(format!("invalid field: error_page: {} have no corresponding redirect", code)) },
				};

				let url = str.to_owned();

				redirect.insert(code, (redirect_code, url));
			} else {
				pages.insert(code, str.clone());
			}
		}

		Ok((
			if pages.is_empty() { None } else {Some(pages)},
			if redirect.is_empty() {None} else {Some(redirect)}
		))

	}

	fn extract_return(value: Vec<String>) -> Result<(u16, Option<String>), String> {
		if value.len() < 1 || value.len() > 2 { return Err("invalid field: return".to_string()) }

		let status_code = match value[0].parse::<u16>() {
			Ok(num) => num,
			Err(e) => { return Err(format!("invalid field: return: {e}")) }
		};

		let url = if value.len() == 2 {
			match is_redirect_status_code(status_code) {
				true => { Some(value[1].clone()) },
				false => {
					println!("'return' field: not redirect code, url ignored ({status_code} {})", value[1]);
					None
				}
			}
		} else { None };

		Ok((status_code, url))

	}

	fn extract_listen(value: Vec<String>) -> Result<(Option<u16>, bool), String> {
		if value.len() < 1 || value.len() > 2 { return Err("invalid field: port".to_string()) }

		let default = value.len() == 2 && value[1] == "default";

		let port = value[0].parse::<u16>();

		return match port {
			Ok(num) => Ok((Some(num), default)),
			Err(err) => Err(format!("invalid field: port: {}", err)),
		};
	}

	fn extract_index(value: Vec<String>) -> Result<String, String> {
		if value.len() != 1 { return Err("invalid field: index".to_string()) }

		Ok(value[0].clone())

	}

	fn extract_auto_index(value: Vec<String>) -> Result<bool, String> {
		if value.len() != 1 { return Err("invalid field: auto_index".to_string()) }

		Ok(value[0] == "on")

	}

	fn extract_cgi(value: Vec<String>) -> Result<(String, PathBuf), String> {
		if value.len() != 2 { return Err("invalid field: cgi".to_string()) }

		let extension = value[0].clone();
		let path = PathBuf::from(&value[1]);

		if path.is_file() == false {
			return Err(format!("invalid field: cgi: invalid path: {}", value[1]))
		} Ok((extension, path))
	}

	//		GETTERS		//
	fn root(&self) -> &Option<PathBuf>;
	fn port(&self) -> Option<u16>;
	fn index(&self) -> &Option<String>;
	fn auto_index(&self) -> bool;
	fn cgi_path(&self, extension: String) -> Option<&PathBuf>;

	//		CHECKERS	//
	fn is_method_allowed(&self, method: String) -> bool;
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