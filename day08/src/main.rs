fn stage1(input: &str) {
    let width = 25;
    let height = 6;
    let mut best_offset = 0;
    let mut best_count = 99999999;
    for i in (0..input.len()).step_by(width * height) {
        let end = std::cmp::min(input.len(), i + (width*height));
        let curr_chunk = &input[i..end];
        let curr_count = curr_chunk.chars().filter(|x| *x == '0').count();
        if curr_count < best_count && curr_count > 0 {
            best_offset = i;
            best_count = curr_count;
        }
    }

    print!("best: {}\n", best_count);
    let best_chunk = &input[best_offset..best_offset + (width*height)];
    let ones = best_chunk.chars().filter(|x| *x == '1').count();
    let twos = best_chunk.chars().filter(|x| *x == '2').count();
    print!("Stage 1: {}\n", ones * twos);
}

fn stage2(input: &str) {
    let width = 25;
    let height = 6;
    let image_size = width * height;
    let mut image = ['2'; 25 * 6];
    for i in (0..input.len()).step_by(image_size) {
        let end = std::cmp::min(input.len(), i + (image_size));
        let curr_chunk = &input[i..end];
        for (i, ch) in curr_chunk.chars().enumerate() {
            if image[i] == '2' && ch != '2' {
                image[i] = ch;
            }
        }
    }

    print!("Stage 2\n");
    for (i, &ch) in image.iter().enumerate() {
        if i > 0 && i % width == 0 {
            print!("\n");
        }
        if ch == '0' {
            print!("{}", '.');
        }
        if ch == '1' {
            print!("{}", '#');
        }
    }
    print!("\n");

}

fn main() {
    let input = include_str!("../input");
    stage1(&input);
    stage2(&input);
}



