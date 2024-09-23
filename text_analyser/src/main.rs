use std::{collections::HashMap, env, fs::*, path::Path};

pub struct Stats {
	phrases: u64,
	words: u64,
	chars: u128,
	phrases_len: u32,
	word_len: u32,
	use_words: String,
}

impl Stats {
	fn count_chars(text: String) -> usize {
		text.chars().filter(|c| c.is_whitespace() == false).count()
	}
	fn count_words(text: String) -> usize
	{
		text.split_whitespace().enumerate().count()
	}
	fn count_phrases(text: String) -> usize
	{
		text.split(|c| c == '.' || c == '?' || c == '!')
		.filter(|phrase| phrase.trim().is_empty() == false)
		.count()
	}
	
	fn word_len(text: String) -> usize
	{
		let word_count = Self::count_words(text.clone());
		let mut len: usize = 0;
		let _ = text.split_whitespace().map(|word|len += word.len());
		return len / word_count;
	}
	fn most_used_words(text: String) -> Result<Option<String>, String>
	{
		let words: Vec<String> = text
			.split_whitespace()
			.map(|tmp| tmp.to_string())
			.collect();
		let mut word_count = HashMap::new();
	
		if words.len() == 0 {
			return Ok(None);
		} for word in words {
			word_count
			.entry(word)
			.and_modify(|count: &mut usize| *count += 1)
			.or_insert(1);
		}
		for word in word_count.iter() {
			if *word.1 == *word_count.values().max().unwrap() {
				return Ok(Some((*word.0.clone()).to_string()));
			}
		}
		Ok(None)
	}
}

fn is_valid_path(path: &Path) -> bool {
	if path.exists() == false {
		println!("Given path does not exists");
		return false;
	} else if path.is_file() == false {
		println!("Only files are accepted");
		return false;
	}
	true
}

fn main() {
	let args: Vec<String> = env::args().collect();
	if args.len() != 2 {
		println!("Enter a file path")
	}

	let path = Path::new(args[1].as_str());
	if is_valid_path(path) == false {
		return ;
	}
    let file = match read_to_string(path) {
		Ok(content) => content,
		Err(_) => {
			println!("Failed to read the file");
			return ;
		}
	};
}
