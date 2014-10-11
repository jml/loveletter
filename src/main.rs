
extern crate loveletter;

#[cfg(not(test))]
fn main() {
    println!("Love Letter");
    let game = loveletter::Game::new();
    println!("{}", game);
}
