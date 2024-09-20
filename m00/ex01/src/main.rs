mod class;

use class::modules::PhoneBook;
use std::io;

fn main() {
	let mut input: String = String::new();
	let mut phone = PhoneBook::new();

	loop
	{
		println!("What do you want to do today ?");
		io::stdin().read_line(&mut input).expect("Bad input");
		input = input.trim_end().to_string();
		if input == "ADD".to_string()
		{
			phone.new_contact();
		} else if String::eq(&input, &"SEARCH".to_string())
		{
			phone.search_contact();
		} else if input.eq("EXIT") {
			return ;
		}
		input.clear();
	}
}
