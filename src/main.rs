
extern crate loveletter;

#[cfg(not(test))]
fn main() {
    println!("Love Letter");
    let game = match loveletter::Game::new(2) {
        Some(g) => g,
        None => {
            println!("Invalid number of players: 2");
            // XXX: [rust] How do I exit with a non-zero code?
            return;
        }
    };
    println!("{}", game);
    let mut current = game;
    loop {
        let (new_game, card) = current.draw();
        current = new_game;
        match card {
            Some(card) => println!("Drew {}", card),
            None => break,
        }
    }
    // While the game is not over
    //   Draw a card
    //   Give it to the player whose turn it is and ask them what their play is
    //   They discard that card
    //   Process it
    //   Advance to the next player
}
