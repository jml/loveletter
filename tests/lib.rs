extern crate loveletter;


fn next_turn_err(g: &loveletter::Game, c: loveletter::Card, p: loveletter::Play) -> loveletter::PlayError {
    g.handle_turn(|_, _| (c, p)).unwrap_err()
}

fn next_turn(g: &loveletter::Game, c: loveletter::Card, p: loveletter::Play) -> loveletter::Game {
    g.handle_turn(|_, _| (c, p)).unwrap().unwrap()
}


#[test]
fn test_invalid_target_attack() {
    let g = loveletter::Game::from_manual(
        [Some(loveletter::General), Some(loveletter::Clown)],
        [loveletter::Soldier, loveletter::Minister, loveletter::Princess, loveletter::Soldier], None).unwrap();
    assert_eq!(
        loveletter::InvalidPlayer(4),
        next_turn_err(&g, loveletter::General, loveletter::Attack(4)));
}

#[test]
fn test_invalid_target_guess() {
    let g = loveletter::Game::from_manual(
        [Some(loveletter::General), Some(loveletter::Clown)],
        [loveletter::Soldier, loveletter::Minister, loveletter::Princess, loveletter::Soldier], None).unwrap();
    assert_eq!(
        loveletter::InvalidPlayer(4),
        next_turn_err(&g, loveletter::Soldier, loveletter::Guess(4, loveletter::Wizard)));
}

#[test]
fn test_inactive_player_attack() {
    let g = loveletter::Game::from_manual(
        [Some(loveletter::General), Some(loveletter::Clown), None],
        [loveletter::Soldier, loveletter::Minister, loveletter::Princess, loveletter::Soldier], None).unwrap();
    assert_eq!(
        loveletter::InactivePlayer(2),
        next_turn_err(&g, loveletter::General, loveletter::Attack(2)));
}

#[test]
fn test_inactive_player_guess() {
    let g = loveletter::Game::from_manual(
        [Some(loveletter::General), Some(loveletter::Clown), None],
        [loveletter::Soldier, loveletter::Minister, loveletter::Princess, loveletter::Soldier], None).unwrap();
    assert_eq!(
        loveletter::InactivePlayer(2),
        next_turn_err(&g, loveletter::Soldier, loveletter::Guess(2, loveletter::Wizard)));
}

#[test]
fn test_bad_guess() {
    let g = loveletter::Game::from_manual(
        [Some(loveletter::General), Some(loveletter::Clown), None],
        [loveletter::Soldier, loveletter::Minister, loveletter::Princess, loveletter::Soldier], None).unwrap();
    assert_eq!(
        loveletter::BadGuess,
        next_turn_err(&g, loveletter::Soldier, loveletter::Guess(1, loveletter::Soldier)));
}

#[test]
fn test_princess_discard_loses() {
    let g = loveletter::Game::from_manual(
        [Some(loveletter::Princess), Some(loveletter::Clown), None],
        [loveletter::Soldier, loveletter::Minister, loveletter::Soldier, loveletter::Soldier], None).unwrap();
    let new_g = next_turn(&g, loveletter::Princess, loveletter::NoEffect);
    assert_eq!(vec![(1, loveletter::Clown)], new_g.winners());
}

#[test]
fn test_princess_forced_discard_loses() {
    let g = loveletter::Game::from_manual(
        [Some(loveletter::Wizard), Some(loveletter::Princess), None],
        [loveletter::Soldier, loveletter::Minister, loveletter::Soldier, loveletter::Soldier], None).unwrap();
    let new_g = next_turn(&g, loveletter::Wizard, loveletter::Attack(1));
    assert_eq!(vec![(0, loveletter::Soldier)], new_g.winners());
}

