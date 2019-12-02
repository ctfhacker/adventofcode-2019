// Function to solve Day 1 Stage 1
fn stage1(input: &str) {
    let solution = input.split("\r\n")
         // Ignore the empty string
         .filter(|x| x.len() > 0)
         // Parse all numbers as u64
         .map(|num| {
             num.parse::<u64>().unwrap()
         })
         // Perform the div by 3 then subtract by 2
         .map(|num| {
             let rounded = (num as f64 / 3.0).floor() as usize;
             rounded - 2
         })
         .sum::<usize>();

    print!("Stage 1: {}\n", solution);
}

// Auxillary function to calculate the fuel needed for a given mass for 
// Stage 2 of Day 1
fn get_fuel(start_mass: usize) -> usize {
    let mut result = 0;
    let mut mass = start_mass;
    loop {
        // Stage 2 mentioned that anything that divided by 9 is zero or less, 
        // the original mass is returned
        if mass < 9 {
           return result;
        }

        // Otherwise, calculate the fuel as usual
        mass = (mass as f64 / 3.0).floor() as usize - 2;
        result += mass;
    }
}

// Function to solve Day 1 Stage 2 
fn stage2(input: &str) {
    let solution = input.split("\r\n")
         // Ignore the empty string
         .filter(|x| x.len() > 0)
         // Parse all numbers as u64
         .map(|num| {
             num.parse::<usize>().unwrap()
         })
         // Perform the div by 3 then subtract by 2
         .map(|num| {
             get_fuel(num)
         })
         .sum::<usize>();

    print!("Stage 2: {}\n", solution);
}

fn main() {
    let input = include_str!("../input");
    stage1(&input);
    stage2(&input);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_fuel() {
        assert_eq!(get_fuel(14), 2);
        assert_eq!(get_fuel(1969), 966);
        assert_eq!(get_fuel(100756), 50346);
    }
}
