use std::fmt;
use std::io;

/// Repeatedly prompt the user until they give us something that parses.
pub fn repeated_prompt<T, E: fmt::Show>(prompt: &str, parser: |&str| -> Result<T, E>) -> T {
    loop {
        print!("{}", prompt);
        let input = io::stdin()
            .read_line()
            .ok()
            .expect("Failed to read line");
        match parser(input.as_slice()) {
            Ok(value) => return value,
            Err(err) => println!("{}", err),
        }
    }
}


pub fn read_int_in_range(x: &str, upper: uint) -> Result<uint, String> {
    match from_str(x.trim()) {
        None => Err(format!(
            "Please enter a number between 1 and {}", upper)),
        Some(x) =>
            if 1 <= x && x <= upper {
                Ok(x - 1)
            } else {
                Err(format!("Please enter a number between 1 and {}", upper))
            }
    }
}


pub fn choose_from_list<'a, T: fmt::Show>(prompt: &str, items: &'a [T]) -> &'a T {
    let mut prompt_vec = vec![prompt.to_string()];
    prompt_vec.push("\n".to_string());
    for (i, x) in items.iter().enumerate() {
        prompt_vec.push(format!("  {}. {}\n", i + 1, x));
    }
    prompt_vec.push(">>> ".to_string());
    let i = repeated_prompt(prompt_vec.concat().as_slice(), |x| read_int_in_range(x, items.len()));
    &items[i]
}
