use std::collections::HashSet;

struct Board {
    /// Total width of the board
    width: usize,

    /// Total height of the board
    _height: usize,

    /// Underlying buffer of the board
    buffer: Vec<char>,

    /// List of all asteroids in the board
    asteroids: Vec<(usize, usize)>
}

impl Board {
    /// Given a width and height, create a new board of width * height
    pub fn from_input(input: &str) -> Board {
        let grid = input.split("\n").filter(|x| x.len() > 0).collect::<Vec<_>>();
        let width = grid[0].len();
        let height = grid.len();

        let mut asteroids = Vec::new();

        // Allocate the print buffer with '.' as blank spaces
        let mut buffer = Vec::new();
        for (y, line) in grid.iter().enumerate() {
            for (x, c) in line.chars().enumerate() {
                buffer.push(c);
                if c == '#' { 
                    asteroids.push((x, y));        
                }
            }
        }

        Board {
            width,
            _height: height,
            buffer,
            asteroids
        }
    }

    pub fn get(&self, x: usize, y: usize) -> char{
        self.buffer[y * self.width + x]
    }

    pub fn mark(&mut self, x: usize, y: usize, t: char) {
        self.buffer[y * self.width + x] = t;
    }

    pub fn print(&self) {
        // Print the resulting board
        for (_i, line) in self.buffer.as_slice().chunks(self.width).enumerate() {
            for c in line {
                print!("{}", c);
            }
            print!("\n");
        }
    }

    pub fn best_station(&self) -> usize {
        let mut most_asteroids = 0;

        for curr_asteroid in &self.asteroids {
            let mut seen_asteroids = HashSet::new();
            for asteroid in &self.asteroids {
                if curr_asteroid == asteroid { continue; }
                let rise = curr_asteroid.1 as isize - asteroid.1 as isize;
                let run  = curr_asteroid.0 as isize - asteroid.0 as isize;
                let direction = if run > 0 { 
                    String::from("+") 
                } else if run < 0 { 
                    String::from("-") 
                } else {
                    String::from("")
                };

                let slope = if run == 0 {
                    if rise > 0 { String::from("L") } else { String::from("R") }
                } else if rise == 0 {
                    if run > 0 { String::from("U") } else { String::from("D") }
                } else {
                    format!("{}{:.4}", direction, rise as f64 / run as f64)
                };

                seen_asteroids.insert(slope);
            }

            print!("{:?} [{}] {:?}\n", curr_asteroid, seen_asteroids.len(), seen_asteroids);
            if most_asteroids < seen_asteroids.len() {
                most_asteroids = seen_asteroids.len()
            }
        }

        most_asteroids
    }
}
fn main() {
    let input = include_str!("../input");
    let board = Board::from_input(input);
    print!("Best: {}\n", board.best_station());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_0() {
        let input = include_str!("../example0");
        let board = Board::from_input(input);
        assert_eq!(board.best_station(), 9);
    }
    #[test]
    fn test_example_1() {
        let input = include_str!("../example1");
        let board = Board::from_input(input);
        assert_eq!(board.best_station(), 33);
    }
    #[test]
    fn test_example_2() {
        let input = include_str!("../example2");
        let board = Board::from_input(input);
        assert_eq!(board.best_station(), 35);
    }
    #[test]
    fn test_example_3() {
        let input = include_str!("../example3");
        let board = Board::from_input(input);
        assert_eq!(board.best_station(), 41);
    }
    #[test]
    fn test_example_4() {
        let input = include_str!("../example4");
        let board = Board::from_input(input);
        assert_eq!(board.best_station(), 210);
    }
}
