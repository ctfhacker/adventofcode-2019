//! Advent of Code 2019 Day 3 solution
//!
//! The wire path is documented in a HashMap. Each step of the wire is inserted into
//! the HashMap along with which wire is currently moving and the current step along the
//! path. When a wire marks a location, the HashMap is checked to see if another wire
//! has already inserted into the hashmap (aka intersected this location). If so,
//! the distance to center and signal distance (other wire's steps and the current wires 
//! steps) are calculated and stored in the Grid if it is the smallest currently seen.
use std::collections::HashMap;

/// Basic Grid struct used to follow the wires for Day 3
struct Grid {
    /// (PositionX, PositionY): (WireId, Steps)
    buffer: HashMap<(isize, isize), (u8, usize)>,
    position_x: isize,
    position_y: isize,
    shortest_intersection: usize,
    shortest_signal_delay: usize,
}

impl Grid {
    /// Initialize the Grid with the center position in the center of the grid
    ///
    /// The grid keeps track of the shortest intersections as we come across them.
    pub fn new() -> Grid {
        Grid {
            buffer: HashMap::new(),
            position_x: 0,
            position_y: 0,
            shortest_intersection: usize::max_value(),
            shortest_signal_delay: usize::max_value(),
        }
    }

    /// Reset the cursor position to the center of the grid
    pub fn reset(&mut self) {
        self.position_x = 0;
        self.position_y = 0;
    }

    /// Calculate the manhattan distance of the given x,y for the current grid
    pub fn distance(&self, x: isize, y: isize) -> usize {
        (x.abs() + y.abs()) as usize
    }

    /// Increase the current cursor position by one. Whenever the current cursor's 
    /// position number is larger than one, we have come across an intersection.
    /// Save that intersection distance if it is the shortest we have seen so far
    pub fn mark(&mut self, wire_id: u8, step: usize) {
        let curr_position = (self.position_x, self.position_y);
        if self.buffer.contains_key(&curr_position) && self.buffer.get(&curr_position).unwrap().0 == wire_id {
            // We only keep track of the first time a wire hits a given location
            return;
        }

        match self.buffer.insert((self.position_x, self.position_y), (wire_id, step)) {
            Some((_old_wire_id, old_steps)) => {
                let curr_distance = self.distance(self.position_x, self.position_y);
                if curr_distance < self.shortest_intersection {
                    self.shortest_intersection = curr_distance;
                }

                let signal = old_steps + step;
                if signal < self.shortest_signal_delay {
                    self.shortest_signal_delay = signal;
                }
            }
            None => {
            }
        }
    }

    /// Move the cursor left a given amount passing along the current wire and current
    /// step of the current wire 
    pub fn left(&mut self, amount: usize, wire_id: u8, step: usize) {
        for i in 1..=amount {
            self.position_x = self.position_x.checked_sub(1).expect("Moved left off board");
            self.mark(wire_id, step+i);
        }
    }

    /// Move the cursor right a given amount passing along the current wire and current
    /// step of the current wire 
    pub fn right(&mut self, amount: usize, wire_id: u8, step: usize) {
        for i in 1..=amount {
            self.position_x = self.position_x.checked_add(1).expect("Moved right off board");
            self.mark(wire_id, step+i);
        }
    }

    /// Move the cursor up a given amount passing along the current wire and current
    /// step of the current wire 
    pub fn up(&mut self, amount: usize, wire_id: u8, step: usize) {
        for i in 1..=amount {
            self.position_y = self.position_y.checked_sub(1).expect("Moved up off board");
            self.mark(wire_id, step+i);
        }
    }

    /// Move the cursor down a given amount passing along the current wire and current
    /// step of the current wire 
    pub fn down(&mut self, amount: usize, wire_id: u8, step: usize) {
        for i in 1..=amount {
            self.position_y = self.position_y.checked_add(1).expect("Moved up off board");
            self.mark(wire_id, step+i);
        }
    }
    
    /// Given the input format from the problem and the current wire, mark the positions
    /// that this wire crosses on the grid.
    pub fn mark_wire(&mut self, input: &str, wire_id: u8) {
        let mut step = 0;
        for movement in input.split(",") {
            let amount = movement[1..].parse::<usize>().unwrap();
            let direction = movement.chars().nth(0).unwrap();
            match direction {
                'D' => self.down(amount, wire_id, step),
                'U' => self.up(amount, wire_id, step),
                'R' => self.right(amount, wire_id, step),
                'L' => self.left(amount, wire_id, step),
                _ => unreachable!()
            }
            step += amount;
        }
    }
}

fn solve(input: &str) {
    let mut grid = Grid::new();
    for (i, line) in input.lines().enumerate() {
        grid.mark_wire(line, i as u8);
        grid.reset();
    }

    print!("Stage1: {}\n", grid.shortest_intersection);
    print!("Stage2: {}\n", grid.shortest_signal_delay);
}

fn main() {
    let input = include_str!("../input");
    solve(input);
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        let mut grid = Grid::new();
        grid.mark_wire("R8,U5,L5,D3", 1);
        grid.reset();
        grid.mark_wire("U7,R6,D4,L4", 2);
        assert_eq!(grid.shortest_intersection, 6);
    }

    #[test]
    fn test_example_2() {
        let mut grid = Grid::new();
        grid.mark_wire("U62,R66,U55,R34,D71,R55,D58,R83", 1);
        grid.reset();
        grid.mark_wire("R75,D30,R83,U83,L12,D49,R71,U7,L72", 2);
        assert_eq!(grid.shortest_intersection, 159);
    }

    #[test]
    fn test_example_3() {
        let mut grid = Grid::new();
        grid.mark_wire("R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51", 1);
        grid.reset();
        grid.mark_wire("U98,R91,D20,R16,D67,R40,U7,R15,U6,R7", 2);
        assert_eq!(grid.shortest_intersection, 135);
    }

    #[test]
    fn test_example_signal_1() {
        let mut grid = Grid::new();
        grid.mark_wire("R75,D30,R83,U83,L12,D49,R71,U7,L72", 1);
        grid.reset();
        grid.mark_wire("U62,R66,U55,R34,D71,R55,D58,R83", 2);
        assert_eq!(grid.shortest_signal_delay, 610);
    }

    #[test]
    fn test_example_signal_2() {
        let mut grid = Grid::new();
        grid.mark_wire("R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51", 1);
        grid.reset();
        grid.mark_wire("U98,R91,D20,R16,D67,R40,U7,R15,U6,R7", 2);
        assert_eq!(grid.shortest_signal_delay, 410);
    }
}
