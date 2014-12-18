/// Core logic for a round of Love Letter
///
/// A round of Love Letter ends when all players except one are eliminated, or
/// when there are no more cards to draw and the final player has played.
///
/// The winner is either the last player standing, or the player with the
/// highest-valued card.

use action;
use action::{Action, Event};
use deck;
use deck::Card;
use player;
use util;

#[deriving(Show, PartialEq, Eq)]
pub struct Turn {
    pub player: uint,
    pub hand: Card,
    pub draw: Card,
}

impl Turn {
    fn new(player: uint, hand: Card, draw: Card) -> Turn {
        Turn { player: player, hand: hand, draw: draw }
    }
}


#[deriving(Show, PartialEq, Eq, Clone)]
/// Possible states of a round of Love Letter.
///
/// ### Notes
///
/// XXX: Not sure I (jml) like this. The current Round class only accepts a
/// callback, so a player is dealt a card and must respond in the same method.
/// It is always some player's turn unless it's the beginning or end. A
/// different concept would be to have a very high level game state enum which
/// had different kinds of values depending on whether the game was over or
/// not.
///
/// e.g. Before the game, you have a Deck, a number of players and nothing
/// else. During the game, there are methods to draw a card, to play it,
/// and (probably) to inspect public state. After the game, the only thing
/// that can happen is you look at who the survivors are, what their cards
/// were, who the winner is, and what the burn card was.
enum State {
    NotStarted,
    PlayerReady(uint, Card),
    RoundOver(RoundResult),
}



#[deriving(Show, PartialEq, Eq)]
/// Errors that can occur while constructing a Round.
pub enum Error {
    /// Specified an invalid number of players.
    // TODO: Now that we're making a distinction between a game and a round,
    // the Round will be told the number of players by the Game
    InvalidPlayers(uint),
    /// The given cards do not form a valid deck.
    BadDeck,
}


#[deriving(Show)]
pub enum TurnOutcome {
    // XXX: Not sure we should include originating player id in this
    // structure, but Round currently doesn't expose whose turn that just was.
    BustedOut(uint),
    // XXX: I think there's only one use case for needing more than one Event
    // (a terrible name in itself): using the Wizard forcing someone to
    // discard the Princess. It's _possible_ we don't need that, but included
    // now for completeness.
    Played(uint, Card, action::Play, Vec<Event>),
}


#[deriving(Show, PartialEq, Eq, Clone)]
/// Represents a single round of Love Letter.
pub struct Round {
    /// The remaining cards in the deck.
    _stack: Vec<Card>,
    /// All of the players of the game. The size does not change once the game is constructed.
    _players: Vec<player::Player>,
    /// The current state of the game.
    _current: State,
}


impl Round {
    // TODO: Provide some way of getting the burnt card when play is over.

    // TODO: Create a state validator, that guarantees that no cards have been
    // created or destroyed.

    // TODO: Create a GameConfiguration object that has only the number of
    // players. We can then rely on that to be correct, allowing our
    // type-checker to prevent an invalid number of players.

    /// Create a new game with a randomly shuffled deck.
    ///
    /// Will return None if given an invalid number of players.
    pub fn new(num_players: uint) -> Option<Round> {
        Round::from_deck(num_players, deck::Deck::new())
    }

    /// Create a new game given an already-shuffled deck.
    ///
    /// Will return None if given an invalid number of players.
    pub fn from_deck(num_players: uint, deck: deck::Deck) -> Option<Round> {
        if !Round::valid_player_count(num_players) {
            return None
        }
        let cards = deck.as_slice();
        let hand_end = num_players + 1;
        let hands: Vec<Option<Card>> = cards.slice(1, hand_end).iter().map(|&x| Some(x)).collect();
        Some(Round {
            _stack: cards.slice_from(hand_end).iter().map(|&x| x).collect(),
            _current: State::NotStarted,
            _players: hands.iter().map(|&x| player::Player::new(x)).collect(),
        })
    }

