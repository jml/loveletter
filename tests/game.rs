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


#[test]
fn test_new_game() {
    let g = loveletter::Game::new(4).unwrap();
    assert_eq!(g.num_players(), 4);
}
