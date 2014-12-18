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
fn choose_target(game: &loveletter::Round) -> PlayerId {
    let num_players = game.num_players();
    loveletter::prompt::repeated_prompt(
        format!(
            "Who are you playing it on? (1-{})\n>>> ",
            num_players).as_slice(),
        |x| loveletter::prompt::read_int_in_range(x, num_players))
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
fn choose(_game: &loveletter::Round, turn: &loveletter::Turn) -> (Card, loveletter::Play) {
    println!("Player {}", turn.player + 1);
    println!("---------");
    let chosen = choose_card(turn);
    let action = match chosen {
        Card::Priestess | Card::Minister | Card::Princess => loveletter::Play::NoEffect,
        _ => {
            let other = choose_target(_game);
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


fn report_outcome(game: &loveletter::Round, outcome: loveletter::TurnOutcome) -> String {
    match outcome {
        loveletter::TurnOutcome::BustedOut(player) => {
            let discards = game.get_discards(player).ok().expect("Busted player did not exist");
            let a = discards[discards.len() - 1];
            let b = discards[discards.len() - 2];
            format!("Player {} busted out with {} and {}!", player + 1, a, b)
        },
        loveletter::TurnOutcome::Played(player, card, play, events) => {
            let prelude = format!("Player {} played {}", player + 1, card);
            let follow_up = match play {
                loveletter::Play::NoEffect => ".".to_string(),
                loveletter::Play::Attack(i) => format!(" on player {}.", i + 1),
                loveletter::Play::Guess(i, guess) =>
                    format!(" on player {}, guessing {}.", i + 1, guess),
            };
            let mut event_str = String::new();
            for event in events.iter() {
                event_str = event_str + (match *event {
                    Event::NoChange => "Nothing happened. ".to_string(),
                    Event::Protected(_) => "Now protected until their next turn. ".to_string(),
                    Event::SwappedHands(_, b) => format!("Swapped hands with player {}. ", b + 1),
                    Event::PlayerEliminated(p) => format!("Player {} eliminated. ", p + 1),
                    Event::ForcedReveal(a, b) => format!("Player {} showed their card to player {}. ", b + 1, a + 1),
                    Event::ForcedDiscard(p) => {
                        // XXX: Worth saying here whether the player was
                        // allowed to draw another card.
                        let last_discard: &Card = game
                            .get_discards(p)
                            .ok().expect("Targeted player did not exist")
                            .last().expect("Player forced to discard does not have any discards");
                        format!("Player {} forced to discard {}. ", p + 1, last_discard)
                    },
                })
            }
            format!("{}{} {}", prelude, follow_up, event_str)
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
            println!("Player {} wins, holding {}", i + 1, card);
        },
        n => {
            println!("Round tied between {} players.", n);
            for &(i, card) in winners.iter() {
                println!("  Player {} holds a {}", i + 1, card);
            }
        }
    }
    println!("");
}


fn announce_current_scores(scores: &[uint]) {
    println!("Scores");
    println!("------");
    for (player_id, &score) in scores.iter().enumerate() {
        println!("Player {}: {}", player_id + 1, score);
    }
    println!("");
}

fn announce_game_winners(scores: &[uint]) {
    println!("GAME OVER");
    println!("");
    announce_current_scores(scores);
}


fn handle_reveal(player: PlayerId, card: Card) -> () {
    println!("SECRET: Player {} has a {}", player + 1, card);
}


#[cfg(not(test))]
fn main() {
    println!("Love Letter");
    println!("===========");
    println!("");

    let num_players = 2u;

    let game = match loveletter::game::new_game(num_players) {
        Ok(g) => g,
        Err(loveletter::Error::InvalidPlayers(..)) => {
            println!("Invalid number of players: {}", num_players);
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
                println!("  P{}: {}", i + 1, discards);
            }
            println!("");
            let result = current_round.handle_turn(choose, handle_reveal);
            let (new_round, outcome) = match result {
                Ok(None) => break,
                Ok(Some(result)) => result,
                Err(e) => { println!("Invalid move: {}\n", e); continue }
            };

            io::println(report_outcome(&new_round, outcome).as_slice());
            println!("");
            current_round = new_round;
        }
        let winners = current_round.winners();
        announce_winner(&winners);
        let winner_ids: Vec<PlayerId> = winners.iter().map(|&(i, _)| i).collect();
        current_game = current_game.players_won(winner_ids.as_slice());
        announce_current_scores(current_game.scores());
        println!("");
    }
    announce_game_winners(current_game.scores());
}
