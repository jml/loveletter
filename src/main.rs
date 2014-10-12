
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


fn choose(_game: &loveletter::Game, turn: &loveletter::Turn) -> (loveletter::Card, loveletter::Play) {
    let chosen = repeated_prompt(
        format!(
            "Player {}: pick a card:\n  1. {}\n  2. {}",
            turn.player + 1, turn.hand, turn.draw).as_slice(),
        |x| match x.trim() {
            "1" => Ok(turn.hand),
            "2" => Ok(turn.draw),
            _ => Err("1 or 2"),
        });
    (chosen, loveletter::Attack((turn.player + 1) % 2))
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
    loop {
        let result = current_game.handle_turn(choose);
        current_game = match result {
            None => break,
            Some(game) => game,
        };
        println!("");
    }
    // XXX: Announce the winner
}
