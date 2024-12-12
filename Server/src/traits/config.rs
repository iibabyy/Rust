use std::{num::ParseIntError, path::{Path, PathBuf}};

#[allow(dead_code)]
pub trait Config {
	fn extract_root(value: Vec<String>) -> Result<PathBuf, String> {
		if value.len() != 1 { return Err("invalid field: root".to_string()) }

		let path = PathBuf::from(&value[0]);
		if path.is_dir() == false { return Err(value[0].clone() + ": invalid root directory") }

		Ok(path)

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