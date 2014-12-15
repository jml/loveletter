extern crate loveletter;

use loveletter::Card;
use loveletter::Play;
use loveletter::PlayError;


fn next_turn_err(g: &loveletter::Game, c: loveletter::Card, p: loveletter::Play) -> loveletter::PlayError {
    let result = g.handle_turn(|_, _| (c, p));
    match result {
        Err(e) => e,
        _ => panic!("Unexpectedly successful"),
    }
}

fn next_turn(g: &loveletter::Game, c: loveletter::Card, p: loveletter::Play) -> loveletter::Game {
    let (g, _) = g.handle_turn(|_, _| (c, p)).unwrap().unwrap();
    g
}


#[test]
fn test_invalid_target_attack() {
    let g = loveletter::Game::from_manual(
        &[Some(Card::General), Some(Card::Clown)],
        &[Card::Soldier, Card::Minister, Card::Princess, Card::Soldier], None).unwrap();
    assert_eq!(
        PlayError::InvalidPlayer(4),
        next_turn_err(&g, Card::General, Play::Attack(4)));
}

#[test]
fn test_invalid_target_guess() {
    let g = loveletter::Game::from_manual(
        &[Some(Card::General), Some(Card::Clown)],
        &[Card::Soldier, Card::Minister, Card::Princess, Card::Soldier], None).unwrap();
    assert_eq!(
        PlayError::InvalidPlayer(4),
        next_turn_err(&g, Card::Soldier, Play::Guess(4, Card::Wizard)));
}

#[test]
fn test_inactive_player_attack() {
    let g = loveletter::Game::from_manual(
        &[Some(Card::General), Some(Card::Clown), None],
        &[Card::Soldier, Card::Minister, Card::Princess, Card::Soldier], None).unwrap();
    assert_eq!(
        PlayError::InactivePlayer(2),
        next_turn_err(&g, Card::General, Play::Attack(2)));
}

#[test]
fn test_inactive_player_guess() {
    let g = loveletter::Game::from_manual(
        &[Some(Card::General), Some(Card::Clown), None],
        &[Card::Soldier, Card::Minister, Card::Princess, Card::Soldier], None).unwrap();
    assert_eq!(
        PlayError::InactivePlayer(2),
        next_turn_err(&g, Card::Soldier, Play::Guess(2, Card::Wizard)));
}

#[test]
fn test_bad_guess() {
    let g = loveletter::Game::from_manual(
        &[Some(Card::General), Some(Card::Clown), None],
        &[Card::Soldier, Card::Minister, Card::Princess, Card::Soldier], None).unwrap();
    assert_eq!(
        PlayError::BadGuess,
        next_turn_err(&g, Card::Soldier, Play::Guess(1, Card::Soldier)));
}

#[test]
fn test_princess_discard_loses() {
    let g = loveletter::Game::from_manual(
        &[Some(Card::Princess), Some(Card::Clown), None],
        &[Card::Soldier, Card::Minister, Card::Soldier, Card::Soldier], None).unwrap();
    let new_g = next_turn(&g, Card::Princess, Play::NoEffect);
    assert_eq!(vec![(1, Card::Clown)], new_g.winners());
}

#[test]
fn test_princess_forced_discard_loses() {
    let g = loveletter::Game::from_manual(
        &[Some(Card::Wizard), Some(Card::Princess), None],
        &[Card::Soldier, Card::Minister, Card::Soldier, Card::Soldier], None).unwrap();
    let new_g = next_turn(&g, Card::Wizard, Play::Attack(1));
    assert_eq!(vec![(0, Card::Soldier)], new_g.winners());
}

#[test]
fn test_princess_self_forced_discard_loses() {
    let g = loveletter::Game::from_manual(
        &[Some(Card::Wizard), Some(Card::Soldier), None],
        &[Card::Soldier, Card::Minister, Card::Soldier, Card::Princess], None).unwrap();
    let new_g = next_turn(&g, Card::Wizard, Play::Attack(0));
    assert_eq!(vec![(1, Card::Soldier)], new_g.winners());
}

#[test]
fn test_minister_eliminates_player() {
    let g = loveletter::Game::from_manual(
        &[Some(Card::General), Some(Card::Soldier)],
        &[Card::Wizard, Card::Minister], None).unwrap();
    let new_g = next_turn(&g, Card::General, Play::Attack(1));
    assert_eq!(vec![(1, Card::Soldier)], new_g.winners());
}

