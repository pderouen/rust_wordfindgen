extern crate rand;
use std::fs;
use std::error::Error;
use std::fmt;
use rand::Rng;
use std::convert::TryFrom;
use std::io::prelude::*;
use rand::seq::SliceRandom;

// Config - configuration based on command line arguments
//
// The first arg should be the name of a text file with the words to place in the puzzle
// If a second argument is included the puzzle will be more difficult,
//    by also placing words right to left (backwards)
//
pub struct Config {
    pub wordsfile: String,
    pub size: usize,
    pub maxtries: usize,
    pub hard: bool,
}

impl Config {
    pub fn new(mut args: std::env::Args) -> Result<Config, &'static str> {
        // move past program invocation
        args.next();
        
        let wordsfile = match args.next(){
            Some(arg) => arg,
            None => return Err("no input words file provided"),
        };
        
        let hard = match args.next() {
            Some(_arg) => true,
            None => false,
        };
        
        Ok(Config { wordsfile, size: 20, maxtries: 10000, hard })
    }
}

// PuzzleError - Just need a struct that implements Error
//
#[derive(Debug)]
struct PuzzleError {
    err: String,
}

impl PuzzleError {
    pub fn new(err: String) -> PuzzleError {
        PuzzleError { err }
    }
}

impl fmt::Display for PuzzleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.err)
    }
}

impl Error for PuzzleError {
}

// Direction - The 8 possible directions in which to place a word in the puzzle
//
// Clone and Copy are derived so that move isn't the default action when using assignment
//
#[derive(Debug,Clone,Copy)]
pub enum Direction {
    Right,
    UpRight,
    Up,
    UpLeft,
    Left,
    DownLeft,
    Down,
    DownRight,
}

impl Direction{
    // The x and y increment values associated with each direction
    pub fn incrementors(&self) -> (i8, i8) {
        match self {
            Direction::Right => (1, 0),
            Direction::UpRight => (1, -1),
            Direction::Up => (0, -1),
            Direction::UpLeft => (-1, -1),
            Direction::Left => (-1, 0),
            Direction::DownLeft => (-1, 1),
            Direction::Down => (0, 1),
            Direction::DownRight => (1, 1),
        }
    }
}

// PuzzleGrid - The main struct for holding and generating the puzzle
//
struct PuzzleGrid {
    grid: Vec<Vec<String>>,
    size: i8,
    maxtries: usize,
    dir_choices: Vec<Direction>,
    entries: Vec<String>,
}

impl PuzzleGrid {
    pub fn new(size: i8, maxtries: usize, hard: bool) -> PuzzleGrid {
        let s = usize::try_from(size).unwrap();
        let grid: Vec<Vec<String>> = vec![vec![String::from(" "); s]; s];
        let dir_choices = if hard {
            vec![Direction::Right, Direction::UpRight, Direction::Up, Direction::UpLeft, Direction::Left, Direction::DownLeft, Direction::Down, Direction::DownRight]
        } else {
            vec![Direction::Right, Direction::UpRight, Direction::Up, Direction::Down, Direction::DownRight]
        };
        PuzzleGrid { grid, size, maxtries, dir_choices, entries: Vec::new() }
    }
    
    // place - attempts to randomly place the given word into the puzzle
    pub fn place(&mut self, word: &str) -> Result<(), Box<dyn Error>> {
        let mut x = 0;
        let mut y = 0;
        let mut dir = Direction::Right;
        let mut placed = false;
        let mut sanitized_word = String::from(word);
        sanitized_word.make_ascii_uppercase();
        
        // randomly select x, y, and direction until maxtries reached, or valid placement was found
        for _ in 1..self.maxtries {
            x = rand::thread_rng().gen_range(0, self.size);
            y = rand::thread_rng().gen_range(0, self.size);
            if let Some(d) = self.dir_choices.choose(&mut rand::thread_rng()) { dir = *d };
            placed = self.placement_valid(&sanitized_word, &x, &y, &dir);
            if placed { break; }
        }
        
        if placed {
            self.entries.push(sanitized_word.to_string());
        
            // place the word in the puzzle here
            // probably could have directly returned to iterators over the indeces
            let (x_indeces, y_indeces) = self.get_indeces(&sanitized_word, &x, &y, &dir);
            let mut x_iter = x_indeces.iter();
            let mut y_iter = y_indeces.iter();
            
            for char in sanitized_word.chars() {
                let xi = x_iter.next().unwrap();
                let yi = y_iter.next().unwrap();
                self.grid[*yi][*xi] = char.to_string();
            }
            
            Ok(())
        } else {
            Err(Box::new(PuzzleError::new(format!("{} could not be placed in the puzzle", word))))
        }        
    }
    
