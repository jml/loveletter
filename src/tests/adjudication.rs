use deck::Card::{Soldier, Clown, Knight, Wizard, General};

use action::Play::{Attack, Guess};
use action::PlayError::{BadActionForCard, BadGuess, SelfTarget};
use action::Action::{
    SwapHands, ForceDiscard, ForceReveal, EliminateWeaker, EliminateOnGuess};
use action::play_to_action;


#[test]
fn test_general_swap() {
    let result = play_to_action(0, General, Attack(3)).unwrap();
    assert_eq!(result, SwapHands(0, 3));
}

#[test]
fn test_self_target_attack() {
    let result = play_to_action(0, General, Attack(0));
    assert_eq!(SelfTarget(0, General), result.unwrap_err());
}

#[test]
fn test_self_target_guess() {
    let result = play_to_action(0, Soldier, Guess(0, Wizard));
    assert_eq!(SelfTarget(0, Soldier), result.unwrap_err());
}

#[test]
fn test_self_target_wizard() {
    let result = play_to_action(0, Wizard, Attack(0));
    assert_eq!(ForceDiscard(0), result.unwrap());
}

#[test]
fn test_knight() {
    let result = play_to_action(0, Knight, Attack(3));
    assert_eq!(EliminateWeaker(0, 3), result.unwrap());
}

#[test]
fn test_wizard() {
    let result = play_to_action(0, Wizard, Attack(1));
    assert_eq!(ForceDiscard(1), result.unwrap());
}

#[test]
fn test_clown() {
    let result = play_to_action(0, Clown, Attack(1));
    assert_eq!(ForceReveal(0, 1), result.unwrap());
}

#[test]
fn test_non_attack() {
    let result = play_to_action(1, Soldier, Attack(0));
    assert_eq!(BadActionForCard(Attack(0), Soldier), result.unwrap_err());
}

#[test]
fn test_soldier() {
    let result = play_to_action(0, Soldier, Guess(1, Wizard));
    assert_eq!(EliminateOnGuess(1, Wizard), result.unwrap());
}

#[test]
fn test_guess_soldier() {
    let result = play_to_action(0, Soldier, Guess(1, Soldier));
    assert_eq!(BadGuess, result.unwrap_err());
}
