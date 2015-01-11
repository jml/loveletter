use deck::Card::{Soldier, Clown, Knight, Wizard, General};

use action::Play::{Attack, Guess};
use action::PlayError::{BadActionForCard, BadGuess, SelfTarget};
use action::Action::{
    SwapHands, ForceDiscard, ForceReveal, EliminateWeaker, EliminateOnGuess};
use action::play_to_action;
use player_id::{PlayerId, player_id_generator};


fn make_players() -> (PlayerId, PlayerId) {
    let players: Vec<PlayerId> = player_id_generator().take(2).collect();
    (players[0], players[1])
}


#[test]
fn test_general_swap() {
    let (player1, player2) = make_players();
    let result = play_to_action(player1, General, Attack(player2)).unwrap();
    assert_eq!(result, SwapHands(player1, player2));
}

#[test]
fn test_self_target_attack() {
    let (player1, _) = make_players();
    let result = play_to_action(player1, General, Attack(player1));
    assert_eq!(SelfTarget(player1, General), result.unwrap_err());
}

#[test]
fn test_self_target_guess() {
    let (player1, _) = make_players();
    let result = play_to_action(player1, Soldier, Guess(player1, Wizard));
    assert_eq!(SelfTarget(player1, Soldier), result.unwrap_err());
}

#[test]
fn test_self_target_wizard() {
    let (player1, _) = make_players();
    let result = play_to_action(player1, Wizard, Attack(player1));
    assert_eq!(ForceDiscard(player1), result.unwrap());
}

#[test]
fn test_knight() {
    let (player1, player2) = make_players();
    let result = play_to_action(player1, Knight, Attack(player2));
    assert_eq!(EliminateWeaker(player1, player2), result.unwrap());
}

#[test]
fn test_wizard() {
    let (player1, player2) = make_players();
    let result = play_to_action(player1, Wizard, Attack(player2));
    assert_eq!(ForceDiscard(player2), result.unwrap());
}

#[test]
fn test_clown() {
    let (player1, player2) = make_players();
    let result = play_to_action(player1, Clown, Attack(player2));
    assert_eq!(ForceReveal(player1, player2), result.unwrap());
}

#[test]
fn test_non_attack() {
    let (player1, player2) = make_players();
    let result = play_to_action(player2, Soldier, Attack(player1));
    assert_eq!(BadActionForCard(Attack(player1), Soldier), result.unwrap_err());
}

#[test]
fn test_soldier() {
    let (player1, player2) = make_players();
    let result = play_to_action(player1, Soldier, Guess(player2, Wizard));
    assert_eq!(EliminateOnGuess(player2, Wizard), result.unwrap());
}

#[test]
fn test_guess_soldier() {
    let (player1, player2) = make_players();
    let result = play_to_action(player1, Soldier, Guess(player2, Soldier));
    assert_eq!(BadGuess, result.unwrap_err());
}
