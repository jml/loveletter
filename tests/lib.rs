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

#[test]
fn test_minister_eliminates_player() {
    let g = loveletter::Game::from_manual(
        [Some(loveletter::General), Some(loveletter::Soldier)],
        [loveletter::Wizard, loveletter::Minister], None).unwrap();
    let new_g = g.handle_turn(|_, _| (loveletter::General, loveletter::Attack(1))).unwrap();
    assert_eq!(vec![(1, loveletter::Soldier)], new_g.unwrap().winners());
}

#[test]
fn test_minister_eliminates_player_2() {
    let g = loveletter::Game::from_manual(
        [Some(loveletter::Minister), Some(loveletter::Soldier)],
        [loveletter::Wizard, loveletter::Wizard], None).unwrap();
    let new_g = g.handle_turn(|_, _| (loveletter::General, loveletter::Attack(1))).unwrap();
    assert_eq!(vec![(1, loveletter::Soldier)], new_g.unwrap().winners());
}
