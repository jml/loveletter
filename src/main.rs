
extern crate loveletter;

#[cfg(not(test))]
fn main() {
    println!("Love Letter");
    let deck = loveletter::Deck::new();
    println!("{}", deck);
    println!("{}", deck.shuffled());
    println!("{}", deck.shuffled());
    println!("{}", deck);
}
