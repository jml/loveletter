
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
        let draw_card = match card {
            Some(card) => card,
            None => break,
        };
        println!("Drew {}", draw_card);
        let result = loveletter::judge(&current, 0, draw_card, (draw_card, loveletter::Attack(1)));
        println!("Result: {}", result);
    }
    // While the game is not over
    //   Draw a card
    //   Give it to the player whose turn it is and ask them what their play is
    //   They discard that card
    //   Process it
    //   Advance to the next player
}
