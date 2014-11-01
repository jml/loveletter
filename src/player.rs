use deck;

#[deriving(Show, PartialEq, Eq, Clone)]
pub struct Player {
    _hand: Option<deck::Card>,
    _protected: bool,
}


impl Player {
    pub fn new(hand: Option<deck::Card>) -> Player {
        Player { _hand: hand, _protected: false }
    }

    pub fn protected(&self) -> bool {
        self._protected
    }

    pub fn protect(&self, protected: bool) -> Player {
        Player { _hand: self._hand, _protected: protected }
    }

    pub fn eliminate(&self) -> Player {
        // Maybe check if protected?
        Player { _hand: None, _protected: self._protected }
    }

    pub fn active(&self) -> bool {
        self._hand.is_some()
    }

    pub fn swap_hands(&self, other: Player) -> (Player, Player) {
        (self.replace(other._hand), other.replace(self._hand))
    }

    pub fn replace(&self, card: Option<deck::Card>) -> Player {
        Player { _hand: card, _protected: self._protected }
    }

    pub fn get_hand(&self) -> Option<deck::Card> {
        self._hand
    }
}
