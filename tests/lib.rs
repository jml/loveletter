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

fn next_turn(g: &loveletter::Game, c: loveletter::Card, p: loveletter::Play) -> loveletter::Game {
    g.handle_turn(|_, _| (c, p)).unwrap().unwrap()
}

#[test]
fn test_minister_eliminates_player() {
    let g = loveletter::Game::from_manual(
        [Some(loveletter::General), Some(loveletter::Soldier)],
        [loveletter::Wizard, loveletter::Minister], None).unwrap();
    let new_g = next_turn(&g, loveletter::General, loveletter::Attack(1));
    assert_eq!(vec![(1, loveletter::Soldier)], new_g.winners());
}

#[test]
fn test_minister_eliminates_player_2() {
    let g = loveletter::Game::from_manual(
        [Some(loveletter::Minister), Some(loveletter::Soldier)],
        [loveletter::Wizard, loveletter::Wizard], None).unwrap();
    let new_g = next_turn(&g, loveletter::General, loveletter::Attack(1));
    assert_eq!(vec![(1, loveletter::Soldier)], new_g.winners());
}


#[test]
fn test_priestess_immune() {
    let g = loveletter::Game::from_manual(
        [Some(loveletter::Priestess), Some(loveletter::Soldier)],
        [loveletter::Clown, loveletter::Wizard], None).unwrap();
    let new_g = next_turn(&g, loveletter::Priestess, loveletter::NoEffect);
    let new_g = next_turn(&new_g, loveletter::Soldier, loveletter::Guess(0, loveletter::Wizard));
    // If player 0 is not protected by the priestess, then at this point,
    // player 1 will have won. If 0 *is* protected, then they win, because
    // they have the Wizard, which is higher than the Clown.
    assert_eq!(vec![(0, loveletter::Wizard)], new_g.winners());
}

// TODO: Priestess immunity expires.
// TODO: Priestess immunity for other actions
// TODO: Error checking for bad guess.
