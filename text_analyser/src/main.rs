
use std::{collections::HashMap, env, fs::*, path::Path, result};

#[allow(dead_code)]
pub struct Stats {
	phrases: u64,
	words: u64,
	chars: u128,
	phrases_len: u32,
	word_len: u32,
	use_words: String,
}

#[allow(dead_code)]
impl Stats {
	fn count_chars(text: &String) -> usize {
		text
		.chars()
		.filter(|c| c.is_whitespace() == false)
		.count()
	}
	fn count_lines(text: &String) -> usize {
		text
		.split(|c| c == '\n')
		.filter(|str| str.is_empty() == false)
		.count()
	}
	fn count_words(text: &String) -> usize
	{
		text
		.split_whitespace()
		.filter(|str| str.contains(|c: char| c.is_alphabetic()))
		.count()
	}
	fn count_phrases(text: &String) -> usize
	{
		text
		.split(|c| c == '.' || c == '?' || c == '!')
		.filter(|phrase| phrase.trim().is_empty() == false)
		.count()
	}
	fn word_len(text: &String) -> usize
	{
		let word_count = Self::count_words(text);
		let mut len: usize = 0;
		let _ = text
		.split_whitespace()
		.filter(|str| str.contains(|c: char| c.is_alphabetic()))
		.map(|word| len += word.len());
		return len / word_count;
	}
	fn most_used_word(text: &String) -> Option<Vec<String>>
	{
		let mut word_count = HashMap::new();
		let words: Vec<String> = text
			.split_whitespace()
			.map(|tmp| tmp.to_string())
			.collect();
	
		if words.len() == 0 {
			return None;
		} for word in words {
			word_count
			.entry(word)
			.and_modify(|count: &mut usize| *count += 1)
			.or_insert(1);
		}
		let mut result = Vec::new();
		let max = *word_count.values().max().unwrap();
		for word in word_count.iter() {
			if *word.1 == max {
				result.push(word.0.clone());
			}
		}
		if result.is_empty() == true {
			return None;
		} else {
			return Some(result);
		}
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
	let mut input: String = String::new();
	let path;
	println!("enter a file path:");
	match std::io::stdin().read_line(&mut input) {
		Ok(_) => {
			path = Path::new(input.trim());	
			if is_valid_path(path) == false {
				return ;
			}
		} Err(_) => {
			println!("bad input");
			return ;
		}
	}

    let file = match read_to_string(path) {
		Ok(content) => content,
		Err(_) => {
			println!("Failed to read the file");
			return ;
		}
	};

	let most_used = Stats::most_used_word(&file).expect("Empty file");
	
	println!("There is {} characters", Stats::count_chars(&file));
	println!("There is {} words", Stats::count_words(&file));
	println!("The average word len is {}", Stats::word_len(&file));
	if most_used.len() == 1 {
		println!("The most used word is '{}'", most_used[0]);
	} else {
		print!("The most used word are");
		let mut i: usize = 1;
		for word in &most_used {
			if i == 1 {
				print!(" '{}'", *word);
			} else if i == most_used.len() {
				print!(" and '{}'", *word);
			} else {
				print!(", '{}'", *word);
			}
			i += 1;
		}
		println!("");
	}
	println!("There is {} phrases", Stats::count_phrases(&file));
	println!("There is {} non-empty lines", Stats::count_lines(&file));
}
