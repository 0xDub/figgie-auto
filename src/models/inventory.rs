use super::{Card};


#[derive(Debug, Clone, Copy)]
pub struct Inventory {
    pub spades: usize,
    pub clubs: usize,
    pub diamonds: usize,
    pub hearts: usize,
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            spades: 0,
            clubs: 0,
            diamonds: 0,
            hearts: 0,
        }
    }

    pub fn count(&mut self, cards: Vec<Card>) {
        for card in cards {
            match card {
                Card::Spade => self.spades += 1,
                Card::Club => self.clubs += 1,
                Card::Diamond => self.diamonds += 1,
                Card::Heart => self.hearts += 1,
            }
        }
    }

    pub fn change(&mut self, card: Card, add: bool) {
        match card {
            Card::Spade => {
                let new_amount: usize;
                if add {
                    new_amount = self.spades + 1;
                } else {
                    new_amount = self.spades - 1;
                }
                self.spades = new_amount;
            },
            Card::Club => {
                let new_amount: usize;
                if add {
                    new_amount = self.clubs + 1;
                } else {
                    new_amount = self.clubs - 1;
                }
                self.clubs = new_amount;
            },
            Card::Diamond => {
                let new_amount: usize;
                if add {
                    new_amount = self.diamonds + 1;
                } else {
                    new_amount = self.diamonds - 1;
                }
                self.diamonds = new_amount;
            },
            Card::Heart => {
                let new_amount: usize;
                if add {
                    new_amount = self.hearts + 1;
                } else {
                    new_amount = self.hearts - 1;
                }
                self.hearts = new_amount;
            },
        }
    }

    pub fn get(&self, card: &Card) -> usize {
        match card {
            Card::Spade => self.spades,
            Card::Club => self.clubs,
            Card::Diamond => self.diamonds,
            Card::Heart => self.hearts,
        }
    }
}