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

    pub fn change(&mut self, card: Card, amount: i32) {
        match card {
            Card::Spade => self.spades = (self.spades as i32 + amount) as usize,
            Card::Club => self.clubs = (self.clubs as i32 + amount) as usize,
            Card::Diamond => self.diamonds = (self.diamonds as i32 + amount) as usize,
            Card::Heart => self.hearts = (self.hearts as i32 + amount) as usize,
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