use std::ops::{Index, Range};

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
	pub fn add_token(&self, column: usize)
	{

	}
	fn lowest_row(self, column: usize) -> usize
	{
		let	i: usize = 0;
	
		for row in self.grill.iter()
		{
			if row[column] == 0 {
				i
			}
		}
		i
	}
}

fn main() {
    println!("Hello, world!");
}
