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

    pub fn eliminate_if_weaker(&self, other: Player) -> Result<(Player, Player), Error> {
        match (self._hand, other._hand) {
            (Some(my_card), Some(their_card)) => {
                if self._protected {
                    Ok((*self, other))
                } else {
                    Ok(match my_card.cmp(&their_card) {
                        Less => (self.replace(None), other),
                        Greater => (*self, other.replace(None)),
                        Equal => (*self, other)
                    })
                }
            }
            _ => Err(Inactive),
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

    pub fn discard_and_draw(&self, new_card: Option<deck::Card>) -> Result<Player, Error> {
        if !self.active() {
            Err(Inactive)
        } else if self._protected {
            Ok(*self)
        } else {
            Ok(self.replace(new_card))
        }
    }

    fn replace(&self, card: Option<deck::Card>) -> Player {
        Player { _hand: card, _protected: self._protected }
    }
}


#[cfg(test)]
mod test {
    use super::{Inactive, Player};

    #[test]
    fn test_eliminate_gone_player() {
        let p = Player::new(None);
        let error = p.eliminate().unwrap_err();
        assert_eq!(Inactive, error);
    }
}
