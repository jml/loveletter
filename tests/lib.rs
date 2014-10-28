extern crate loveletter;

mod game;

#[test]
fn test_integration() {
    let g = loveletter::Game::from_manual([], [], None);
    assert_eq!(g, Err(loveletter::InvalidPlayers(0)));
}

#[test]
fn test_new_game() {
    let g = loveletter::Game::new(4).unwrap();
    assert_eq!(g.num_players(), 4);
}
