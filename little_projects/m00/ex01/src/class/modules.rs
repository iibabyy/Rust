use std::{io};

pub struct Contact {
	pub	first_name: String,
	pub last_name: String,
	pub nickname: String,
	pub phone_number: String,
	pub darkest_secret: String,
}

pub struct PhoneBook {
	pub contact: Vec<Contact>,
}

impl Contact
{
	pub fn new() -> Self
	{
		let  first_name = String::new();
		let  last_name = String::new();
		let  nickname = String::new();
		let  phone_number = String::new();
		let  darkest_secret = String::new();
		Contact {
			first_name,
			last_name,
			nickname,
			phone_number,
			darkest_secret,
		}
	}
	pub fn print_contact(&self)
	{
		println!("First name: {}", self.first_name);
		println!("Last name: {}", self.last_name);
		println!("Nickname: {}", self.nickname);
		println!("Phone number: {}", self.phone_number);
		println!("Darkest secret: {}", self.darkest_secret);
		println!("");
	}
	pub fn changefirst_name(&mut self, newfirst_name: String)
	{
		self.first_name = newfirst_name;
	}
	pub fn changelast_name(&mut self, newlast_name: String)
	{
		self.last_name = newlast_name;
	}
	pub fn changenickname(&mut self, newnickname: String)
	{
		self.nickname = newnickname;
	}
	pub fn changephone_number(&mut self, newphone_number: String)
	{
		self.phone_number = newphone_number;
	}
	pub fn changesecret(&mut self, newsecret: String)
	{
		self.darkest_secret = newsecret;
	}
}

impl PhoneBook
{
	pub fn new() -> Self
	{
		let contact = Vec::new();
		PhoneBook {
			contact,
		}
	}
	pub fn new_contact(&mut self)
	{
		let mut input: String;
		let mut contact = Contact::new();
		
		input = get_input("First Name");
		Contact::changefirst_name(&mut contact, input);
		input = get_input("Last Name");
		Contact::changelast_name(&mut contact, input);
		input = get_input("nickname");
		Contact::changenickname(&mut contact, input);
		input = get_input("Phone Number");
		while is_num(&input) == false || input.len() != 10 as usize{
			input = get_input("Enter a valid phone number");
		}
		Contact::changephone_number(&mut contact, input);
		input = get_input("Darkest secret");
		Contact::changesecret(&mut contact, input);
		self.contact.insert(0, contact);
	}
	pub fn search_contact(&self)
	{
		let mut input = String::new();

		if self.contact.len() < 1 {
			println!("no contact found, try to add one !");
			return ;
		}
		self.print_book();
		println!("Enter the index");
		io::stdin().read_line(&mut input).expect("Bad input");
		let input: usize = match input.trim().parse() {
			Ok(num) => num,
			Err(_) => {
				println!("try with a better input maybe...");
				return ;
			}
		};
		println!("");
		if input > self.contact.len() - 1 {
			println!("retry with a better index maybe...\n");
			return ;
		}
		self.contact[input].print_contact();
	}
	fn print_book(&self)
	{
		let mut i: i32 = 0;
		
		println!("");
		for contact in self.contact.iter()
		{
			println!("{}|{}|{}|{}",
				i,
				max_10(contact.first_name.clone().trim().to_string()),
				max_10(contact.last_name.clone().trim().to_string()),
				max_10(contact.nickname.clone().trim().to_string()));
			i += 1;
			if i >= 8 {
				break ;
			}
		}
		println!("");
	}
}

fn is_num(str: &String) -> bool
{
	for i in str.chars()
	{
		if i.is_ascii_digit() == false {
			return false;
		}
	}
	true
}

fn max_10(tmp: String) -> String
{
	let mut str: String = String::from(tmp);

	if str.len() > 10 {
		str.truncate(9);
		str.push_str(".");
	} else if str.len() < 10{

		for _i in str.len()..10
		{
			str.push(' ');
		}
	}
	str
}

fn get_input(prompt: &str) -> String
{
	let mut input = String::new();

	println!("{}:", prompt);
	io::stdin().read_line(&mut input).expect("Error while reading input");
	input.trim().to_string()
}