
extern crate loveletter;

#[cfg(not(test))]
fn main() {
    println!("Love Letter");
    let game = loveletter::Game::new(2);
    println!("{}", game);
    // While the game is not over
    //   Draw a card
    //   Give it to the player whose turn it is and ask them what their play is
    //   They discard that card
    //   Process it
    //   Advance to the next player
}
