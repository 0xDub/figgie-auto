use super::PlayerName;

#[derive(Debug, Clone)]
pub struct Quote {
    pub price: usize,
    pub player_name: PlayerName,
}

#[derive(Debug, Clone)]
pub struct Book {
    pub bid: Quote,
    pub ask: Quote,
    pub last_trade: Option<usize>,
}

impl Book {
    pub fn reset_quotes(&mut self) {
        self.bid = Quote {
            price: 0,
            player_name: PlayerName::None,
        };
        self.ask = Quote {
            price: 99,
            player_name: PlayerName::None,
        };
    }

    pub fn update_last_trade(&mut self, price: usize) {
        self.last_trade = Some(price);
    }
}

impl Book {
    pub fn new() -> Self {
        Self {
            bid: Quote {
                price: 0,
                player_name: PlayerName::None,
            },
            ask: Quote {
                price: 99,
                player_name: PlayerName::None,
            },
            last_trade: None,
        }
    }
}