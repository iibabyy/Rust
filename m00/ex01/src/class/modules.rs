use std::{io, iter};

pub struct contact {
	pub	FirstName: String,
	pub LastName: String,
	pub Nickname: String,
	pub PhoneNumber: String,
	pub DarkestSecret: String,
}

pub struct PhoneBook {
	pub contact: Vec<contact>,
}

impl contact
{
	pub fn new() -> Self
	{
		let  FirstName = String::new();
		let  LastName = String::new();
		let  Nickname = String::new();
		let  PhoneNumber = String::new();
		let  DarkestSecret = String::new();
		contact {
			FirstName,
			LastName,
			Nickname,
			PhoneNumber,
			DarkestSecret,
		}
	}
	pub fn ChangeFirstName(&mut self, NewFirstName: String)
	{
		self.FirstName = NewFirstName;
	}
	pub fn ChangeLastName(&mut self, NewLastName: String)
	{
		self.LastName = NewLastName;
	}
	pub fn ChangeNickname(&mut self, NewNickname: String)
	{
		self.Nickname = NewNickname;
	}
	pub fn ChangePhoneNumber(&mut self, NewPhoneNumber: String)
	{
		self.PhoneNumber = NewPhoneNumber;
	}
	pub fn ChangeSecret(&mut self, NewSecret: String)
	{
		self.DarkestSecret = NewSecret;
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
		let mut contact = contact::new();
		
		input = PhoneBook::get_input("First Name");
		contact::ChangeFirstName(&mut contact, input);
		input = PhoneBook::get_input("Last Name");
		contact::ChangeLastName(&mut contact, input);
		input = PhoneBook::get_input("Nickname");
		contact::ChangeNickname(&mut contact, input);
		input = PhoneBook::get_input("Phone Number");
		contact::ChangePhoneNumber(&mut contact, input);
		input = PhoneBook::get_input("Darkest Secret");
		contact::ChangeSecret(&mut contact, input);
		self.contact.push(contact);
	}
	pub fn search_contact()
	{

	}
	fn get_input(prompt: &str) -> String
	{
		let mut input = String::new();

		println!("{}:", prompt);
		io::stdin().read_line(&mut input).expect("Error while reading input");
		input
	}
	fn print_book(&self)
	{
		for i in self.contact.iter()
		{
			// print
		}
	}
	fn print_contact(contact: contact, i)
	{
		
	}
	fn print_max_10(mut str: String)
	{
		if str.len() > 10 {
			str.truncate(9);
			str.push_str(".");
		}
		else {
			let mut len = str.chars();
			len = 10 - len;
			for i in len
			{
				str.push(ch);
			}
		}
	}
}
