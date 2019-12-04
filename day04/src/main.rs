use std::collections::HashSet;
/// Check if the number passwords is valid for Stage 1
fn is_password_1(input: &str) -> bool {
    let mut prev = 0;
    let mut repeated = false;
    for i in input.chars() {
        // Digit's hex value is also increasing like the digit itself, so this 
        // conversion is still valid without having to parse the exact digit
        let curr = i as u8;

        // If we are starting, just set the first character as prev and continue
        if prev == 0 {
            prev = curr; 
            continue;
        }

        // Quick return false if the string is not in increasing order of digits
        if prev > curr { 
            return false; 
        }

        if prev == curr { repeated = true; }

        // Set the prev element to the current for the next iteration
        prev = curr;
    }

    repeated
}

/// Check if the number password is valid for Stage 2
fn is_password_2(input: &str) -> bool {
    let mut prev = 0;
    let mut repeated = HashSet::new();
    let mut curr_count = 1;
    for i in input.chars() {
        // Digit's hex value is also increasing like the digit itself, so this 
        // conversion is still valid without having to parse the exact digit
        let curr = i as u8;

        // If we are starting, just set the first character as prev and continue
        if prev == 0 {
            prev = curr; 
            continue;
        }

        // Quick return false if the string is not in increasing order of digits
        if prev > curr { 
            return false; 
        }

        if prev != curr {
            // If the count of the previous digit is more than a double (2) it is 
            // invalid, so remove it from the repeated HashSet.
            if curr_count > 2 {
                repeated.remove(&prev);
            }
            curr_count = 1;
        } else {
            // Current element is the same as previous, increase the current seen count
            curr_count += 1;
            repeated.insert(curr);
        }

        // Set the prev element to the current for the next iteration
        prev = curr;
    }

    // Need to check this one more time just in case the last characters were an 
    // odd contiguous amount
    if curr_count > 2 {
        // If the count of the previous digit is more than a double (2) it is 
        // invalid, so remove it from the repeated HashSet.
        repeated.remove(&prev);
    }

    // Only return true if we have seen at least one repeated digit
    repeated.len() > 0
}

fn main() {
    let passwords = (246540..787419)
        .filter(|num| is_password_1(&format!("{}", num)))
        .count();
    print!("Stage 1: {}\n", passwords);

    let passwords = (246540..787419)
        .filter(|num| is_password_2(&format!("{}", num)))
        .count();
    print!("Stage 2: {}\n", passwords);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_num_check_1() {
        assert_eq!(is_password_1(&"111111"), true);
        assert_eq!(is_password_1(&"223450"), false);
        assert_eq!(is_password_1(&"123789"), false);
    }

    #[test]
    fn test_num_check_2() {
        assert_eq!(is_password_2(&"112233"), true);
        assert_eq!(is_password_2(&"123444"), false);
        assert_eq!(is_password_2(&"134445"), false);
        assert_eq!(is_password_2(&"344456"), false);
        assert_eq!(is_password_2(&"444567"), false);
        assert_eq!(is_password_2(&"134456"), true);
        assert_eq!(is_password_2(&"111122"), true);
        assert_eq!(is_password_2(&"111123"), false);
        assert_eq!(is_password_2(&"111111"), false);
        assert_eq!(is_password_2(&"111115"), false);
        assert_eq!(is_password_2(&"223450"), false);
        assert_eq!(is_password_2(&"223455"), true);
    }
}