    /// Create a new, in-progress game.
    ///
    /// `hands` is a slice of player hands. If the card is None, then that
    /// player is assumed to have been eliminated already. Otherwise, that's
    /// what's in their hand.
    ///
    /// `deck` is a stack of remaining cards in the deck. When players draw
    /// cards, they'll draw from the end.
    ///
    /// If `current_player` is `None`, then assume the game hasn't started.
    /// Otherwise (and this is a bit broken), the next player to play is the
    /// one **after** the one given here. e.g. `Some(0)` means it's player 1's
    /// turn next.
    pub fn from_manual(hands: &[Option<Card>], deck: &[Card],
                       current_player: Option<uint>) -> Result<Round, Error> {
        let num_players = hands.len();
        if !Round::valid_player_count(num_players) {
            return Err(Error::InvalidPlayers(num_players));
        }
        let mut stack: Vec<Card> = deck.iter().map(|&x| x).collect();
        let mut all_cards = stack.clone();
        for x in hands.as_slice().iter().filter_map(|&x| x) {
            all_cards.push(x);
        }
        if deck::is_valid_subdeck(all_cards.as_slice()) {
            let state = match current_player {
                None => State::NotStarted,
                Some(i) => {
                    match stack.pop() {
                        Some(card) => State::PlayerReady(i, card),
                        None => { return Err(Error::BadDeck); }
                    }
                }
            };
            Ok(Round {
                _stack: stack,
                _current: state,
                _players: hands.iter().map(|&x| player::Player::new(x)).collect(),
            })
        } else {
            Err(Error::BadDeck)
        }
    }

    fn valid_player_count(num_players: uint) -> bool {
        2 <= num_players && num_players <= 4
    }

    /// Number of players in this game.
    pub fn num_players(&self) -> uint {
        self._players.len()
    }

    pub fn all_discards(&self) -> Vec<&[Card]> {
        let mut discards = vec![];
        for player in self._players.iter() {
            discards.push(player.discards());
        }
        discards
    }

    #[cfg(test)]
    fn num_cards_remaining(&self) -> uint {
        self._stack.len()
    }

    /// Number of active players still playing.
    fn num_players_remaining(&self) -> uint {
        self._players.iter().filter(|p| p.active()).count()
    }

    fn current_player(&self) -> Option<uint> {
        match self._current {
            State::NotStarted => None,
            State::PlayerReady(i, _) => Some(i),
            State::RoundOver(..) => None,
        }
    }

    fn current_turn(&self) -> Option<Turn> {
        match self._current {
            State::PlayerReady(i, card) => {
                let hand = self.get_hand(i).ok().expect("Activated disabled player");
                Some(Turn::new(i, hand, card))
            },
            _ => None,
        }
    }

    fn _game_result(&self) -> RoundResult {
        // XXX: probably doesn't need to be a clone
        RoundResult::new(self._players.clone())
    }

    /// At the end of the game, return all winners and their hands.
    pub fn winners(&self) -> Vec<(uint, Card)> {
        match self.next_player() {
            (_, Some(..)) => vec![],
            (_, None) => self._game_result().winners()
        }
    }

    pub fn get_discards(&self, player_id: uint) -> Result<&[Card], action::PlayError> {
        self._players
            .get(player_id)
            .ok_or(action::PlayError::InvalidPlayer(player_id))
            .map(|p| p.discards())
    }

    fn get_player(&self, player_id: uint) -> Result<&player::Player, action::PlayError> {
        if player_id < self.num_players() {
            let ref p = self._players[player_id];
            if p.active() {
                Ok(p)
            } else {
                Err(action::PlayError::InactivePlayer(player_id))
            }
        } else {
            Err(action::PlayError::InvalidPlayer(player_id))
        }
    }

    fn get_hand(&self, player_id: uint) -> Result<Card, action::PlayError> {
        self.get_player(player_id).map(|p| p.get_hand().unwrap())
    }

    fn update_player(&self, player_id: uint, player: player::Player) -> Round {
        let mut new_game = self.clone();
        new_game._players[player_id] = player;
        new_game
    }

    fn update_player_by(&self, player_id: uint, updater: |&player::Player| -> Result<player::Player, player::Error>) -> Result<Round, action::PlayError> {
        self.get_player(player_id)
            .map(updater)
            .and_then(
                |result| match result {
                    Ok(new_player) => Ok(self.update_player(player_id, new_player)),
                    Err(player::Error::Inactive) => Err(action::PlayError::InactivePlayer(player_id)),
                    Err(player::Error::NoSuchCard(c, d)) => Err(action::PlayError::CardNotFound(c, d)),
                })
    }

