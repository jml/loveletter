use super::Game;

use deck::{Soldier, Clown, Knight, Priestess, Wizard, General, Minister, Princess};

use action::{
    PlayError, NoChange, SwapHands, EliminatePlayer, InvalidPlayer, InactivePlayer,
    EliminateOnGuess, BadGuess};

mod adjudication;
mod game;


fn make_arbitrary_game() -> Game {
    Game::new(4).unwrap()
}

fn eliminate(g: &Game, player_id: uint) -> Result<Game, PlayError> {
    g.update_player_by(player_id, |p| p.eliminate())
}

#[test]
fn test_current_player_after_next() {
    let g = make_arbitrary_game();
    let (g2, _) = g.next_player();
    assert_eq!(Some(0), g2.current_player());
}

#[test]
fn test_next_player_gets_draw() {
    let g = make_arbitrary_game();
    let (_, turn) = g.next_player();
    let super::Turn { player: p, draw: d, hand: _ } = turn.unwrap();
    let (_, expected) = g.draw();
    assert_eq!((p, d), (0, expected.unwrap()));
}

#[test]
fn test_next_player_increments() {
    let g = Game::new(2).unwrap();
    let (g, _) = g.next_player();
    let (g, _) = g.next_player();
    assert_eq!(Some(1), g.current_player());
}

#[test]
fn test_next_player_cycles() {
    let g = Game::new(2).unwrap();
    let (g, _) = g.next_player();
    let (g, _) = g.next_player();
    let (g, _) = g.next_player();
    assert_eq!(Some(0), g.current_player());
}

#[test]
fn test_get_card_active_player() {
    let g = Game::from_manual(
        [Some(General), Some(Clown), Some(Knight), Some(Priestess)],
        [Soldier, Minister, Princess, Soldier, Wizard], None).unwrap();
    assert_eq!(g.get_hand(0), Ok(General));
}

#[test]
fn test_get_card_nonexistent_player() {
    let g = Game::from_manual(
        [Some(General), Some(Clown), Some(Knight), Some(Priestess)],
        [Soldier, Minister, Princess, Soldier, Wizard], None).unwrap();
    assert_eq!(g.get_hand(5), Err(InvalidPlayer(5)));
}

#[test]
fn test_get_card_inactive_player() {
    let g = Game::from_manual(
        [Some(General), Some(Clown), None, Some(Priestess)],
        [Soldier, Minister, Princess, Soldier, Wizard], None).unwrap();
    assert_eq!(g.get_hand(2), Err(InactivePlayer(2)));
}

#[test]
fn test_update_nonexistent_player() {
    let g = Game::from_manual(
        [Some(General), Some(Clown), None, Some(Priestess)],
        [Soldier, Minister, Princess, Soldier, Wizard], None).unwrap();
    let error = g.update_player_by(5, |p| Ok(p.clone())).unwrap_err();
    assert_eq!(InvalidPlayer(5), error);
}

#[test]
fn test_eliminate_gone_player() {
    let g = Game::from_manual(
        [Some(General), Some(Clown), None, Some(Priestess)],
        [Soldier, Minister, Princess, Soldier, Wizard], None).unwrap();
    let error = eliminate(&g, 2).unwrap_err();
    assert_eq!(InactivePlayer(2), error);
}

#[test]
fn test_skip_eliminated_player() {
    let g = Game::new(3).unwrap();
    let (g, _) = g.next_player();
    let g = eliminate(&g, 1).unwrap();
    let (g, t) = g.next_player();
    assert_eq!(g.current_player(), Some(2));
    assert_eq!(t.unwrap().player, 2);
}

#[test]
fn test_last_player() {
    let g = Game::new(2).unwrap();
    let (g, _) = g.next_player();
    let g = eliminate(&g, 1).unwrap();
    let (new_game, turn) = g.next_player();
    assert_eq!(None, turn);
    assert_eq!(new_game, g);
}

#[test]
fn test_eliminate_self_last_player() {
    let g = Game::new(2).unwrap();
    let (g, _) = g.next_player();
    let g = eliminate(&g, 0).unwrap();
    let (new_game, turn) = g.next_player();
    assert_eq!(None, turn);
    assert_eq!(new_game, g);
}


#[test]
fn test_swap_cards() {
    let g = Game::from_manual(
        [Some(General), Some(Clown), None, Some(Priestess)],
        [Soldier, Minister, Princess, Soldier, Wizard], None).unwrap();
    let new_game = g.apply_action(SwapHands(0, 1)).unwrap();
    assert_eq!(
        vec![Some(Clown), Some(General), None, Some(Priestess)],
        new_game.hands());
}

#[test]
fn test_swap_cards_nonexistent() {
    let g = Game::from_manual(
        [Some(General), Some(Clown), None, Some(Priestess)],
        [Soldier, Minister, Princess, Soldier, Wizard], None).unwrap();
    let error = g.apply_action(SwapHands(0, 5)).unwrap_err();
    assert_eq!(InvalidPlayer(5), error);
    let error = g.apply_action(SwapHands(5, 0)).unwrap_err();
    assert_eq!(InvalidPlayer(5), error);
}

#[test]
fn test_no_change() {
    let g = make_arbitrary_game();
    let new_g = g.apply_action(NoChange).unwrap();
    assert_eq!(g, new_g);
}

#[test]
fn test_eliminate_action() {
    let g = Game::new(3).unwrap();
    let (g, _) = g.next_player();
    let new_g = g.apply_action(EliminatePlayer(1)).unwrap();
    let (_, t) = new_g.next_player();
    assert_eq!(2, t.unwrap().player);
}

#[test]
fn test_force_swap() {
    let g = Game::from_manual(
        [Some(Soldier), Some(Clown), Some(Knight)],
        [Soldier, Minister, Princess, Soldier, General], None).unwrap();
    let (g, t) = g.next_player();
    let t = t.unwrap();
    let ours = t.hand;
    let theirs = g.get_hand(1).unwrap();
    let new_g = g.apply_action(SwapHands(0, 1)).unwrap();
    assert_eq!(theirs, new_g.get_hand(0).unwrap());
    assert_eq!(ours, new_g.get_hand(1).unwrap());
}

#[test]
fn test_eliminate_on_guess_incorrect() {
    // Got this error:
    // Player 2: pick a card:
    //   1. Priestess
    //   2. Soldier
    // 2
    // Player 2 => Soldier: Guess(0, Wizard)
    // Error: BadGuess
    let g = Game::from_manual(
        [Some(Soldier), Some(Soldier)], [Wizard, Wizard], Some(0)).unwrap();
    let result = g.apply_action(EliminateOnGuess(1, Clown));
    assert_eq!(Ok(g), result);
}

#[test]
fn test_eliminate_on_guess_correct() {
    let g = Game::from_manual(
        [Some(Soldier), Some(Clown)], [Wizard, Wizard], Some(0)).unwrap();
    let result = g.apply_action(EliminateOnGuess(1, Clown));
    assert_eq!(eliminate(&g, 1), result);
}

#[test]
fn test_eliminate_on_guess_soldier() {
    let g = Game::from_manual(
        [Some(Soldier), Some(Soldier)], [Wizard, Wizard], Some(0)).unwrap();
    let result = g.apply_action(EliminateOnGuess(1, Soldier));
    assert_eq!(Err(BadGuess), result);
}
