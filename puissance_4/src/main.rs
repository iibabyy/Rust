use std::{io, ops::{Index, Range}};

pub struct Game {
	grill: Vec<String>,
}

impl Game {
	pub fn new() -> Game
	{
		let mut grill = vec![
			"000000".to_string(),
			"000000".to_string(),
			"000000".to_string(),
			"000000".to_string(),
			"000000".to_string()];
		Game {
			grill,
		}
	}
	pub fn print_grill(&self)
	{
		let mut i = self.grill.len();

		println!("1  2  3  4  5  6");
		println!("⇩  ⇩  ⇩  ⇩  ⇩  ⇩");
		while i != 0 {
			for c in self.grill[i - 1].chars() {
				print!("{}  ", c);
			}
			println!("");
			i -= 1;
		}
	}
	pub fn add_token(&mut self, column: usize, player: char)
	{
		if column > self.grill[0].len() {
			println!("Enter a valid column !");
			return ;
		}
		let lowest_row = self.lowest_empty_row(column);
		if lowest_row == -1 {
			println!("column full");
			return ;
		}
		self.grill[lowest_row as usize].chars().nth(column).replace(player);
	}
	fn lowest_empty_row(&self, column: usize) -> i32
	{
		let mut	i: usize = 0;
		
		for row in self.grill.iter()
		{
			if row.chars().nth(column) == Some('0') {
				return i as i32;
			}
			i += 1;
		}
		-1
	}
}

fn main() {
    let mut game = Game::new();
	let mut input = String::new();
	let mut player: char = 'R';

	loop {
		println!("player {}: enter a column\n", player);
		game.print_grill();
		std::io::stdin().read_line(&mut input);
		let input: usize = match input.trim().parse(){
			Ok(num) => num,
			Err(_) => {
				println!("enter a valid input");
				continue ;
			}
		};
		game.add_token(input, player);

		if player == 'R' {
			player = 'B';
		} else {
			player = 'R';
		}
	}
}