    fn update_two_players_by(&self, p1_id: uint, p2_id: uint, updater: |&player::Player, &player::Player| -> Result<(player::Player, player::Player), player::Error>) -> Result<Round, action::PlayError> {
        match (self.get_player(p1_id), self.get_player(p2_id)) {
            (Ok(player1), Ok(player2)) => {
                match updater(player1, player2) {
                    Ok((new_player1, new_player2)) => {
                        let mut new_game = self.clone();
                        new_game._players[p1_id] = new_player1;
                        new_game._players[p2_id] = new_player2;
                        Ok(new_game)
                    },
                    Err(player::Error::Inactive) => Err(action::PlayError::InactivePlayer(p2_id)),
                    Err(e) => panic!(e),
                }
            },
            (_, Err(e)) | (Err(e), _) => Err(e),
        }
    }

    #[cfg(test)]
    fn hands(&self) -> Vec<Option<Card>> {
        self._players.iter().map(|ref x| x.get_hand()).collect()
    }

    #[cfg(test)]
    fn deck(&self) -> &[Card] {
        self._stack.as_slice()
    }

    fn draw(&self) -> (Round, Option<Card>) {
        let mut g = self.clone();
        let c = g._stack.pop();
        (g, c)
    }

    fn _next_player(&self) -> Option<uint> {
        if self.num_players_remaining() <= 1 {
            None
        } else {
            let current_num = match self.current_player() {
                None => -1,
                Some(i) => i,
            };
            let num_players = self.num_players();
            range(1, num_players)
                .map(|i| (current_num + i) % num_players)
                .find(|i| self._players[*i].active())
        }
    }

    fn next_player(&self) -> (Round, Option<Turn>) {
        match (self._next_player(), self.draw()) {
            (Some(new_player_id), (game, Some(c))) => {
                let mut new_game = game;
                new_game._current = State::PlayerReady(new_player_id, c);
                // Protection from the priestess expires when your
                // turn begins.
                new_game = new_game
                    .update_player_by(new_player_id, |p| p.protect(false))
                    .ok().expect("Activated disabled player");
                let turn = new_game.current_turn();
                (new_game, turn)
            },
            _ => {
                let mut new_game = self.clone();
                new_game._current = State::RoundOver(self._game_result());
                (new_game, None)
            },
        }
    }

    fn action_to_event(&self, action: Action) -> Result<Event, action::PlayError> {
        // XXX: I think the types I've chosen are working against me. If we
        // keep Play around, then we can simplify the checking code a great
        // deal (only check the target is active & not protected once).
        //
        // Perhaps it's also wise to have a type that actually has a reference
        // to a player rather than a uint
        match action {
            Action::NoChange => Ok(Event::NoChange),
            Action::Protect(i) => self.get_player(i).map(|_| Event::Protected(i)),
            Action::SwapHands(src, tgt) => {
                let target = try!(self.get_player(tgt));
                if target.protected() {
                    Ok(Event::NoChange)
                } else {
                    Ok(Event::SwappedHands(src, tgt))
                }
            },
            Action::EliminatePlayer(i) => {
                let target = try!(self.get_player(i));
                if target.protected() {
                    Ok(Event::NoChange)
                } else {
                    Ok(Event::PlayerEliminated(i))
                }
            },
            Action::ForceDiscard(i) => {
                let target = try!(self.get_player(i));
                if target.protected() {
                    Ok(Event::NoChange)
                } else {
                    Ok(Event::ForcedDiscard(i))
                }
            },
            Action::ForceReveal(src, tgt) => {
                let target = try!(self.get_player(tgt));
                if target.protected() {
                    Ok(Event::NoChange)
                } else {
                    Ok(Event::ForcedReveal(src, tgt))
                }
            },
            Action::EliminateWeaker(src, tgt) => {
                let source_hand = try!(self.get_hand(src));
                let target = try!(self.get_player(tgt));
                if target.protected() {
                    Ok(Event::NoChange)
                } else {
                    let target_hand = target.get_hand().expect("Hand should be active");
                    match source_hand.cmp(&target_hand) {
                        Less => Ok(Event::PlayerEliminated(src)),
                        Greater => Ok(Event::PlayerEliminated(tgt)),
                        Equal => Ok(Event::NoChange),
                    }
                }
            },
            Action::EliminateOnGuess(tgt, guess) => {
                if guess == Card::Soldier {
                    return Err(action::PlayError::BadGuess);
                }
                let target = try!(self.get_player(tgt));
                if target.protected() {
                    Ok(Event::NoChange)
                } else {
                    let target_hand = target.get_hand().expect("Hand should be active");
                    if target_hand == guess {
                        Ok(Event::PlayerEliminated(tgt))
                    } else {
                        Ok(Event::NoChange)
                    }
                }
            },
        }
    }

