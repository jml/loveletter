extern crate loveletter;

use std::io;
use std::os;
use loveletter::{Card, Event, PlayerId};


#[cfg(not(test))]
fn choose_card(turn: &loveletter::Turn) -> loveletter::Card {
    let list = [turn.hand, turn.draw];
    let choice = loveletter::prompt::choose_from_list("Pick a card", &list);
    *choice
}


#[cfg(not(test))]
fn choose_target(game: &loveletter::Game) -> PlayerId {
    let players = game.players();
    *loveletter::prompt::choose_from_list(
        format!(
            "Who are you playing it on? (1-{:?})\n>>> ",
            players.len()).as_slice(),
        players.as_slice())
}


#[cfg(not(test))]
fn choose_guess() -> Card {
    *loveletter::prompt::choose_from_list(
        "Which card do you guess?",
        &[Card::Clown,
          Card::Knight,
          Card::Priestess,
          Card::Wizard,
          Card::General,
          Card::Minister,
          Card::Princess])
}


/// Allow the player to choose a card to play.
#[cfg(not(test))]
fn choose(game: &loveletter::Game, turn: &loveletter::Turn) -> (Card, loveletter::Play) {
    println!("{:?}", turn.player);
    println!("---------");
    let chosen = choose_card(turn);
    let action = match chosen {
        Card::Priestess | Card::Minister | Card::Princess => loveletter::Play::NoEffect,
        _ => {
            let other = choose_target(game);
            match chosen {
                Card::Soldier => {
                    let guess = choose_guess();
                    loveletter::Play::Guess(other, guess)
                },
                _ => loveletter::Play::Attack(other),
            }
        },
    };
    (chosen, action)
}


fn format_event(event: &loveletter::Event) -> String {
    match *event {
        Event::NoChange => "Nothing happened. ".to_string(),
        Event::Protected(_) => "Now protected until their next turn. ".to_string(),
        Event::SwappedHands(_, b) => format!("Swapped hands with {:?}. ", b),
        Event::PlayerEliminated(p) => format!("{:?} eliminated. ", p),
        Event::ForcedReveal(a, b) => format!("{:?} showed their card to {:?}. ", b, a),
        Event::ForcedDiscard(p, card) => {
            // XXX: Worth saying here whether the player was allowed to draw
            // another card?
            format!("{:?} forced to discard {:?}. ", p, card)
        }
    }
}


fn report_outcome(outcome: loveletter::TurnOutcome) -> String {
    match outcome {
        loveletter::TurnOutcome::BustedOut(player, a, b) => {
            format!("{:?} busted out with {:?} and {:?}!", player, a, b)
        },
        loveletter::TurnOutcome::Played(player, card, play, events) => {
            let prelude = format!("{:?} played {:?}", player, card);
            let follow_up = match play {
                loveletter::Play::NoEffect => ".".to_string(),
                loveletter::Play::Attack(i) => format!(" on {:?}.", i),
                loveletter::Play::Guess(i, guess) =>
                    format!(" on {:?}, guessing {:?}.", i, guess),
            };
            let mut event_str = String::new();
            for event in events.iter() {
                event_str = event_str + format_event(event).as_slice();
            }
            format!("{:?}{:?} {:?}", prelude, follow_up, event_str)
        },
    }
}


#[cfg(not(test))]
fn announce_winner(winners: &Vec<(PlayerId, Card)>) {
    // TODO: Probably want to report on all survivors.
    // TODO: Probably want to say *why* the game is over: no more players or
    // no more cards.
    // TODO: Message for last player standing should be different from highest
    // card.
    print!("ROUND OVER: ");
    match winners.len() {
        0 => println!("Something went wrong. No winners at all. Is the game over yet?"),
        1 => {
            let (i, card) = winners[0];
            println!("{:?} wins, holding {:?}", i, card);
        },
        n => {
            println!("Round tied between {:?} players.", n);
            for &(i, card) in winners.iter() {
                println!("  {:?} holds a {:?}", i, card);
            }
        }
    }
    println!("");
}


fn announce_current_scores(scores: &[uint]) {
    println!("Scores");
    println!("------");
    for (player_id, &score) in scores.iter().enumerate() {
        println!("Player {:?}: {:?}", player_id + 1, score);
    }
    println!("");
}

fn announce_game_winners(scores: &[uint]) {
    println!("GAME OVER");
    println!("");
    announce_current_scores(scores);
}


fn handle_reveal(player: PlayerId, card: Card) -> () {
    println!("SECRET: {:?} has a {:?}", player, card);
}


#[cfg(not(test))]
fn main() {
    println!("Love Letter");
    println!("===========");
    println!("");

    let num_players = 2u;

    let game = match loveletter::game::new_game(num_players) {
        Some(g) => g,
        None => {
            println!("Invalid number of players: {:?}", num_players);
            os::set_exit_status(2);
            return;
        }
    };

    // While the game is not over
    //   Draw a card
    //   Give it to the player whose turn it is and ask them what their play is
    //   They discard that card
    //   Process it
    //   Advance to the next player

    let mut current_game = game;
    loop {
        let round = match current_game.next_round() {
            Some(r) => r,
            None => break,
        };
        let mut current_round = round;
        println!("NEW ROUND");
        println!("");
        loop {
            println!("All Discards");
            println!("------------");
            for (i, discards) in current_round.all_discards().iter().enumerate() {
                println!("  P{:?}: {:?}", i + 1, discards);
            }
            println!("");
            // XXX: Maybe Round should have a reference to Game so this capture isn't need
            let result = current_round.handle_turn(
                |_, turn| choose(&current_game, turn), handle_reveal);
            let (new_round, outcome) = match result {
                Ok(None) => break,
                Ok(Some(result)) => result,
                Err(e) => { println!("Invalid move: {:?}\n", e); continue }
            };

            io::println(report_outcome(outcome).as_slice());
            println!("");
            current_round = new_round;
        }
        let winners = current_round.winners();
        announce_winner(&winners);
        let winner_ids: Vec<PlayerId> = winners.iter().map(|&(i, _)| i).collect();
        current_game = current_game.players_won(winner_ids.as_slice());
        let scores = current_game.scores();
        announce_current_scores(scores.as_slice());
        println!("");
    }
    let scores = current_game.scores();
    announce_game_winners(scores.as_slice());
}
