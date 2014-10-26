
extern crate loveletter;

use loveletter::Game;


#[test]
fn test_too_few_players() {
    assert_eq!(None, Game::new(1));
}

#[test]
fn test_too_many_players() {
    assert_eq!(None, Game::new(5));
}