#[test]
fn test_princess_self_forced_discard_loses() {
    let g = loveletter::Game::from_manual(
        [Some(loveletter::Wizard), Some(loveletter::Soldier), None],
        [loveletter::Soldier, loveletter::Minister, loveletter::Soldier, loveletter::Princess], None).unwrap();
    let new_g = next_turn(&g, loveletter::Wizard, loveletter::Attack(0));
    assert_eq!(vec![(1, loveletter::Soldier)], new_g.winners());
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
fn test_priestess_immune_to_soldier_guess() {
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

#[test]
fn test_priestess_immune_to_clown() {
    let g = loveletter::Game::from_manual(
        [Some(loveletter::Priestess), Some(loveletter::Clown)],
        [loveletter::Clown, loveletter::Wizard], None).unwrap();
    let new_g = next_turn(&g, loveletter::Priestess, loveletter::NoEffect);
    let new_g = next_turn(&new_g, loveletter::Clown, loveletter::Attack(0));
    // If player 0 is not protected by the priestess, then at this point,
    // player 1 will have won. If 0 *is* protected, then they win, because
    // they have the Wizard, which is higher than the Clown.
    assert_eq!(vec![(0, loveletter::Wizard)], new_g.winners());
}

#[test]
fn test_priestess_immune_to_knight() {
    let g = loveletter::Game::from_manual(
        [Some(loveletter::Priestess), Some(loveletter::Knight)],
        [loveletter::Soldier, loveletter::Wizard, loveletter::Minister, loveletter::Wizard], None).unwrap();
    let new_g = next_turn(&g, loveletter::Priestess, loveletter::NoEffect);
    let new_g = next_turn(&new_g, loveletter::Knight, loveletter::Attack(0));
    let new_g = next_turn(&new_g, loveletter::Wizard, loveletter::Attack(1));
    // If player 0 is not protected by the priestess, then at this point,
    // player 1 will have won. If 0 *is* protected, then they win, because
    // they have the Wizard, which is higher than the Clown.
    assert_eq!(vec![(0, loveletter::Wizard)], new_g.winners());
}

#[test]
fn test_priestess_immune_to_wizard() {
    let g = loveletter::Game::from_manual(
        [Some(loveletter::Priestess), Some(loveletter::Wizard)],
        [loveletter::Clown, loveletter::Wizard], None).unwrap();
    let new_g = next_turn(&g, loveletter::Priestess, loveletter::NoEffect);
    let new_g = next_turn(&new_g, loveletter::Wizard, loveletter::Attack(0));
    // If player 0 is not protected by the priestess, then at this point,
    // player 1 will have won. If 0 *is* protected, then they win, because
    // they have the Wizard, which is higher than the Clown.
    assert_eq!(vec![(0, loveletter::Wizard)], new_g.winners());
}

#[test]
fn test_priestess_immune_to_general() {
    let g = loveletter::Game::from_manual(
        [Some(loveletter::Priestess), Some(loveletter::General)],
        [loveletter::Soldier, loveletter::Princess], None).unwrap();
    let new_g = next_turn(&g, loveletter::Priestess, loveletter::NoEffect);
    let new_g = next_turn(&new_g, loveletter::General, loveletter::Attack(0));
    // If player 0 is not protected by the priestess, then at this point,
    // player 1 will have won. If 0 *is* protected, then they win, because
    // they have the Wizard, which is higher than the Clown.
    assert_eq!(vec![(0, loveletter::Princess)], new_g.winners());
}

#[test]
fn test_priestess_immunity_expires() {
    let g = loveletter::Game::from_manual(
        [Some(loveletter::Priestess), Some(loveletter::Soldier)],
        [loveletter::Soldier, loveletter::Wizard, loveletter::Clown, loveletter::Clown], None).unwrap();
    let new_g = next_turn(&g, loveletter::Priestess, loveletter::NoEffect);
    let new_g = next_turn(&new_g, loveletter::Soldier, loveletter::Guess(0, loveletter::Wizard));
    let new_g = next_turn(&new_g, loveletter::Clown, loveletter::Attack(1));
    let new_g = next_turn(&new_g, loveletter::Soldier, loveletter::Guess(0, loveletter::Wizard));
    // If player 0 is not protected by the priestess, then at this point,
    // player 1 will have won. If 0 *is* protected, then they win, because
    // they have the Wizard, which is higher than the Clown.
    assert_eq!(vec![(1, loveletter::Clown)], new_g.winners());
}
