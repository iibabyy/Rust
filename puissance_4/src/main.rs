use std::{fs::remove_dir, io, ops::{Index, Range}, process::exit};

use colored::Colorize;

pub struct Game {
	grill: Vec<String>,
}

enum Direction {
	n ,
	s ,
	w ,
	e ,
	nw ,
	ne ,
	sw ,
	se ,
}

impl Game {
	pub fn new() -> Game
	{
		let mut grill = vec![
			"0000000".to_string(),
			"0000000".to_string(),
			"0000000".to_string(),
			"0000000".to_string(),
			"0000000".to_string(),
			"0000000".to_string()];
		Game {
			grill,
		}
	}
	pub fn print_grill(&self)
	{
		let mut i = self.grill.len();

		println!("  1   2   3   4   5   6   7  ");
		println!("  ⇩   ⇩   ⇩   ⇩   ⇩   ⇩   ⇩  \n");
		while i != 0 {
			print!("| ");
			for c in self.grill[i - 1].chars() {
				if c == '0' {
					print!("  | ")
				} else {
					print!("{} | ", c);
				}
			}
			println!("");
			i -= 1;
		}
		println!("¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯\n");
	}
	pub fn add_token(&mut self, column: usize, player: char)
	{
		if column > self.grill[0].len() {
			println!("{}", "Enter a valid column !".red());
			return ;
		}
		let lowest_row = self.lowest_empty_row(column);
		if lowest_row == -1 {
			println!("{}", "column full".red());
			return ;
		}
		self.grill[lowest_row as usize].remove(column);
		self.grill[lowest_row as usize].insert(column, player);
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
	fn is_finished(&mut self) -> bool
	{
		let mut col: usize;
		let mut row: usize = 0;
		
		for r in &self.grill {
			col = 0;
			for c in r.chars() {
				let directions = vec![Direction::s, Direction::n, Direction::w, Direction::e, Direction::sw, Direction::se, Direction::nw, Direction::ne];

				if c == '0' {
					continue ;
				} else {
					for dir in &directions {
						if self.count_aligned(c, dir, col, row) >= 3 {
							
							self.hilight_winner(col, row, dir, c);
							self.print_grill();
							println!("Player {} wins !!!", match c {
								'r' => "red".red(),
								'b' => "blue".blue(),
								_ => "?".white(),
							});
							return  true;
						}
					}
				}
				col += 1;
			}
			row += 1;
		}
		return false;
	}
	fn count_aligned(&self, player: char, dir: &Direction, col: usize, row: usize) -> usize
	{
		match dir {
			Direction::s => {
				if row == 0 { return 0 }
				if player == self.grill[row - 1].chars().nth(col).unwrap() {
					return 1 + self.count_aligned(player, dir, col, row - 1);
				} else {
					return 0;
				}
			}
			Direction::n => {
				if row + 1 >= self.grill.len() { return 0 }

				if player == self.grill[row + 1].chars().nth(col).unwrap() {
					return 1 + self.count_aligned(player, dir, col, row + 1);
				} else {
					return 0;
				}
			}
			Direction::e => {
				if col + 1 >= self.grill[0].len() {
					return 0;
				}else if player == self.grill[row].chars().nth(col + 1).unwrap() {
					return 1 + self.count_aligned(player, dir, col + 1, row);
				} else {
					return 0;
				}
			}
			Direction::w => {
				if col == 0 { return 0 }

				if player == self.grill[row].chars().nth(col - 1).unwrap() {
					return 1 + self.count_aligned(player, dir, col - 1, row);
				} else {
					return 0;
				}
			}
			Direction::se => {
				if col + 1 >= self.grill[0].len() || row == 0 { return 0 }

				if player == self.grill[row - 1].chars().nth(col + 1).unwrap() {
					return 1 + self.count_aligned(player, dir, col + 1, row - 1);
				} else {
					return 0;
				}
			}
			Direction::sw => {
				if row == 0 || col == 0 { return 0 }

				if player == self.grill[row - 1].chars().nth(col - 1).unwrap() {
					return 1 + self.count_aligned(player, dir, col - 1, row - 1);
				} else {
					return 0;
				}
			}
			Direction::ne => {
				if col + 1 >= self.grill[0].len() || row + 1 >= self.grill.len() { return 0 }

				if player == self.grill[row + 1].chars().nth(col + 1).unwrap() {
					return 1 + self.count_aligned(player, dir, col + 1, row + 1);
				} else {
					return 0;
				}
			}
			Direction::nw => {
				if col + 1 >= self.grill[0].len() || col == 0 { return 0 }

				if player == self.grill[row + 1].chars().nth(col - 1).unwrap() {
					return 1 + self.count_aligned(player, dir, col - 1, row + 1);
				} else {
					return 0;
				}
			}
		}
	}
	fn hilight_winner(&mut self, mut col: usize, mut row: usize, dir: &Direction, player: char)
	{
		for i in 0..4
		{
			self.grill[row].remove(col);
			self.grill[row].insert(col, player.to_ascii_uppercase());
			match dir {
				Direction::s => {
					row -= 1;
				}
				Direction::n => {
					row += 1;
				}
				Direction::e => {
					col += 1;
				}
				Direction::w => {
					col -= 1;
				}
				Direction::se => {
					row -= 1;
					col += 1;
				}
				Direction::sw => {
					row -= 1;
					col -= 1;
				}
				Direction::ne => {
					col += 1;
					row += 1;
				}
				Direction::nw => {
					col -= 1;
					row += 1;
				}
			}
		}
	}
}

fn main() {
    let mut game = Game::new();
	let mut player: char = 'r';
	
	std::process::Command::new("clear").status().unwrap();
	loop {
		game.print_grill();
		println!("\nplayer {}: enter a column\n", match player {
			'r' => "red".red(),
			'b' => "blue".blue(),
			_ => "?".white(),
		});
		let mut input = String::new();
		std::io::stdin().read_line(&mut input);
		println!("");
		let mut input: usize = match input.trim().parse() {
			Ok(num) => num,
			Err(_) => {
				std::process::Command::new("clear").status().unwrap();
				println!("enter a valid input\n");
				continue ;
			}
		};
		if input == 0 || input > game.grill[0].len() {
			std::process::Command::new("clear").status().unwrap();
			println!("{} {} {}\n", "column".red(), input.to_string().red(), "don't exist".red());
			continue ;
		}
		input -= 1;
		if game.lowest_empty_row(input) == -1 {
			std::process::Command::new("clear").status().unwrap();
			println!("{}\n", "column full".red());
			continue ;
		}
		game.add_token(input, player);
		if game.is_finished() == true {
			println!("");
			return ;
		}
		if player == 'r' {
			player = 'b';
		} else {
			player = 'r';
		}
		std::process::Command::new("clear").status().unwrap();
	}
}
