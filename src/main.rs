extern crate loveletter;


#[cfg(not(test))]
fn choose_card(turn: &loveletter::Turn) -> loveletter::Card {
    let list = [turn.hand, turn.draw];
    let choice = loveletter::prompt::choose_from_list("Pick a card", list);
    *choice
}


#[cfg(not(test))]
fn choose_target(game: &loveletter::Game) -> uint {
    let num_players = game.num_players();
    loveletter::prompt::repeated_prompt(
        format!(
            "Who are you playing it on? (1-{})\n>>> ",
            num_players).as_slice(),
        |x| loveletter::prompt::read_int_in_range(x, num_players))
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
