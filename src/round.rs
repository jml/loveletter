/// Core logic for a round of Love Letter
///
/// A round of Love Letter ends when all players except one are eliminated, or
/// when there are no more cards to draw and the final player has played.
///
/// The winner is either the last player standing, or the player with the
/// highest-valued card.

use action;
use action::{Action, Event};
use config;
use deck;
use deck::Card;
use player;
use player_id;
use util;


#[deriving(Show, PartialEq, Eq)]
pub struct Turn {
    pub player: player_id::PlayerId,
    pub hand: Card,
    pub draw: Card,
}

impl Turn {
    fn new(player: player_id::PlayerId, hand: Card, draw: Card) -> Turn {
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
    PlayerReady(player_id::PlayerId, Card),
    RoundOver(RoundResult),
}



#[deriving(Show, PartialEq, Eq, Copy)]
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
    BustedOut(player_id::PlayerId),
    // XXX: I think there's only one use case for needing more than one Event
    // (a terrible name in itself): using the Wizard forcing someone to
    // discard the Princess. It's _possible_ we don't need that, but included
    // now for completeness.
    Played(player_id::PlayerId, Card, action::Play, Vec<Event>),
}


#[deriving(Show, PartialEq, Eq, Clone)]
/// Represents a single round of Love Letter.
pub struct Round {
    /// The remaining cards in the deck.
    _stack: Vec<Card>,
    /// All of the players of the game. The size does not change once the game is constructed.
    _players: Vec<(player_id::PlayerId, player::Player)>,
    /// The current state of the game.
    _current: State,
}


impl Round {
    // TODO: Provide some way of getting the burnt card when play is over.

    // TODO: Create a state validator, that guarantees that no cards have been
    // created or destroyed.

    /// Create a new game with a randomly shuffled deck.
    ///
    /// Will return None if given an invalid number of players.
    pub fn new(player_ids: &[player_id::PlayerId]) -> Round {
        Round::from_deck(player_ids, deck::Deck::new())
    }

