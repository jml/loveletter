extern crate loveletter;

use loveletter::Card;
use loveletter::Play;
use loveletter::PlayError;


fn make_round(hands: &[Option<Card>], deck: &[Card], current_player: Option<uint>) -> (loveletter::Round, Vec<loveletter::PlayerId>) {
    let game = loveletter::game::new_game(hands.len()).unwrap();
    let players = game.players();
    assert_eq!(players.len(), hands.len());
    let hands: Vec<(loveletter::PlayerId, Option<Card>)> = players
        .iter()
        .zip(hands.iter())
        .map(|(&a, &b)| (a, b))
        .collect();
    (loveletter::Round::from_manual(
        hands.as_slice(), deck, current_player.map(|i| players[i])).unwrap(), players)
}

fn next_turn_err(g: &loveletter::Round, c: loveletter::Card, p: loveletter::Play) -> loveletter::PlayError {
    let result = g.handle_turn(|_, _| (c, p), |_, _| ());
    match result {
        Err(e) => e,
        _ => panic!("Unexpectedly successful"),
    }
}

fn next_turn(g: &loveletter::Round, c: loveletter::Card, p: loveletter::Play) -> loveletter::Round {
    let (g, _) = g.handle_turn(|_, _| (c, p), |_, _| ()).unwrap().unwrap();
    g
}

#[test]
fn test_inactive_player_attack() {
    let (g, players) = make_round(
        &[Some(Card::General), Some(Card::Clown), None],
        &[Card::Soldier, Card::Minister, Card::Princess, Card::Soldier], None);
    assert_eq!(
        PlayError::InactivePlayer(players[2]),
        next_turn_err(&g, Card::General, Play::Attack(players[2])));
}

#[test]
fn test_inactive_player_guess() {
    let (g, players) = make_round(
        &[Some(Card::General), Some(Card::Clown), None],
        &[Card::Soldier, Card::Minister, Card::Princess, Card::Soldier], None);
    assert_eq!(
        PlayError::InactivePlayer(players[2]),
        next_turn_err(&g, Card::Soldier, Play::Guess(players[2], Card::Wizard)));
}

#[test]
fn test_bad_guess() {
    let (g, players) = make_round(
        &[Some(Card::General), Some(Card::Clown), None],
        &[Card::Soldier, Card::Minister, Card::Princess, Card::Soldier], None);
    assert_eq!(
        PlayError::BadGuess,
        next_turn_err(&g, Card::Soldier, Play::Guess(players[1], Card::Soldier)));
}

#[test]
fn test_princess_discard_loses() {
    let (g, players) = make_round(
        &[Some(Card::Princess), Some(Card::Clown), None],
        &[Card::Soldier, Card::Minister, Card::Soldier, Card::Soldier], None);
    let new_g = next_turn(&g, Card::Princess, Play::NoEffect);
    assert_eq!(vec![(players[1], Card::Clown)], new_g.winners());
}

#[test]
fn test_princess_forced_discard_loses() {
    let (g, players) = make_round(
        &[Some(Card::Wizard), Some(Card::Princess), None],
        &[Card::Soldier, Card::Minister, Card::Soldier, Card::Soldier], None);
    let new_g = next_turn(&g, Card::Wizard, Play::Attack(players[1]));
    assert_eq!(vec![(players[0], Card::Soldier)], new_g.winners());
}

#[test]
fn test_princess_self_forced_discard_loses() {
    let (g, players) = make_round(
        &[Some(Card::Wizard), Some(Card::Soldier), None],
        &[Card::Soldier, Card::Minister, Card::Soldier, Card::Princess], None);
    let new_g = next_turn(&g, Card::Wizard, Play::Attack(players[0]));
    assert_eq!(vec![(players[1], Card::Soldier)], new_g.winners());
}

#[test]
fn test_minister_eliminates_player() {
    let (g, players) = make_round(
        &[Some(Card::General), Some(Card::Soldier)],
        &[Card::Wizard, Card::Minister], None);
    let new_g = next_turn(&g, Card::General, Play::Attack(players[1]));
    assert_eq!(vec![(players[1], Card::Soldier)], new_g.winners());
}

#[test]
fn test_minister_eliminates_player_2() {
    let (g, players) = make_round(
        &[Some(Card::Minister), Some(Card::Soldier)],
        &[Card::Wizard, Card::Wizard], None);
    let new_g = next_turn(&g, Card::General, Play::Attack(players[1]));
    assert_eq!(vec![(players[1], Card::Soldier)], new_g.winners());
}


