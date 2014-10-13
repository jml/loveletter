extern crate loveletter;


use loveletter::{Soldier, Clown, Knight, Priestess, Wizard, General, Minister, Princess};


#[test]
fn test_integration() {
    let g = loveletter::Game::from_manual([], [], None);
    assert_eq!(g, Err(loveletter::InvalidPlayers(0)));
}


#[test]
fn test_bad_soldier_guess() {
    let cards = [
        Soldier, Wizard, Soldier, Soldier,
        Princess, Clown, Soldier, Knight,
        Knight, Priestess, General, Minister,
        Priestess, Wizard, Clown, Soldier,
        ];
    let deck = loveletter::deck::Deck::from_slice(cards).unwrap();
    let game = loveletter::Game::from_deck(2, deck).unwrap();
    let new_game = game.handle_turn(|_, _| (Soldier, loveletter::Guess(1, Knight)));
}
