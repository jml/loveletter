/// A whole game of Love Letter.
///
/// A game of Love Letter consists of multiple rounds. The winners of each
/// round receive a token of affection from the princess. The first players to
/// receive four tokens of affection are declared to have won her heart, and
/// thus, the game.

use player_id;
use player_id::{PlayerId, Players};
use round;


// XXX: Should this be a configuration parameter?
const WINNING_SCORE: u32 = 4;


#[derive(Clone)]
pub struct Game {
    // XXX: Possibly Game should not own Config. In the only current non-test
    // use case, Config can easily last longer than Game. The only reason we
    // want to own this is for the helper `make_game` function.
    _players: Vec<(PlayerId, u32)>,
}


impl Game {
    fn new(players: Players) -> Game {
        let players: Vec<(PlayerId, u32)> = players.iter().map(|&p| (p, 0)).collect();
        Game { _players: players }
    }

    fn num_players(&self) -> usize {
        self._players.len()
    }

    pub fn new_round(&self) -> round::Round {
        let players: Vec<PlayerId> = self._players.iter().map(|&(i, _)| i).collect();
        round::Round::new(players.as_slice())
    }

    pub fn next_round(&self) -> Option<round::Round> {
        if self.winners().len() == 0 {
            Some(self.new_round())
        } else {
            None
        }
    }

    pub fn players(&self) -> Vec<PlayerId> {
        self._players.iter().map(|&(p, _)| p).collect()
    }

    pub fn scores(&self) -> Vec<u32> {
        self._players.iter().map(|&(_, x)| x).collect()
    }

    fn player_won_mut(&mut self, player_id: PlayerId) {
        // XXX: Will panic if player_id wrong
        // XXX: What if score exceeds WINNING_SCORE
        let idx = self._players
            .iter()
            .position(|&(id, _)| id == player_id)
            .expect("No such player ID");
        let (p, score) = self._players[idx];
        self._players[idx] = (p, score + 1);
    }

    fn players_won_mut(&mut self, player_ids: &[PlayerId]) {
        // XXX: what if not unique
        for i in player_ids.iter() {
            self.player_won_mut(*i);
        }
    }

    pub fn players_won(&self, player_ids: &[PlayerId]) -> Game {
        let mut new_game = self.clone();
        new_game.players_won_mut(player_ids);
        new_game
    }

    fn winners(&self) -> Vec<PlayerId> {
        self._players
            .iter()
            .filter_map(|&(i, n)| if n >= WINNING_SCORE { Some(i) } else { None })
            .collect()
    }
}


/// Create a new game with the given number of arbitrary players.
pub fn new_game(num_players: usize) -> Option<Game> {
    player_id::make_players(num_players).map(|players| Game::new(players))
}


#[cfg(test)]
mod test {

    use player_id::{player_id_generator, PlayerId, Players};
    use super::Game;

    // XXX: Duplicated from round.rs
    fn make_player_ids(num_players: usize) -> Vec<PlayerId> {
        player_id_generator().take(num_players).collect()
    }

    fn make_game_from_players(players: &[PlayerId]) -> Game {
        Players::new(players.as_slice()).map(|players| Game::new(players)).ok().unwrap()
    }

    fn make_game(num_players: usize) -> Game {
        super::new_game(num_players).unwrap()
    }

    #[test]
    fn test_num_players() {
        let game = make_game(4);
        assert_eq!(game.num_players(), 4);
    }

    #[test]
    fn test_make_round() {
        let g = make_game(4);
        let r = g.new_round();
        assert_eq!(r.num_players(), g.num_players());
    }

    #[test]
    fn initial_scores_zero() {
        let game = make_game(4);
        let expected = vec![0, 0, 0, 0];
        assert_eq!(expected, game.scores());
    }

    #[test]
    fn test_one_player_winning() {
        let players = make_player_ids(2);
        let mut game = make_game_from_players(players.as_slice());
        let expected = vec![0, 1];
        game.player_won_mut(players[1]);
        assert_eq!(expected, game.scores());
    }

    #[test]
    fn many_players_winning() {
        let players = make_player_ids(4);
        let mut game = make_game_from_players(players.as_slice());
        let expected = vec![0, 1, 1, 0];
        game.players_won_mut(&[players[1], players[2]]);
        assert_eq!(expected, game.scores());
    }

    #[test]
    fn immutable_players_winning() {
        let players = make_player_ids(4);
        let game = make_game_from_players(players.as_slice());
        let expected = vec![0, 1, 1, 0];
        let new_game = game.players_won(&[players[1], players[2]]);
        let new_scores = new_game.scores();
        assert_eq!(expected, new_scores);
    }

    #[test]
    fn initial_winners() {
        let game = make_game(4);
        assert_eq!(vec![], game.winners());
    }

}
