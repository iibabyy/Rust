mod class;

use class::modules::PhoneBook;
use core::time;
use std::{io, thread::sleep};

fn main() {
	let mut input: String = String::new();
	let mut phone = PhoneBook::new();

	loop
	{
		println!("What do you want to do today ?");
		io::stdin().read_line(&mut input).expect("Bad input");
		input = input.trim_end().to_string();
		if input == "add".to_string()
		{
			phone.new_contact();
		} else if String::eq(&input, &"search".to_string())
		{
			phone.search_contact();
		} else if input.eq("exit") {
			return ;
		}
		else
		{
			println!("\nWhat do you mean ???");
			sleep(time::Duration::from_secs(2));
		}
		input.clear();
	}
}
