fn stage1(input: &str) {
    let solution = input.split("\r\n")
         .filter(|x| x.len() > 0)
         .map(|num| {
             num.parse::<u64>().unwrap()
         })
         .map(|num| {
             let rounded = (num as f64 / 3.0).round() as usize;
             print!("{} -> {}\n", num, rounded - 2);
             rounded - 2
         })
         .sum::<usize>();

    print!("Stage 1: {}\n", solution);
}

fn main() {
    let input = include_str!("../input");
    stage1(&input);
}
