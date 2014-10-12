
extern crate loveletter;

use std::io;


fn repeated_prompt<T, E: std::fmt::Show>(prompt: &str, parser: |&str| -> Result<T, E>) -> T {
    loop {
        println!("{}", prompt);
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
    // While the game is not over
    //   Draw a card
    //   Give it to the player whose turn it is and ask them what their play is
    //   They discard that card
    //   Process it
    //   Advance to the next player
    let mut current_game = game;
    let current_player = 0u;
    loop {
        let (new_game, card) = current_game.draw();
        current_game = new_game;
        let current_card = match current_game.get_hand(current_player) {
            Ok(c) => c,
            Err(e) => {
                println!("player {} has no card!: {}", current_player, e);
                return;
            }
        };
        let draw_card = match card {
            Some(card) => card,
            None => break,
        };
        repeated_prompt(
            format!("Pick a card: {}, {}", current_card, draw_card).as_slice(),
            |x| match x.trim() {
                "1" => Ok(current_card),
                "2" => Ok(draw_card),
                _ => Err("1 or 2"),
            });
        // XXX: Allow to pick which of draw_card or current_card
        let result = loveletter::judge(
            &current_game, 0, draw_card, (draw_card, loveletter::Attack(1)));
        // XXX: Apply the action
        // XXX: Discard the played card
        // XXX: Advance to the next player
        println!("Result: {}", result);
        println!("");
    }
    // XXX: Announce the winner
}