#[test]
fn test_priestess_immune_to_soldier_guess() {
    let (g, players) = make_round(
        &[Some(Card::Priestess), Some(Card::Soldier)],
        &[Card::Clown, Card::Wizard], None);
    let new_g = next_turn(&g, Card::Priestess, Play::NoEffect);
    let new_g = next_turn(&new_g, Card::Soldier, Play::Guess(players[0], Card::Wizard));
    // If player 0 is not protected by the priestess, then at this point,
    // player 1 will have won. If 0 *is* protected, then they win, because
    // they have the Wizard, which is higher than the Clown.
    assert_eq!(vec![(players[0], Card::Wizard)], new_g.winners());
}

#[test]
fn test_priestess_immune_to_clown() {
    let (g, players) = make_round(
        &[Some(Card::Priestess), Some(Card::Clown)],
        &[Card::Clown, Card::Wizard], None);
    let new_g = next_turn(&g, Card::Priestess, Play::NoEffect);
    let new_g = next_turn(&new_g, Card::Clown, Play::Attack(players[0]));
    // If player 0 is not protected by the priestess, then at this point,
    // player 1 will have won. If 0 *is* protected, then they win, because
    // they have the Wizard, which is higher than the Clown.
    assert_eq!(vec![(players[0], Card::Wizard)], new_g.winners());
}

#[test]
fn test_priestess_immune_to_knight() {
    let (g, players) = make_round(
        &[Some(Card::Priestess), Some(Card::Knight)],
        &[Card::Soldier, Card::Wizard, Card::Minister, Card::Wizard], None);
    let new_g = next_turn(&g, Card::Priestess, Play::NoEffect);
    let new_g = next_turn(&new_g, Card::Knight, Play::Attack(players[0]));
    let new_g = next_turn(&new_g, Card::Wizard, Play::Attack(players[1]));
    // If player 0 is not protected by the priestess, then at this point,
    // player 1 will have won. If 0 *is* protected, then they win, because
    // they have the Wizard, which is higher than the Clown.
    assert_eq!(vec![(players[0], Card::Wizard)], new_g.winners());
}

#[test]
fn test_priestess_immune_to_wizard() {
    let (g, players) = make_round(
        &[Some(Card::Priestess), Some(Card::Wizard)],
        &[Card::Clown, Card::Wizard], None);
    let new_g = next_turn(&g, Card::Priestess, Play::NoEffect);
    let new_g = next_turn(&new_g, Card::Wizard, Play::Attack(players[0]));
    // If player 0 is not protected by the priestess, then at this point,
    // player 1 will have won. If 0 *is* protected, then they win, because
    // they have the Wizard, which is higher than the Clown.
    assert_eq!(vec![(players[0], Card::Wizard)], new_g.winners());
}

#[test]
fn test_priestess_immune_to_general() {
    let (g, players) = make_round(
        &[Some(Card::Priestess), Some(Card::General)],
        &[Card::Soldier, Card::Princess], None);
    let new_g = next_turn(&g, Card::Priestess, Play::NoEffect);
    let new_g = next_turn(&new_g, Card::General, Play::Attack(players[0]));
    // If player 0 is not protected by the priestess, then at this point,
    // player 1 will have won. If 0 *is* protected, then they win, because
    // they have the Wizard, which is higher than the Clown.
    assert_eq!(vec![(players[0], Card::Princess)], new_g.winners());
}

#[test]
fn test_priestess_immunity_expires() {
    let (g, players) = make_round(
        &[Some(Card::Priestess), Some(Card::Soldier)],
        &[Card::Soldier, Card::Wizard, Card::Clown, Card::Clown], None);
    let new_g = next_turn(&g, Card::Priestess, Play::NoEffect);
    let new_g = next_turn(&new_g, Card::Soldier, Play::Guess(players[0], Card::Wizard));
    let new_g = next_turn(&new_g, Card::Clown, Play::Attack(players[1]));
    let new_g = next_turn(&new_g, Card::Soldier, Play::Guess(players[0], Card::Wizard));
    // If player 0 is not protected by the priestess, then at this point,
    // player 1 will have won. If 0 *is* protected, then they win, because
    // they have the Wizard, which is higher than the Clown.
    assert_eq!(vec![(players[1], Card::Clown)], new_g.winners());
}
