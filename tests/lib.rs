extern crate loveletter;

mod game;

#[test]
fn test_integration() {
    let g = loveletter::Game::from_manual([], [], None);
    assert_eq!(g, Err(loveletter::InvalidPlayers(0)));
}
