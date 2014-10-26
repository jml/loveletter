extern crate loveletter;

use std::io;

/// Repeatedly prompt the user until they give us something that parses.
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


/// Allow the player to choose a card to play.
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
    // TODO: Allow specifying other.
    let other = (turn.player + 1) % 2;
    let action = match chosen {
        loveletter::Priestess | loveletter::Minister | loveletter::Princess => loveletter::NoEffect,
        // TODO: Allow specifying guess.
        loveletter::Soldier => loveletter::Guess(other, loveletter::Wizard),
        _ => loveletter::Attack(other),
    };
    println!("Player {} => {}: {}", turn.player + 1, chosen, action);
    (chosen, action)
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
            Ok(None) => break,
            Ok(Some(game)) => game,
            Err(e) => { println!("Error: {}", e); return }
        };
        println!("{}", current_game);
        println!("");
    }
    // TODO: Announce the winner
}
