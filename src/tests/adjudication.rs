use deck::{
    Soldier, Clown, Knight, Priestess, Wizard, General, Minister, Princess};

use super::super::Game;
use super::super::{judge, play_to_action};
use super::super::{SwapHands, ForceDiscard, ForceReveal, EliminateWeaker,
                   EliminateOnGuess};
use super::super::{Attack, Guess};
use super::super::{CardNotFound, SelfTarget, BadActionForCard, BadGuess};


#[test]
fn test_judge_play_without_card() {
    let g = Game::from_manual(
        [Some(Soldier), Some(Clown), Some(Knight), Some(Priestess)],
        [Soldier, Minister, Princess, Soldier], None).unwrap();
    // Player 0 has a Wizard and a Soldier, but is trying to play a
    // General.
    let result = judge(&g, 0, Soldier, Wizard, (General, Attack(2)));
    assert_eq!(
        CardNotFound(General, (Soldier, Wizard)), result.unwrap_err());
}

#[test]
fn test_general_swap() {
    let result = play_to_action(0, General, Wizard, Attack(3)).unwrap();
    assert_eq!(result, SwapHands(0, 3, Wizard));
}

#[test]
fn test_self_target_attack() {
    let result = play_to_action(0, General, Wizard, Attack(0));
    assert_eq!(SelfTarget(0, General), result.unwrap_err());
}

#[test]
fn test_self_target_guess() {
    let result = play_to_action(0, Soldier, Wizard, Guess(0, Wizard));
    assert_eq!(SelfTarget(0, Soldier), result.unwrap_err());
}

#[test]
fn test_self_target_wizard() {
    let result = play_to_action(0, Wizard, General, Attack(0));
    assert_eq!(ForceDiscard(0), result.unwrap());
}

#[test]
fn test_knight() {
    let result = play_to_action(0, Knight, Knight, Attack(3));
    assert_eq!(EliminateWeaker(0, 3), result.unwrap());
}

#[test]
fn test_wizard() {
    let result = play_to_action(0, Wizard, Soldier, Attack(1));
    assert_eq!(ForceDiscard(1), result.unwrap());
}

#[test]
fn test_clown() {
    let result = play_to_action(0, Clown, Wizard, Attack(1));
    assert_eq!(ForceReveal(0, 1), result.unwrap());
}

#[test]
fn test_non_attack() {
    let result = play_to_action(1, Soldier, Knight, Attack(0));
    assert_eq!(BadActionForCard(Attack(0), Soldier), result.unwrap_err());
}

#[test]
fn test_soldier() {
    let result = play_to_action(0, Soldier, Wizard, Guess(1, Wizard));
    assert_eq!(EliminateOnGuess(1, Wizard), result.unwrap());
}

#[test]
fn test_guess_soldier() {
    let result = play_to_action(0, Soldier, Wizard, Guess(1, Soldier));
    assert_eq!(BadGuess, result.unwrap_err());
}
