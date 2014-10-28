extern crate loveletter;

// XXX: [rust]: After marking all of the other code below as not part of test,
// then I get unused-import warning for this. What's the correct way to have a
// main module that doesn't emit warnings?
use std::io;

/// Repeatedly prompt the user until they give us something that parses.
#[cfg(not(test))]
fn repeated_prompt<T, E: std::fmt::Show>(prompt: &str, parser: |&str| -> Result<T, E>) -> T {
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


#[cfg(not(test))]
fn read_int_in_range(x: &str, upper: uint) -> Result<uint, String> {
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


#[cfg(not(test))]
fn choose_from_list<'a, T: std::fmt::Show>(prompt: &str, items: &'a [T]) -> &'a T {
    let mut prompt_vec = vec![prompt.to_string()];
    prompt_vec.push("\n".to_string());
    for (i, x) in items.iter().enumerate() {
        prompt_vec.push(format!("  {}. {}\n", i + 1, x));
    }
    prompt_vec.push(">>> ".to_string());
    let i = repeated_prompt(prompt_vec.concat().as_slice(), |x| read_int_in_range(x, items.len()));
    &items[i]
}


#[cfg(not(test))]
fn choose_card(turn: &loveletter::Turn) -> loveletter::Card {
    let list = [turn.hand, turn.draw];
    let choice = choose_from_list("Pick a card", list);
    *choice
}


#[cfg(not(test))]
fn choose_target(game: &loveletter::Game) -> uint {
    let num_players = game.num_players();
    repeated_prompt(
        format!(
            "Who are you playing it on? (1-{})\n>>> ",
            num_players).as_slice(),
        |x| read_int_in_range(x, num_players))
}


/// Allow the player to choose a card to play.
#[cfg(not(test))]
fn choose(_game: &loveletter::Game, turn: &loveletter::Turn) -> (loveletter::Card, loveletter::Play) {
    println!("Player {}", turn.player + 1);
    println!("---------");
    let chosen = choose_card(turn);
    let action = match chosen {
        loveletter::Priestess | loveletter::Minister | loveletter::Princess => loveletter::NoEffect,
        _ => {
            let other = choose_target(_game);
            match chosen {
                // TODO: Allow specifying guess.
                loveletter::Soldier => loveletter::Guess(other, loveletter::Wizard),
                _ => loveletter::Attack(other),
            }
        },
    };
    println!("// action: player {} => {}: {}", turn.player + 1, chosen, action);
    (chosen, action)
}


#[cfg(not(test))]
fn announce_winner(winners: Vec<(uint, loveletter::Card)>) {
    // TODO: Probably want to report on all survivors.
    // TODO: Probably want to say *why* the game is over: no more players or
    // no more cards.
    // TODO: Message for last player standing should be different from highest
    // card.
    println!("GAME OVER");
    match winners.len() {
        0 => println!("Something went wrong. No winners at all. Is the game over yet?"),
        1 => {
            let (i, card) = winners[0];
            println!("Player {} wins, holding {}", i + 1, card);
        },
        n => {
            println!("Game tied between {} players.", n);
            for &(i, card) in winners.iter() {
                println!("  Player {} holds a {}", i + 1, card);
            }
        }
    }
}


#[cfg(not(test))]
fn main() {
    println!("Love Letter");
    println!("===========");
    println!("");

    let game = match loveletter::Game::new(2) {
        Some(g) => g,
        None => {
            println!("Invalid number of players: 2");
            // XXX: [rust] How do I exit with a non-zero code?
            return;
        }
    };
    println!("// game = {}\n", game);
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
            Err(e) => { println!("Invalid move: {}\n", e); continue }
        };
        println!("// game = {}\n", current_game);
    }
    announce_winner(current_game.winners());
}