    /// Create a new game given an already-shuffled deck.
    ///
    /// Will return None if given an invalid number of players.
    pub fn from_deck(player_ids: &[player_id::PlayerId], deck: deck::Deck) -> Round {
        // XXX: ... aaaand we now allow invalid numbers of players.
        let mut cards: Vec<Card> = deck.as_slice().iter().map(|&x| x).collect();
        let mut players = vec![];

        cards.pop().expect("Deck had no cards!");
        for &player_id in player_ids.iter() {
            let card = cards.pop().expect("Deck had too few cards!");
            let player = player::Player::new(Some(card));
            players.push((player_id, player));
        }

        Round {
            _stack: cards,
            _current: State::NotStarted,
            _players: players,
        }
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
    pub fn from_manual(players: &[(player_id::PlayerId, Option<Card>)], deck: &[Card],
                       current_player: Option<player_id::PlayerId>) -> Result<Round, Error> {
        match config::Config::new(players.len()) {
            Err(..) => return Err(Error::InvalidPlayers(players.len())),
            _ => ()
        };
        let mut stack: Vec<Card> = deck.iter().map(|&x| x).collect();
        let mut all_cards = stack.clone();
        for x in players.iter().filter_map(|&(_, x)| x) {
            all_cards.push(x);
        }
        if !deck::is_valid_subdeck(all_cards.as_slice()) {
            return Err(Error::BadDeck);
        }

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
            _players: players.iter().map(|&(id, x)| (id, player::Player::new(x))).collect(),
        })
    }

    /// Number of players in this game.
    pub fn num_players(&self) -> uint {
        self._players.len()
    }

    /// Return the player IDs in the order of play.
    fn player_ids(&self) -> Vec<player_id::PlayerId> {
        let mut ids = vec![];
        for &(id, _) in self._players.iter() {
            ids.push(id)
        }
        ids
    }

    pub fn all_discards(&self) -> Vec<&[Card]> {
        let mut discards = vec![];
        for p in self._players.iter() {
            let &(_, ref player) = p;
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
        self._players.iter().filter(|&&(_, ref p)| p.active()).count()
    }

    fn current_player(&self) -> Option<player_id::PlayerId> {
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
    pub fn winners(&self) -> Vec<(player_id::PlayerId, Card)> {
        match self.next_player() {
            (_, Some(..)) => vec![],
            (_, None) => self._game_result().winners()
        }
    }

    pub fn get_discards(&self, player_id: player_id::PlayerId) -> Result<&[Card], action::PlayError> {
        self.get_player(player_id).map(|p| p.discards())
    }

    fn _player_index(&self, player_id: player_id::PlayerId) -> uint {
        self._players
            .iter()
            .position(|&(id, _)| id == player_id)
            .expect(format!("Unknown player ID: {}", player_id).as_slice())
    }

    fn get_player(&self, player_id: player_id::PlayerId) -> Result<&player::Player, action::PlayError> {
        match self._players.iter().find(|&&(id, _)| id == player_id) {
            None => Err(action::PlayError::InvalidPlayer(player_id)),
            Some(&(_, ref player)) => if player.active() {
                Ok(player)
            } else {
                Err(action::PlayError::InactivePlayer(player_id))
            },
        }
    }

    fn get_hand(&self, player_id: player_id::PlayerId) -> Result<Card, action::PlayError> {
        self.get_player(player_id).map(|p| p.get_hand().unwrap())
    }

    fn update_player(&self, player_id: player_id::PlayerId, player: player::Player) -> Round {
        let i = self._player_index(player_id);
        let mut new_game = self.clone();
        new_game._players[i] = (player_id, player);
        new_game
    }

    fn update_player_by(&self, player_id: player_id::PlayerId, updater: |&player::Player| -> Result<player::Player, player::Error>) -> Result<Round, action::PlayError> {
        self.get_player(player_id)
            .map(|x| updater(x))
            .and_then(
                |result| match result {
                    Ok(new_player) => Ok(self.update_player(player_id, new_player)),
                    Err(player::Error::Inactive) => Err(action::PlayError::InactivePlayer(player_id)),
                    Err(player::Error::NoSuchCard(c, d)) => Err(action::PlayError::CardNotFound(c, d)),
                })
    }

    fn update_two_players_by(
        &self, p1_id: player_id::PlayerId, p2_id: player_id::PlayerId,
        updater: |&player::Player, &player::Player| -> Result<(player::Player, player::Player), player::Error>)
        -> Result<Round, action::PlayError> {

        let p1 = try!(self.get_player(p1_id));
        let p2 = try!(self.get_player(p2_id));

        match updater(p1, p2) {
            Ok((new_player1, new_player2)) => {
                Ok(self.update_player(p1_id, new_player1).update_player(p2_id, new_player2))
            },
            Err(player::Error::Inactive) => Err(action::PlayError::InactivePlayer(p2_id)),
            Err(e) => panic!(e),
        }
    }

    #[cfg(test)]
    fn hands(&self) -> Vec<Option<Card>> {
        self._players.iter().map(|&(_, ref x)| x.get_hand()).collect()
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

    fn _next_player(&self) -> Option<player_id::PlayerId> {
        if self.num_players_remaining() <= 1 {
            None
        } else {
            let current_num = match self.current_player() {
                None => -1,
                Some(player) => self._player_index(player),
            };
            let num_players = self.num_players();
            range(1, num_players)
                .map(|i| &self._players[(current_num + i) % num_players])
                .find(|&&(_, ref player)| player.active())
                .map(|&(id, _)| id)
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
                       reveal_card: |player_id::PlayerId, Card| -> ())
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
    _players: Vec<(player_id::PlayerId, player::Player)>,
}


impl RoundResult {

    fn new(players: Vec<(player_id::PlayerId, player::Player)>) -> RoundResult {
        RoundResult { _players: players }
    }

    /// At the end of the game, return players and their hands.
    fn survivors(&self) -> Vec<(player_id::PlayerId, Card)> {
        self._players
            .iter()
            .filter_map(
                |&(i, ref p)| match p.get_hand() {
                    Some(y) => Some((i, y)),
                    None => None,
                })
            .collect()
    }

    /// At the end of the game, return all winners and their hands.
    pub fn winners(&self) -> Vec<(player_id::PlayerId, Card)> {
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
    use player_id::{PlayerId, player_id_generator};
    use super::{Round, Turn};
    use super::RoundResult;


    fn make_arbitrary_game() -> Round {
        make_round(4)
    }

    fn make_player_ids(num_players: uint) -> Vec<PlayerId> {
        player_id_generator().take(num_players).collect()
    }

    fn make_round(num_players: uint) -> Round {
        let players: Vec<PlayerId> = make_player_ids(num_players);
        Round::new(players.as_slice())
    }

    fn eliminate(g: &Round, player_id: PlayerId) -> Result<Round, action::PlayError> {
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
        let num_cards = cards.len();
        let deck = deck::Deck::from_slice(&cards).unwrap();
        let num_players = 3u;
        let players = make_player_ids(num_players);
        let g = Round::from_deck(players.as_slice(), deck);
        assert_eq!(
            cards.slice(num_cards - num_players - 1, num_cards - 1)
                .iter()
                .map(|&x| Some(x))
                .collect::<Vec<Option<Card>>>(),
            g.hands());
        assert_eq!(cards.slice_to(num_cards - num_players - 1), g.deck());
        assert_eq!(num_players, g.num_players());
    }

    #[test]
    fn test_manual_game() {
        let mut id_gen = player_id_generator();
        let hands = vec![
            (id_gen.next().unwrap(), Some(Card::Soldier)),
            (id_gen.next().unwrap(), Some(Card::Clown)),
            (id_gen.next().unwrap(), Some(Card::Soldier)),
            ];
        let stack = [Card::Soldier, Card::Soldier, Card::Minister];
        let round = Round::from_manual(hands.as_slice(), &stack, None).unwrap();
        assert_eq!(
            vec![Some(Card::Soldier), Some(Card::Clown), Some(Card::Soldier)], round.hands());
        assert_eq!(stack.as_slice(), round.deck().as_slice());
        assert_eq!(hands.len(), round.num_players());
    }

    #[test]
    fn test_current_player_after_next() {
        let g = make_arbitrary_game();
        let (g2, _) = g.next_player();
        let player_ids = g.player_ids();
        assert_eq!(player_ids[0], g2.current_player().unwrap());
    }

    #[test]
    fn test_next_player_gets_draw() {
        let g = make_arbitrary_game();
        let (g2, turn) = g.next_player();
        let Turn { player: p, draw: d, hand: _ } = turn.unwrap();
        let expected_player = g2.current_player().unwrap();
        let (_, expected_draw) = g.draw();
        assert_eq!((p, d), (expected_player, expected_draw.unwrap()));
    }

    #[test]
    fn test_next_player_increments() {
        let g = make_round(2);
        let player_ids = g.player_ids();
        let (g, _) = g.next_player();
        let (g, _) = g.next_player();
        assert_eq!(player_ids[1], g.current_player().unwrap());
    }

    #[test]
    fn test_next_player_cycles() {
        let g = make_round(2);
        let player_ids = g.player_ids();
        let (g, _) = g.next_player();
        let (g, _) = g.next_player();
        let (g, _) = g.next_player();
        assert_eq!(player_ids[0], g.current_player().unwrap());
    }

    #[test]
    fn test_get_card_active_player() {
        let player_ids = make_player_ids(4);
        let g = Round::from_manual(
            &[(player_ids[0], Some(Card::General)),
              (player_ids[1], Some(Card::Clown)),
              (player_ids[2], Some(Card::Knight)),
              (player_ids[3], Some(Card::Priestess)),
              ],
            &[Card::Soldier, Card::Minister, Card::Princess, Card::Soldier, Card::Wizard], None).unwrap();
        assert_eq!(g.get_hand(player_ids[0]), Ok(Card::General));
    }

    #[test]
    fn test_get_card_nonexistent_player() {
        let players: Vec<PlayerId> = make_player_ids(5);
        let round = Round::new(players.slice_to(4));
        let bad_id = players[4];
        assert_eq!(round.get_hand(bad_id), Err(PlayError::InvalidPlayer(bad_id)));
    }

    #[test]
    fn test_get_card_inactive_player() {
        let player_ids = make_player_ids(4);
        let g = Round::from_manual(
            &[(player_ids[0], Some(Card::General)),
              (player_ids[1], Some(Card::Clown)),
              (player_ids[2], None),
              (player_ids[3], Some(Card::Priestess)),
              ],
            &[Card::Soldier, Card::Minister, Card::Princess, Card::Soldier, Card::Wizard], None).unwrap();
        assert_eq!(g.get_hand(player_ids[2]), Err(PlayError::InactivePlayer(player_ids[2])));
    }

    #[test]
    fn test_update_nonexistent_player() {
        let players: Vec<PlayerId> = make_player_ids(5);
        let round = Round::new(players.slice_to(4));
        let bad_id = players[4];
        let error = round.update_player_by(bad_id, |p| Ok(p.clone())).unwrap_err();
        assert_eq!(PlayError::InvalidPlayer(bad_id), error);
    }

    #[test]
    fn test_eliminate_gone_player() {
        let player_ids = make_player_ids(4);
        let g = Round::from_manual(
            &[(player_ids[0], Some(Card::General)),
              (player_ids[1], Some(Card::Clown)),
              (player_ids[2], None),
              (player_ids[3], Some(Card::Priestess)),
              ],
            &[Card::Soldier, Card::Minister, Card::Princess, Card::Soldier, Card::Wizard], None).unwrap();
        let error = eliminate(&g, player_ids[2]).unwrap_err();
        assert_eq!(PlayError::InactivePlayer(player_ids[2]), error);
    }

    #[test]
    fn test_skip_eliminated_player() {
        let round = make_round(3);
        let player_ids = round.player_ids();
        let (round, _) = round.next_player();
        let round = eliminate(&round, player_ids[1]).unwrap();
        let (round, t) = round.next_player();
        assert_eq!(round.current_player(), Some(player_ids[2]));
        assert_eq!(t.unwrap().player, player_ids[2]);
    }

    fn assert_winners(game: &Round, expected_winners: Vec<PlayerId>) {
        let observed_winners: Vec<PlayerId> = game
            .winners()
            .iter()
            .map(|&(i, _)| i)
            .collect();
        assert_eq!(expected_winners, observed_winners);
    }

    #[test]
    fn test_last_player() {
        let g = make_round(2);
        let players = g.player_ids();
        let (g, _) = g.next_player();
        let g = eliminate(&g, players[1]).unwrap();
        let (new_game, turn) = g.next_player();
        assert_eq!(None, turn);
        assert_winners(&new_game, vec![players[0]]);
    }

    #[test]
    fn test_eliminate_self_last_player() {
        let g = make_round(2);
        let players = g.player_ids();
        let (g, _) = g.next_player();
        let g = eliminate(&g, players[0]).unwrap();
        let (new_game, turn) = g.next_player();
        assert_eq!(None, turn);
        assert_winners(&new_game, vec![players[1]]);
    }

    #[test]
    fn test_swap_cards() {
        let round = make_round(4);
        let original_hands = round.hands();
        let players = round.player_ids();
        let (new_round, _) = round.apply_event(Event::SwappedHands(players[0], players[1])).unwrap();

        assert_eq!(
            vec![original_hands[1], original_hands[0], original_hands[2], original_hands[3]],
            new_round.hands());
    }

    #[test]
    fn test_swap_cards_nonexistent() {
        let players: Vec<PlayerId> = make_player_ids(5);
        let round = Round::new(players.slice_to(4));
        let bad_id = players[4];
        let error = round.apply_event(Event::SwappedHands(players[0], bad_id)).unwrap_err();
        assert_eq!(PlayError::InvalidPlayer(bad_id), error);
        let error = round.apply_event(Event::SwappedHands(bad_id, players[0])).unwrap_err();
        assert_eq!(PlayError::InvalidPlayer(bad_id), error);
    }

    #[test]
    fn test_no_change() {
        let g = make_arbitrary_game();
        let (new_g, _) = g.apply_event(Event::NoChange).unwrap();
        assert_eq!(g, new_g);
    }

    #[test]
    fn test_eliminate_action() {
        let g = make_round(3);
        let players = g.player_ids();
        let (g, _) = g.next_player();
        let (new_g, _) = g.apply_event(Event::PlayerEliminated(players[1])).unwrap();
        let (_, t) = new_g.next_player();
        assert_eq!(players[2], t.unwrap().player);
    }

    #[test]
    fn test_round_result_survivors() {
        let player_ids = make_player_ids(2);
        let p1 = player::Player::new(Some(Card::Princess));
        let p2 = player::Player::new(None);
        let r = RoundResult::new(vec![(player_ids[0], p1), (player_ids[1], p2)]);
        assert_eq!(vec![(player_ids[0], Card::Princess)], r.survivors());
    }

    #[test]
    fn test_round_result_multiple_survivors() {
        let player_ids = make_player_ids(3);
        let p1 = player::Player::new(Some(Card::Princess));
        let p2 = player::Player::new(Some(Card::Wizard));
        let p3 = player::Player::new(None);
        let r = RoundResult::new(
            vec![(player_ids[0], p1), (player_ids[1], p2), (player_ids[2], p3)]);
        assert_eq!(
            vec![(player_ids[0], Card::Princess), (player_ids[1], Card::Wizard)], r.survivors());
    }
}
