use deck::{
    Soldier, Clown, Knight, Priestess, Wizard, General, Minister, Princess};

use super::super::Game;
use super::super::{judge, play_to_action};
use super::super::{SwapHands, ForceDiscard, ForceReveal, EliminateWeaker,
                   EliminateOnGuess};
use super::super::{Attack, Guess, NoEffect};
use super::super::{InvalidPlayer, CardNotFound, InactivePlayer, SelfTarget,
                   BadActionForCard, BadGuess};

use super::make_arbitrary_game;

#[test]
fn test_judge_invalid_player() {
    let g = make_arbitrary_game();
    let err = judge(&g, 5, Soldier, (Priestess, NoEffect)).unwrap_err();
    assert_eq!(InvalidPlayer(5), err);
}

#[test]
fn test_judge_invalid_target_attack() {
    let g = Game::from_manual(
        [Some(General), Some(Clown), Some(Knight), Some(Priestess)],
        [Soldier, Minister, Princess, Soldier], None).unwrap();
    let arbitrary_card = Wizard;
    let result = judge(&g, 0, arbitrary_card, (General, Attack(4)));
    assert_eq!(InvalidPlayer(4), result.unwrap_err());
}

#[test]
fn test_judge_invalid_target_guess() {
    let g = Game::from_manual(
        [Some(Soldier), Some(Clown), Some(Knight), Some(Priestess)],
        [Soldier, Minister, Princess, Soldier], None).unwrap();
    let arbitrary_card = Wizard;
    let result = judge(&g, 0, arbitrary_card, (Soldier, Guess(4, Minister)));
    assert_eq!(InvalidPlayer(4), result.unwrap_err());
}

#[test]
fn test_judge_inactive_player_attack() {
    let g = Game::from_manual(
        [Some(General), Some(Clown), None, Some(Priestess)],
        [Soldier, Minister, Princess, Soldier], None).unwrap();
    let arbitrary_card = Wizard;
    let result = judge(&g, 0, arbitrary_card, (General, Attack(2)));
    assert_eq!(InactivePlayer(2), result.unwrap_err());
}

#[test]
fn test_judge_inactive_player_guess() {
    let g = Game::from_manual(
        [Some(General), Some(Clown), None, Some(Priestess)],
        [Soldier, Minister, Princess, Soldier], None).unwrap();
    let arbitrary_card = Wizard;
    let result = judge(&g, 0, arbitrary_card, (General, Guess(2, Minister)));
    assert_eq!(InactivePlayer(2), result.unwrap_err());
}

#[test]
fn test_judge_play_without_card() {
    let g = Game::from_manual(
        [Some(Soldier), Some(Clown), Some(Knight), Some(Priestess)],
        [Soldier, Minister, Princess, Soldier], None).unwrap();
    // Player 0 has a Wizard and a Soldier, but is trying to play a
    // General.
    let result = judge(&g, 0, Wizard, (General, Attack(2)));
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
