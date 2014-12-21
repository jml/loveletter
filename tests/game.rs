extern crate loveletter;

use loveletter::Card;
use loveletter::RoundError;


#[test]
fn test_new_round_from_game() {
    let game = loveletter::game::new_game(4).unwrap();
    let round = game.new_round();
    assert_eq!(round.num_players(), 4);
}

#[test]
fn test_invalid_manual_game() {
    let game = loveletter::game::new_game(4).unwrap();
    let players = game.players();
    let stack = [
        Card::Soldier,
        Card::Princess,
        Card::Minister,
        ];
    let result = loveletter::Round::from_manual(
        &[(players[0], Some(Card::Soldier)),
          (players[1], Some(Card::Clown)),
          (players[2], Some(Card::Soldier)),
          (players[3], Some(Card::Princess))],
        &stack, None).unwrap_err();
    assert_eq!(result, RoundError::BadDeck);
}

#[test]
fn test_manual_game_bad_players() {
    assert_eq!(Err(RoundError::InvalidPlayers(0)), loveletter::Round::from_manual(&[], &[], None));
}
