use deck;

#[deriving(Show, PartialEq, Eq, Clone)]
pub struct Player {
    _hand: Option<deck::Card>,
    _protected: bool,
}


#[deriving(Show, PartialEq, Eq)]
pub enum Error {
    Inactive,
    BadGuess,
    NoSuchCard(deck::Card, (deck::Card, deck::Card)),
}


impl Player {
    pub fn new(hand: Option<deck::Card>) -> Player {
        Player { _hand: hand, _protected: false }
    }

    pub fn active(&self) -> bool {
        self._hand.is_some()
    }

    pub fn get_hand(&self) -> Option<deck::Card> {
        self._hand
    }

    pub fn protected(&self) -> bool {
        self._protected
    }

    pub fn protect(&self, protected: bool) -> Result<Player, Error> {
        if self.active() {
            Ok(Player { _hand: self._hand, _protected: protected })
        } else {
            Err(Inactive)
        }
    }

    pub fn eliminate(&self) -> Result<Player, Error> {
        // Maybe check if protected?
        if !self.active() {
            Err(Inactive)
        } else if self._protected {
            Ok(*self)
        }
        else {
            Ok(Player { _hand: None, _protected: false })
        }
    }

    pub fn eliminate_if_guessed(&self, guess: deck::Card) -> Result<Player, Error> {
        if guess == deck::Soldier {
            return Err(BadGuess)
        }
        match self._hand {
            None => Err(Inactive),
            Some(card) =>
                if card == guess {
                    self.eliminate()
                } else {
                    Ok(*self)
                },
        }
    }

    pub fn swap_hands(&self, other: Player) -> Result<(Player, Player), Error> {
        if !self.active() {
            Err(Inactive)
        } else if self._protected {
            Ok((*self, other))
        } else {
            Ok((self.replace(other._hand), other.replace(self._hand)))
        }
    }

    pub fn play_card(&self, dealt: deck::Card, chosen: deck::Card) -> Result<Player, Error> {
        match self._hand {
            None => Err(Inactive),
            Some(hand) =>
                if chosen == hand {
                    Ok(self.replace(Some(dealt)))
                } else if chosen == dealt {
                    Ok(*self)
                } else {
                    Err(NoSuchCard(chosen, (hand, dealt)))
                },
        }
    }

    pub fn replace(&self, card: Option<deck::Card>) -> Player {
        Player { _hand: card, _protected: self._protected }
    }
}
