use deck;
use deck::{Card, Soldier, Clown, Knight, Priestess, Wizard, General, Minister, Princess};

use super::super::{BadDeck, Game, InvalidPlayers};
use super::make_arbitrary_game;


#[test]
fn test_current_player_at_start() {
    assert_eq!(None, make_arbitrary_game().current_player());
}

#[test]
fn test_num_cards_remaining_on_new() {
    // make a new game, make sure that the number & kinds of cards matches the
    // rules (5 soldiers, 2 clowns, etc.)
    let g = make_arbitrary_game();
    assert_eq!(11, g.num_cards_remaining())
}

#[test]
fn test_all_cards_in_game() {
    // make a new game, make sure that the number & kinds of cards matches the
    // rules (5 soldiers, 2 clowns, etc.). One card is 'burned' before dealing.
    let g = make_arbitrary_game();
    let mut full_deck: Vec<Card> = deck::Deck::new().iter().map(|x| *x).collect();
    full_deck.sort();
    let mut found_cards: Vec<Card> = g.deck().iter().map(|x| *x).collect();
    for card in g.hands().iter() {
        match *card {
            Some(c) => found_cards.push(c),
            None    => ()
        }
    }
    found_cards.sort();
    for card in found_cards.iter() {
        let found = full_deck.iter().position(|c| c == card);
        match found {
            Some(i) => full_deck.swap_remove(i),
            None    => panic!("card in game that's not in deck: {}", card),
        };
    }
    assert_eq!(1, full_deck.len());
    found_cards.push(full_deck[0]);

    let mut fresh_deck: Vec<Card> = deck::Deck::new().iter().map(|x| *x).collect();
    fresh_deck.sort();
    found_cards.sort();
    assert_eq!(fresh_deck, found_cards);
}

#[test]
fn test_from_deck() {
    let cards = [
        Soldier,
        Clown,
        Knight,
        Priestess,
        Wizard,
        General,
        Minister,
        Princess,
        Soldier,
        Clown,
        Soldier,
        Knight,
        Soldier,
        Priestess,
        Soldier,
        Wizard,
        ];
    let deck = deck::Deck::from_slice(cards).unwrap();
    let num_players = 3u;
    let g = Game::from_deck(num_players, deck).unwrap();
    assert_eq!(
        cards.slice(1, num_players + 1)
            .iter()
            .map(|&x| Some(x))
            .collect::<Vec<Option<Card>>>(),
        g.hands());
    assert_eq!(cards.slice_from(num_players + 1), g.deck());
    assert_eq!(num_players, g.num_players());
}

#[test]
fn test_manual_game() {
    let hands = vec![Some(Soldier), Some(Clown), Some(Soldier)];
    let stack = [Soldier, Soldier, Minister];
    let game = Game::from_manual(hands.as_slice(), stack, None).unwrap();
    assert_eq!(hands, game.hands());
    assert_eq!(stack.as_slice(), game.deck().as_slice());
    assert_eq!(hands.len(), game.num_players());
}

#[test]
fn test_invalid_manual_game() {
    let hands = [Some(Soldier), Some(Clown), Some(Soldier), Some(Princess)];
    let stack = [Soldier, Princess, Minister];
    let result = Game::from_manual(hands, stack, None);
    match result {
        Ok(_)  => panic!("Had two Princesses, should not be ok."),
        Err(e) => assert_eq!(e, BadDeck)
    }
}

#[test]
fn test_manual_game_bad_players() {
    assert_eq!(Err(InvalidPlayers(0)), Game::from_manual([], [], None));
}

#[test]
fn test_survivors_at_game_end() {
    let g = Game::from_manual([Some(Knight), Some(Princess)], [Soldier], Some(0)).unwrap();
    assert_eq!(vec![(0, Knight), (1, Princess)], g.survivors());
}

#[test]
fn test_winner_from_multiple_survivors() {
    let g = Game::from_manual([Some(Knight), Some(Princess)], [Soldier], Some(0)).unwrap();
    assert_eq!(vec![(1, Princess)], g.winners());
}
