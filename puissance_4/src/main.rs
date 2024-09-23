use colored::Colorize;

pub struct Game {
	grill: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
enum Direction {
	South ,
	Est ,
	SWest ,
	SEst ,
}

impl Game {
	pub fn new() -> Game
	{
		let grill = vec![
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

		println!("{}", "  1   2   3   4   5   6   7  \n".green().blink());
		// println!("{}", "  ⇩   ⇩   ⇩   ⇩   ⇩   ⇩   ⇩  \n\n".green().blink());
		while i != 0 {
			print!("{} ", "|".blue());
			for c in self.grill[i - 1].chars() {
				if c == '0' {
					print!("  {} ", "|".blue());
				} else {
					print!("{} {} ", match c {
						'r' | 'R' => c.to_string().red(),
						'j' | 'J' => c.to_string().yellow(),
						_ => "?".white(),
					}, "|".blue());
				}
			}
			println!("");
			i -= 1;
		}
		println!("{}", "¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯\n".blue());
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
				let directions = vec![
					Direction::South,
					Direction::Est,
					Direction::SWest,
					Direction::SEst,
					];
				if c == '0' {
					continue ;
				} else {
					for dir in directions {
						// println!("aligned == {}", self.count_aligned(c, dir, col, row));
						if self.count_aligned(c, dir.clone(), col, row) >= 3
						{
							self.hilight_winner(col, row, dir.clone(), c);
							// std::process::Command::new("clear").status().unwrap();
							self.print_grill();
							println!("{}",  match c {
								'r' | 'R' => "Red wins !!!".red(),
								'j' | 'J' => "Yellow wins !!!".yellow(),
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
	fn count_aligned(&self, player: char, dir: Direction, col: usize, row: usize) -> usize
	{
		match dir {
			Direction::South => {
				if row == 0 { return 0 }
				if player == self.grill[row - 1].chars().nth(col).unwrap() {
					println!("{}: South += 1", player);
					return 1 + self.count_aligned(player, dir, col, row - 1);
				} else {
					return 0;
				}
			}
			Direction::Est => {
				if col + 1 >= self.grill[0].len() {
					return 0;
				}else if player == self.grill[row].chars().nth(col + 1).unwrap() {
					println!("{}: Est += 1", player);
					return 1 + self.count_aligned(player, dir, col + 1, row);
				} else {
					return 0;
				}
			}
			Direction::SEst => {
				if col + 1 >= self.grill[0].len() || row == 0 {
					return 0
				}
				if player == self.grill[row - 1].chars().nth(col + 1).unwrap() {
					println!("{}: South Est += 1", player);
					return 1 + self.count_aligned(player, dir, col + 1, row - 1);
				} else {
					return 0;
				}
			}
			Direction::SWest => {
				if row == 0 || col == 0 { return 0 }

				if player == self.grill[row - 1].chars().nth(col - 1).unwrap() {
					println!("{}: South West += 1", player);
					return 1 + self.count_aligned(player, dir, col - 1, row - 1);
				} else {
					return 0;
				}
			}
		}
	}
	fn hilight_winner(&mut self, mut col: usize, mut row: usize, dir: Direction, player: char)
	{
		for i in 0..4
		{
			self.grill[row].remove(col);
			self.grill[row].insert(col, player.to_ascii_uppercase());
			if i == 3 {
				return ;
			} match dir {
				Direction::South => {
					row -= 1;
				}
				Direction::Est => {
					col += 1;
				}
				Direction::SEst => {
					row -= 1;
					col += 1;
				}
				Direction::SWest => {
					row -= 1;
					col -= 1;
				}
			}
		}
	}
}

fn main() {
    let mut game = Game::new();
	let mut player: char = 'r';
	
	// std::process::Command::new("clear").status().unwrap();
	loop {
		game.print_grill();
		println!("\n{}\n", match player {
			'r' => "player Red: enter a column".red(),
			'j' => "player Yellow: enter a column".yellow(),
			_ => "?".white(),
		});
		let mut input = String::new();
		let _ = std::io::stdin().read_line(&mut input);
		println!("");
		let mut input: usize = match input.trim().parse() {
			Ok(num) => num,
			Err(_) => {
				// std::process::Command::new("clear").status().unwrap();
				println!("enter a valid column number\n");
				continue ;
			}
		};
		if input == 0 || input > game.grill[0].len() {
			// std::process::Command::new("clear").status().unwrap();
			println!("{} {} {}\n", "column".red(), input.to_string().red(), "don't exist".red());
			continue ;
		}
		input -= 1;
		if game.lowest_empty_row(input) == -1 {
			// std::process::Command::new("clear").status().unwrap();
			println!("{}\n", "column full".red());
			continue ;
		}
		game.add_token(input, player);
		if game.is_finished() == true {
			println!("");
			return ;
		}
		if player == 'r' {
			player = 'j';
		} else {
			player = 'r';
		}
		// std::process::Command::new("clear").status().unwrap();
	}
}