    fn apply_event(&self, event: Event) -> Result<(Round, Option<Event>), action::PlayError> {
        match event {
            Event::NoChange => Ok((self.clone(), None)),
            Event::Protected(i) => self.update_player_by(i, |player| player.protect(true)).map(|g| (g, None)),
            Event::PlayerEliminated(i) => self.update_player_by(i, |p| p.eliminate()).map(|g| (g, None)),
            Event::SwappedHands(src, tgt) => self.update_two_players_by(
                tgt, src, |tgt_player, src_player| tgt_player.swap_hands(src_player)).map(|g| (g, None)),
            Event::ForcedDiscard(i) => {
                // XXX: This can cause another event.
                let player = try!(self.get_player(i));
                if player.get_hand() == Some(Card::Princess) {
                    self.update_player_by(
                        i, |p| p.eliminate()).map(|g| (g, Some(Event::PlayerEliminated(i))))
                } else {
                    let (game, new_card) = self.draw();
                    game.update_player_by(i, |p| p.discard_and_draw(new_card)).map(|g| (g, None))
                }
            },
            Event::ForcedReveal(..) => Ok((self.clone(), None)),
        }
    }

    /// Crank the handle of a loveletter game.
    ///
    /// Takes a function which is given the game, the current player, the card
    /// they were dealt and the card in their hand. That function must return
    /// a card to play and the `Play` associated with it.
    ///
    /// `handle_turn` makes sure everything is valid and returns the new `Round`.
    ///
    /// If the game is now over, will return `Ok(None)`. If not, will return
    /// `Ok(Some(new_game))`.
    pub fn handle_turn(&self, decide_play: |&Round, &Turn| -> (Card, action::Play),
                       reveal_card: |uint, Card| -> ())
                       -> Result<Option<(Round, TurnOutcome)>, action::PlayError> {
        let (new_game, turn) = self.next_player();
        let turn = match turn {
            None => return Ok(None),
            Some(turn) => turn,
        };

        if minister_bust(turn.draw, turn.hand) {
            // XXX: Add tests to verify that the discard pile includes both
            // picked up card & held card.
            let new_game = try!(new_game.update_player_by(
                turn.player, |p| p.play_card(turn.draw, turn.draw).and_then(|p| p.eliminate())));
            Ok(Some((new_game, TurnOutcome::BustedOut(turn.player))))
        } else {
            // Find out what they'd like to play.
            let (card, play) = decide_play(&new_game, &turn);

            // Update their hand and the played card.
            let new_game = try!(new_game.update_player_by(turn.player, |p| p.play_card(turn.draw, card)));

            let action = try!(action::play_to_action(turn.player, card, play));

            let event = try!(new_game.action_to_event(action));
            match event {
                Event::ForcedReveal(_, target) =>
                    reveal_card(target, self.get_hand(target).unwrap()),
                _ => (),
            }
            let mut events = vec![event];
            let (new_game, follow_up) = try!(new_game.apply_event(event));
            match follow_up {
                Some(event) => events.push(event),
                None => (),
            };
            Ok(Some((new_game, TurnOutcome::Played(turn.player, card, play, events))))
        }
    }
}


fn minister_bust(a: Card, b: Card) -> bool {
    match util::other((a, b), Card::Minister) {
        Some(Card::Wizard) | Some(Card::General) | Some(Card::Princess) => true,
        Some(Card::Minister) => panic!("Called with 2 ministers!"),
        _ => false,
    }
}


/// The result of a finished round of Love Letter.
#[deriving(Eq, PartialEq, Show, Clone)]
pub struct RoundResult {
    _players: Vec<player::Player>,
}


impl RoundResult {

    fn new(players: Vec<player::Player>) -> RoundResult {
        RoundResult { _players: players }
    }

    /// At the end of the game, return players and their hands.
    fn survivors(&self) -> Vec<(uint, Card)> {
        self._players
            .iter()
            .enumerate()
            .filter_map(
                |(i, ref p)| match p.get_hand() {
                    Some(y) => Some((i, y)),
                    None => None,
                })
            .collect()
    }