#[test]
fn test_minister_eliminates_player_2() {
    let g = loveletter::Game::from_manual(
        &[Some(Card::Minister), Some(Card::Soldier)],
        &[Card::Wizard, Card::Wizard], None).unwrap();
    let new_g = next_turn(&g, Card::General, Play::Attack(1));
    assert_eq!(vec![(1, Card::Soldier)], new_g.winners());
}


#[test]
fn test_priestess_immune_to_soldier_guess() {
    let g = loveletter::Game::from_manual(
        &[Some(Card::Priestess), Some(Card::Soldier)],
        &[Card::Clown, Card::Wizard], None).unwrap();
    let new_g = next_turn(&g, Card::Priestess, Play::NoEffect);
    let new_g = next_turn(&new_g, Card::Soldier, Play::Guess(0, Card::Wizard));
    // If player 0 is not protected by the priestess, then at this point,
    // player 1 will have won. If 0 *is* protected, then they win, because
    // they have the Wizard, which is higher than the Clown.
    assert_eq!(vec![(0, Card::Wizard)], new_g.winners());
}

#[test]
fn test_priestess_immune_to_clown() {
    let g = loveletter::Game::from_manual(
        &[Some(Card::Priestess), Some(Card::Clown)],
        &[Card::Clown, Card::Wizard], None).unwrap();
    let new_g = next_turn(&g, Card::Priestess, Play::NoEffect);
    let new_g = next_turn(&new_g, Card::Clown, Play::Attack(0));
    // If player 0 is not protected by the priestess, then at this point,
    // player 1 will have won. If 0 *is* protected, then they win, because
    // they have the Wizard, which is higher than the Clown.
    assert_eq!(vec![(0, Card::Wizard)], new_g.winners());
}

#[test]
fn test_priestess_immune_to_knight() {
    let g = loveletter::Game::from_manual(
        &[Some(Card::Priestess), Some(Card::Knight)],
        &[Card::Soldier, Card::Wizard, Card::Minister, Card::Wizard], None).unwrap();
    let new_g = next_turn(&g, Card::Priestess, Play::NoEffect);
    let new_g = next_turn(&new_g, Card::Knight, Play::Attack(0));
    let new_g = next_turn(&new_g, Card::Wizard, Play::Attack(1));
    // If player 0 is not protected by the priestess, then at this point,
    // player 1 will have won. If 0 *is* protected, then they win, because
    // they have the Wizard, which is higher than the Clown.
    assert_eq!(vec![(0, Card::Wizard)], new_g.winners());
}

#[test]
fn test_priestess_immune_to_wizard() {
    let g = loveletter::Game::from_manual(
        &[Some(Card::Priestess), Some(Card::Wizard)],
        &[Card::Clown, Card::Wizard], None).unwrap();
    let new_g = next_turn(&g, Card::Priestess, Play::NoEffect);
    let new_g = next_turn(&new_g, Card::Wizard, Play::Attack(0));
    // If player 0 is not protected by the priestess, then at this point,
    // player 1 will have won. If 0 *is* protected, then they win, because
    // they have the Wizard, which is higher than the Clown.
    assert_eq!(vec![(0, Card::Wizard)], new_g.winners());
}

#[test]
fn test_priestess_immune_to_general() {
    let g = loveletter::Game::from_manual(
        &[Some(Card::Priestess), Some(Card::General)],
        &[Card::Soldier, Card::Princess], None).unwrap();
    let new_g = next_turn(&g, Card::Priestess, Play::NoEffect);
    let new_g = next_turn(&new_g, Card::General, Play::Attack(0));
    // If player 0 is not protected by the priestess, then at this point,
    // player 1 will have won. If 0 *is* protected, then they win, because
    // they have the Wizard, which is higher than the Clown.
    assert_eq!(vec![(0, Card::Princess)], new_g.winners());
}

#[test]
fn test_priestess_immunity_expires() {
    let g = loveletter::Game::from_manual(
        &[Some(Card::Priestess), Some(Card::Soldier)],
        &[Card::Soldier, Card::Wizard, Card::Clown, Card::Clown], None).unwrap();
    let new_g = next_turn(&g, Card::Priestess, Play::NoEffect);
    let new_g = next_turn(&new_g, Card::Soldier, Play::Guess(0, Card::Wizard));
    let new_g = next_turn(&new_g, Card::Clown, Play::Attack(1));
    let new_g = next_turn(&new_g, Card::Soldier, Play::Guess(0, Card::Wizard));
    // If player 0 is not protected by the priestess, then at this point,
    // player 1 will have won. If 0 *is* protected, then they win, because
    // they have the Wizard, which is higher than the Clown.
    assert_eq!(vec![(1, Card::Clown)], new_g.winners());
}
