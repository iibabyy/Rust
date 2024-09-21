mod class;

use class::modules::PhoneBook;
use core::time;
use std::{io, thread::sleep};

fn main() {
	let mut input: String = String::new();
	let mut phone = PhoneBook::new();

	println!("");
	println!("What do you want to do today ?");
	loop
	{
		io::stdin().read_line(&mut input).expect("Bad input");
		input = input.trim_end().to_string();
		if input == "add".to_string()
		{
			phone.new_contact();
			println!("What do you want to do today ?");
		} else if String::eq(&input, &"search".to_string())
		{
			phone.search_contact();
			println!("What do you want to do today ?");
		} else if input.eq("exit") {
			println!("\nBIIIP");
			return ;
		}
		else
		{
			println!("What do you mean ???");
		}
		input.clear();
	}
}