    // get_indeces - returns the Vec[x][y] for placement into the puzzle of each character in the word
    //               There is likely a more elegant way to do this
    pub fn get_indeces(&self, word: &str, x: &i8, y: &i8, dir: &Direction) -> (Vec<usize>, Vec<usize>) {
        let (x_inc, y_inc) = dir.incrementors();
        let mut x_indeces: Vec<usize> = Vec::with_capacity(word.len());
        let mut y_indeces: Vec<usize> = Vec::with_capacity(word.len());
        
        let mut xi = *x;
        let mut yi = *y;
        for _ in word.chars() {
            x_indeces.push(usize::try_from(xi).unwrap());
            y_indeces.push(usize::try_from(yi).unwrap());
            xi += x_inc;
            yi += y_inc;
        }
        
        (x_indeces, y_indeces)
    }
    
    // placement_valid - returns true if the word fits at the given coordinates and direction with no collisions
    //                   Lots of code duplication with get_indeces, likely a better way to do this.
    fn placement_valid(&self, word: &str, x: &i8, y: &i8, dir: &Direction) -> bool {
        let (x_inc, y_inc) = dir.incrementors();
        let mut xi = *x;
        let mut yi = *y;
        for _ in word.chars() {
            xi += x_inc;
            yi += y_inc;
        }
        
        if xi >= 0 && xi <= self.size && yi >= 0 && yi <= self.size {
            // the word fits, now make sure it doesn't collide
            let (x_indeces, y_indeces) = self.get_indeces(&word, &x, &y, &dir);
            let mut x_iter = x_indeces.iter();
            let mut y_iter = y_indeces.iter();
            let space = String::from(" ");
            
            for char in word.chars() {
                let xi = x_iter.next().unwrap();
                let yi = y_iter.next().unwrap();
                
                // as long as the grid contains " " or a matching character there is no collision
                if !(self.grid[*yi][*xi] == char.to_string() || self.grid[*yi][*xi] == space) {
                    return false
                }
            }
            true
        } else {
            false
        }
    }
    
    // output - write the puzzle grid and words to a file in csv
    pub fn output(&self, file_name: &str) -> Result<(), Box<dyn Error>> {
        let mut file = fs::File::create(file_name)?;
        
        // puzzle grid
        for v in self.grid.iter() {
            file.write(b",,,")?;
            file.write(v.join(",").as_bytes())?;
            file.write(b"\n")?;
        }
        
        // search words
        file.write(b"\n\n\n")?;
        let mut i = 0;
        for entry in self.entries.iter() {
            file.write(b",,,")?;
            file.write(entry.as_bytes())?;
            i += 1;
            if i == 2 {
                file.write(b"\n")?;
                i = 0;
            }
        }
        
        Ok(())
    }
    
    // fill_in - locate all blank grid entries and fill with a random letter
    pub fn fill_in(&mut self) {
        let chars = String::from("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
        let space = String::from(" ");
        for v in self.grid.iter_mut() {
            for i in v.iter_mut() {
                if *i == space {
                    let idx = rand::thread_rng().gen_range(0, chars.len());
                    *i = chars.chars().nth(idx).unwrap().to_string();
                }
            }
        }
    }
}

// run - the main runner. Creates the PuzzleGrid, places the words and outputs results
pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let words = fs::read_to_string(config.wordsfile)?;
    
    // validate that the words are all shorter than the grid size
    for word in words.lines() {
        if word.len() > config.size {
            return Err(Box::new(PuzzleError::new(format!("{} is too long to fit in a {} x {} puzzle", word, config.size, config.size))));
        }
    }
    
    let mut puzzle = PuzzleGrid::new(i8::try_from(config.size).unwrap(), config.maxtries, config.hard);
    
    // place all of the words in the puzzle
    for word in words.lines() {
        puzzle.place(&word)?;
    }
    
    // output the answer key
    puzzle.output("answer_key.csv")?;
    
    // fill empty grid spaces with random letters
    puzzle.fill_in();
    
    // output the finished puzzle
    puzzle.output("puzzle.csv")?;
    
    Ok(())
}

// not really exhaustively tested... just needed to check a few pieces along the way

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dir_incrementors_work() {
        let mut dir = Direction::Right;
        assert_eq!(dir.incrementors(), (1, 0));
        dir = Direction::UpRight;
        assert_eq!(dir.incrementors(), (1, -1));
    }
    
    #[test]
    fn indeces(){
        let puzzle = PuzzleGrid::new(20, 10000, true);
        let x: i8 = 10;
        let y: i8 = 10;
        let dir = Direction::DownRight;
        let (x_indeces, y_indeces) = puzzle.get_indeces("Thanks", &x, &y, &dir);
        assert_eq!(x_indeces, [10, 11, 12, 13, 14, 15]);
        assert_eq!(y_indeces, [10, 11, 12, 13, 14, 15]);
    }
}