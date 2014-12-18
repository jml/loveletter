/// A whole game of Love Letter.
///
/// A game of Love Letter consists of multiple rounds. The winners of each
/// round receive a token of affection from the princess. The first players to
/// receive four tokens of affection are declared to have won her heart, and
/// thus, the game.

use action::PlayerId;
use config;
use round;


// XXX: Should this be a configuration parameter?
const WINNING_SCORE: uint = 4u;


#[deriving(Clone)]
pub struct Game {
    // XXX: Possibly Game should not own Config. In the only current non-test
    // use case, Config can easily last longer than Game. The only reason we
    // want to own this is for the helper `make_game` function.
    _config: config::Config,
    _scores: Vec<uint>,
}


impl Game {
    fn new(config: config::Config) -> Game {
        let scores = Vec::from_elem(config.num_players(), 0u);
        Game { _config: config, _scores: scores }
    }

    fn num_players(&self) -> uint {
        self._config.num_players()
    }

    pub fn new_round(&self) -> round::Round {
        round::Round::new(&self._config)
    }

    pub fn next_round(&self) -> Option<round::Round> {
        if self.winners().len() == 0 {
            Some(self.new_round())
        } else {
            None
        }
    }

    pub fn scores(&self) -> &[uint] {
        self._scores.as_slice()
    }

    fn player_won_mut(&mut self, player_id: PlayerId) {
        // XXX: Will panic if player_id wrong
        // XXX: What if score exceeds WINNING_SCORE
        self._scores[player_id] += 1;
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
        self._scores
            .iter()
            .enumerate()
            .filter_map(|(i, &n)| if n >= WINNING_SCORE { Some(i) } else { None })
            .collect()
    }
}


pub fn new_game(num_players: uint) -> Result<Game, config::Error> {
    config::Config::new(num_players).map(|cfg| Game::new(cfg))
}


#[cfg(test)]
mod test {

    use config;
    use super::Game;

    fn make_game(num_players: uint) -> Game {
        let cfg = config::Config::new(num_players).ok().unwrap();
        Game::new(cfg)
    }

    #[test]
    fn test_num_players() {
        let num_players = 4;
        let cfg = config::Config::new(num_players).ok().unwrap();
        let g = Game::new(cfg);
        assert_eq!(g.num_players(), num_players);
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
        let expected = [0, 0, 0, 0];
        assert_eq!(expected.as_slice(), game.scores());
    }

    #[test]
    fn test_one_player_winning() {
        let mut game = make_game(4);
        let expected = [0, 1, 0, 0];
        game.player_won_mut(1);
        assert_eq!(expected.as_slice(), game.scores());
    }

    #[test]
    fn many_players_winning() {
        let mut game = make_game(4);
        let expected = [0, 1, 1, 0];
        game.players_won_mut(&[1, 2]);
        assert_eq!(expected.as_slice(), game.scores());
    }

    #[test]
    fn immutable_players_winning() {
        let game = make_game(4);
        let expected = [0, 1, 1, 0];
        let new_game = game.players_won(&[1, 2]);
        let new_scores = new_game.scores();
        assert_eq!(expected.as_slice(), new_scores);
    }

    #[test]
    fn initial_winners() {
        let game = make_game(4);
        assert_eq!(vec![], game.winners());
    }

}