    /// At the end of the game, return all winners and their hands.
    pub fn winners(&self) -> Vec<(uint, Card)> {
        let survivors = self.survivors();
        let mut ws = vec![];
        for x in util::maxima_by(&survivors, |&(_, card)| card).iter() {
            let &&(i, c) = x;
            ws.push((i, c))
        }
        ws
    }
}


#[cfg(test)]
mod test {
    use action;
    use action::{Event, PlayError};
    use deck;
    use deck::Card;
    use player;
    use super::{Round, Turn};
    use super::RoundResult;


    fn make_arbitrary_game() -> Round {
        Round::new(4).unwrap()
    }

    fn eliminate(g: &Round, player_id: uint) -> Result<Round, action::PlayError> {
        g.update_player_by(player_id, |p| p.eliminate())
    }

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
            Card::Soldier,
            Card::Clown,
            Card::Knight,
            Card::Priestess,
            Card::Wizard,
            Card::General,
            Card::Minister,
            Card::Princess,
            Card::Soldier,
            Card::Clown,
            Card::Soldier,
            Card::Knight,
            Card::Soldier,
            Card::Priestess,
            Card::Soldier,
            Card::Wizard,
            ];
        let deck = deck::Deck::from_slice(&cards).unwrap();
        let num_players = 3u;
        let g = Round::from_deck(num_players, deck).unwrap();
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
        let hands = vec![Some(Card::Soldier), Some(Card::Clown), Some(Card::Soldier)];
        let stack = [Card::Soldier, Card::Soldier, Card::Minister];
        let game = Round::from_manual(hands.as_slice(), &stack, None).unwrap();
        assert_eq!(hands, game.hands());
        assert_eq!(stack.as_slice(), game.deck().as_slice());
        assert_eq!(hands.len(), game.num_players());
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
        let Turn { player: p, draw: d, hand: _ } = turn.unwrap();
        let (_, expected) = g.draw();
        assert_eq!((p, d), (0, expected.unwrap()));
    }

    #[test]
    fn test_next_player_increments() {
        let g = Round::new(2).unwrap();
        let (g, _) = g.next_player();
        let (g, _) = g.next_player();
        assert_eq!(Some(1), g.current_player());
    }

    #[test]
    fn test_next_player_cycles() {
        let g = Round::new(2).unwrap();
        let (g, _) = g.next_player();
        let (g, _) = g.next_player();
        let (g, _) = g.next_player();
        assert_eq!(Some(0), g.current_player());
    }

    #[test]
    fn test_get_card_active_player() {
        let g = Round::from_manual(
            &[Some(Card::General), Some(Card::Clown), Some(Card::Knight), Some(Card::Priestess)],
            &[Card::Soldier, Card::Minister, Card::Princess, Card::Soldier, Card::Wizard], None).unwrap();
        assert_eq!(g.get_hand(0), Ok(Card::General));
    }

    #[test]
    fn test_get_card_nonexistent_player() {
        let g = Round::from_manual(
            &[Some(Card::General), Some(Card::Clown), Some(Card::Knight), Some(Card::Priestess)],
            &[Card::Soldier, Card::Minister, Card::Princess, Card::Soldier, Card::Wizard], None).unwrap();
        assert_eq!(g.get_hand(5), Err(PlayError::InvalidPlayer(5)));
    }

    #[test]
    fn test_get_card_inactive_player() {
        let g = Round::from_manual(
            &[Some(Card::General), Some(Card::Clown), None, Some(Card::Priestess)],
            &[Card::Soldier, Card::Minister, Card::Princess, Card::Soldier, Card::Wizard], None).unwrap();
        assert_eq!(g.get_hand(2), Err(PlayError::InactivePlayer(2)));
    }

    #[test]
    fn test_update_nonexistent_player() {
        let g = Round::from_manual(
            &[Some(Card::General), Some(Card::Clown), None, Some(Card::Priestess)],
            &[Card::Soldier, Card::Minister, Card::Princess, Card::Soldier, Card::Wizard], None).unwrap();
        let error = g.update_player_by(5, |p| Ok(p.clone())).unwrap_err();
        assert_eq!(PlayError::InvalidPlayer(5), error);
    }

    #[test]
    fn test_eliminate_gone_player() {
        let g = Round::from_manual(
            &[Some(Card::General), Some(Card::Clown), None, Some(Card::Priestess)],
            &[Card::Soldier, Card::Minister, Card::Princess, Card::Soldier, Card::Wizard], None).unwrap();
        let error = eliminate(&g, 2).unwrap_err();
        assert_eq!(PlayError::InactivePlayer(2), error);
    }

    #[test]
    fn test_skip_eliminated_player() {
        let g = Round::new(3).unwrap();
        let (g, _) = g.next_player();
        let g = eliminate(&g, 1).unwrap();
        let (g, t) = g.next_player();
        assert_eq!(g.current_player(), Some(2));
        assert_eq!(t.unwrap().player, 2);
    }

    fn assert_winners(game: &Round, expected_winners: Vec<uint>) {
        let observed_winners = game.winners();
        assert_eq!(expected_winners, observed_winners.iter().map(|&(i, _)| i).collect());
    }

    #[test]
    fn test_last_player() {
        let g = Round::new(2).unwrap();
        let (g, _) = g.next_player();
        let g = eliminate(&g, 1).unwrap();
        let (new_game, turn) = g.next_player();
        assert_eq!(None, turn);
        assert_winners(&new_game, vec![0]);
    }

    #[test]
    fn test_eliminate_self_last_player() {
        let g = Round::new(2).unwrap();
        let (g, _) = g.next_player();
        let g = eliminate(&g, 0).unwrap();
        let (new_game, turn) = g.next_player();
        assert_eq!(None, turn);
        assert_winners(&new_game, vec![1]);
    }

    #[test]
    fn test_swap_cards() {
        let g = Round::from_manual(
            &[Some(Card::General), Some(Card::Clown), None, Some(Card::Priestess)],
            &[Card::Soldier, Card::Minister, Card::Princess, Card::Soldier, Card::Wizard], None).unwrap();
        let (new_game, _) = g.apply_event(Event::SwappedHands(0, 1)).unwrap();
        assert_eq!(
            vec![Some(Card::Clown), Some(Card::General), None, Some(Card::Priestess)],
            new_game.hands());
    }

    #[test]
    fn test_swap_cards_nonexistent() {
        let g = Round::from_manual(
            &[Some(Card::General), Some(Card::Clown), None, Some(Card::Priestess)],
            &[Card::Soldier, Card::Minister, Card::Princess, Card::Soldier, Card::Wizard], None).unwrap();
        let error = g.apply_event(Event::SwappedHands(0, 5)).unwrap_err();
        assert_eq!(PlayError::InvalidPlayer(5), error);
        let error = g.apply_event(Event::SwappedHands(5, 0)).unwrap_err();
        assert_eq!(PlayError::InvalidPlayer(5), error);
    }

    #[test]
    fn test_no_change() {
        let g = make_arbitrary_game();
        let (new_g, _) = g.apply_event(Event::NoChange).unwrap();
        assert_eq!(g, new_g);
    }

    #[test]
    fn test_eliminate_action() {
        let g = Round::new(3).unwrap();
        let (g, _) = g.next_player();
        let (new_g, _) = g.apply_event(Event::PlayerEliminated(1)).unwrap();
        let (_, t) = new_g.next_player();
        assert_eq!(2, t.unwrap().player);
    }

    #[test]
    fn test_force_swap() {
        let g = Round::from_manual(
            &[Some(Card::Soldier), Some(Card::Clown), Some(Card::Knight)],
            &[Card::Soldier, Card::Minister, Card::Princess, Card::Soldier, Card::General], None).unwrap();
        let (g, t) = g.next_player();
        let t = t.unwrap();
        let ours = t.hand;
        let theirs = g.get_hand(1).unwrap();
        let (new_g, _) = g.apply_event(Event::SwappedHands(0, 1)).unwrap();
        assert_eq!(theirs, new_g.get_hand(0).unwrap());
        assert_eq!(ours, new_g.get_hand(1).unwrap());
    }

    #[test]
    fn test_round_result_survivors() {
        let p1 = player::Player::new(Some(Card::Princess));
        let p2 = player::Player::new(None);
        let r = RoundResult::new(vec![p1, p2]);
        assert_eq!(vec![(0, Card::Princess)], r.survivors());
    }

    #[test]
    fn test_round_result_multiple_survivors() {
        let p1 = player::Player::new(Some(Card::Princess));
        let p2 = player::Player::new(Some(Card::Wizard));
        let p3 = player::Player::new(None);
        let r = RoundResult::new(vec![p1, p2, p3]);
        assert_eq!(vec![(0, Card::Princess), (1, Card::Wizard)], r.survivors());
    }
}
