extern crate loveletter;

use loveletter::Card;
use loveletter::GameError;


#[test]
fn test_too_few_players() {
    assert_eq!(None, loveletter::Game::new(1));
}

#[test]
fn test_too_many_players() {
    assert_eq!(None, loveletter::Game::new(5));
}


#[test]
fn test_new_game() {
    let g = loveletter::Game::new(4).unwrap();
    assert_eq!(g.num_players(), 4);
}

#[test]
fn test_invalid_manual_game() {
    let hands = [
        Some(Card::Soldier),
        Some(Card::Clown),
        Some(Card::Soldier),
        Some(Card::Princess),
        ];
    let stack = [
        Card::Soldier,
        Card::Princess,
        Card::Minister,
        ];
    let result = loveletter::Game::from_manual(&hands, &stack, None).unwrap_err();
    assert_eq!(result, GameError::BadDeck);
}

#[test]
fn test_manual_game_bad_players() {
    assert_eq!(Err(GameError::InvalidPlayers(0)), loveletter::Game::from_manual(&[], &[], None));
}
