use super::{Card, Direction, Book, Trade, Inventory, Order, Event, CL, PlayerName};
use kanal::{AsyncSender};
use tokio::sync::broadcast::{Sender, Receiver};
use std::sync::Arc;
use rand::rngs::StdRng;
use rand::SeedableRng;
use tokio::sync::Mutex;
use rand::Rng;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::Instant;



pub struct GenericPlayer {
    pub name: PlayerName,
    pub verbose: bool,
    pub timer: Arc<Mutex<Instant>>,
    pub spades_book: Arc<Mutex<Book>>,
    pub clubs_book: Arc<Mutex<Book>>,
    pub diamonds_book: Arc<Mutex<Book>>,
    pub hearts_book: Arc<Mutex<Book>>,
    pub inventory: Arc<Mutex<Inventory>>,
    pub trades: Arc<Mutex<Vec<Trade>>>,
    pub lower_frequency: u64,
    pub higher_frequency: u64,
    pub event_receiver: Sender<Event>,
    pub order_sender: Arc<AsyncSender<Order>>,
    pub trading: Arc<AtomicBool>,
}

impl GenericPlayer {
    pub fn new(
        player_name: PlayerName,
        verbose: bool,
        lower_frequency: u64,
        higher_frequency: u64,
        event_receiver: Sender<Event>,
        order_sender: Arc<AsyncSender<Order>>,
    ) -> Self {
        Self {
            name: player_name,
            verbose,
            timer: Arc::new(Mutex::new(std::time::Instant::now())),
            spades_book: Arc::new(Mutex::new(Book::new())),
            clubs_book: Arc::new(Mutex::new(Book::new())),
            diamonds_book: Arc::new(Mutex::new(Book::new())),
            hearts_book: Arc::new(Mutex::new(Book::new())),
            inventory: Arc::new(Mutex::new(Inventory::new())),
            trades: Arc::new(Mutex::new(Vec::new())),
            lower_frequency,
            higher_frequency,
            event_receiver,
            order_sender,
            trading: Arc::new(AtomicBool::new(false)),
        }
    }



    pub async fn start(&mut self) {
        self.listen_to_events().await;

        let mut rng = StdRng::from_entropy();
        loop {

            let trading_flag = self.trading.load(Ordering::Acquire);
            if !trading_flag {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                continue;
            }

            let seconds_left = 240 - self.timer.lock().await.elapsed().as_secs();

            let inventory = self.inventory.lock().await.clone();

            let spades_book = self.spades_book.lock().await.clone();
            let clubs_book = self.clubs_book.lock().await.clone();
            let diamonds_book = self.diamonds_book.lock().await.clone();
            let hearts_book = self.hearts_book.lock().await.clone();

            println!("{}{:?} | Inventory |:| Spades: {} | Clubs: {} | Diamonds: {} | Hearts: {}{}", CL::Dull.get(), self.name, inventory.spades, inventory.clubs, inventory.diamonds, inventory.hearts, CL::End.get());

            // with the above information, we can now decide what to do
            // core logic goes here (examples below)


            // =-= Notes =-= //
            // - The first possible edge comes from the difference of probabilities between the common suit and the others. For example,
            // the probability of getting 4x of the common suit is 13%, while the probabilities of the others are either 7.4% (10 card suit) or 3.3% (8 card suit) (I think so anyway, using this calculator: https://stattrek.com/online-calculator/hypergeometric)
            // - Another edge comes from understanding the value of the cards. Such as starting cost / value of cards given other strategies
            // - Lastly, the flow of information throughout the game is highly important. If each trade is deliberate, it must contain some amount of information which can be used
            // ------------- //
            // - The role of a market maker in this system is quite interesting. If we extrapolate the values of the cards to the end of the game, we have 3/4 worth 0 and 1/4 worth 10 + possible bonus,
            // this extrapolation leaves the market maker in an interesting position, exposed to the extremes of toxic flow. Like the real market, the experience of their competitors is highly correlated with the
            // effectiveness of the market maker. A dumb market maker can win with noisy players, but will lose to players who are able to predict the goal suit. How to handle this is a fun problem to dive into
            match self.name {
                PlayerName::PrayingMantis => {
                    self.praying_mantis_sell(seconds_left, inventory.spades, spades_book.clone(), Card::Spade).await;
                    self.praying_mantis_sell(seconds_left, inventory.clubs, clubs_book.clone(), Card::Club).await;
                    self.praying_mantis_sell(seconds_left, inventory.diamonds, diamonds_book.clone(), Card::Diamond).await;
                    self.praying_mantis_sell(seconds_left, inventory.hearts, hearts_book.clone(), Card::Heart).await;

                    let mut cards = vec![
                        (Card::Spade, spades_book.last_trade.unwrap_or(0)),
                        (Card::Club, clubs_book.last_trade.unwrap_or(0)),
                        (Card::Diamond, diamonds_book.last_trade.unwrap_or(0)),
                        (Card::Heart, hearts_book.last_trade.unwrap_or(0)),
                    ];

                    cards.sort_by(|a, b| b.1.cmp(&a.1));

                    let most_expensive_card = cards[0].0.clone();
                    let (inventory, book) = match most_expensive_card {
                        Card::Spade => (inventory.spades, spades_book),
                        Card::Club => (inventory.clubs, clubs_book),
                        Card::Diamond => (inventory.diamonds, diamonds_book),
                        Card::Heart => (inventory.hearts, hearts_book),
                    };

                    self.praying_mantis_snipe(seconds_left, inventory, book, most_expensive_card).await;
                },
                PlayerName::TheHoarder => {
                    self.hoard(seconds_left, inventory.spades, spades_book, Card::Spade).await;
                    self.hoard(seconds_left, inventory.clubs, clubs_book, Card::Club).await;
                    self.hoard(seconds_left, inventory.diamonds, diamonds_book, Card::Diamond).await;
                    self.hoard(seconds_left, inventory.hearts, hearts_book, Card::Heart).await;
                },
                PlayerName::Noisy => {
                    self.noisy_trader(inventory, spades_book, clubs_book, diamonds_book, hearts_book, &mut rng).await;
                },
                PlayerName::Seller => {
                    self.sell_inventory(seconds_left, inventory.spades, spades_book, Card::Spade).await;
                    self.sell_inventory(seconds_left, inventory.clubs, clubs_book, Card::Club).await;
                    self.sell_inventory(seconds_left, inventory.diamonds, diamonds_book, Card::Diamond).await;
                    self.sell_inventory(seconds_left, inventory.hearts, hearts_book, Card::Heart).await;
                },
                PlayerName::Spread => {
                    let average_inventory = (inventory.spades + inventory.clubs + inventory.diamonds + inventory.hearts) / 4;
                    self.provide_spread(seconds_left, average_inventory, inventory.spades, spades_book, Card::Spade).await;
                    self.provide_spread(seconds_left, average_inventory, inventory.clubs, clubs_book, Card::Club).await;
                    self.provide_spread(seconds_left, average_inventory, inventory.diamonds, diamonds_book, Card::Diamond).await;
                    self.provide_spread(seconds_left, average_inventory, inventory.hearts, hearts_book, Card::Heart).await;
                },
                _ => {}
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(rng.gen_range(self.lower_frequency..self.higher_frequency))).await; // promote some sort of fairness, HFT route should go event-driven
        }
    }


    pub async fn send_order(&self, price: usize, direction: Direction, card: &Card, book: &Book) {

        let mut trade = false;
        match direction {
            Direction::Buy => {
                if book.bid.price < price && book.bid.player_name != self.name {
                    trade = true;
                }
            },
            Direction::Sell => {
                if book.ask.price > price && book.ask.player_name != self.name {
                    trade = true;
                }
            }
        }
        
        if trade {
            let order = Order {
                player_name: self.name.clone(),
                price,
                direction,
                card: card.clone(),
            };
    
            if self.verbose {
                println!("{:?} |:| Sending order: {:?}", self.name, order);
            }
    
            if let Err(e) = self.order_sender.send(order).await {
                println!("[!] {:?} |:| Error sending order: {:?}", self.name, e);
            }
        }
        
    }

    pub async fn noisy_trader(&self, inventory: Inventory, spades_book: Book, clubs_book: Book, diamonds_book: Book, hearts_book: Book, rng: &mut StdRng,) {
        let (random_card, current_inventory, book) = match rng.gen_range(1..=4) {
            1 => (Card::Spade, inventory.spades, spades_book),
            2 => (Card::Club, inventory.clubs, clubs_book),
            3 => (Card::Diamond, inventory.diamonds, diamonds_book),
            4 => (Card::Heart, inventory.hearts, hearts_book),
            _ => (Card::Spade, 0, spades_book) // this should never happen
        };
        
        let is_buy = rand::random::<bool>();
        match is_buy {
            true => {
                let price = rng.gen_range(1..15);
                if current_inventory < 4 {
                    println!("NOISY |:| BUY | Random card: {:?} | Price: {}", random_card, price);
                    self.send_order(price, Direction::Buy, &random_card, &book).await;
                }
            },
            false => {
                let price = rng.gen_range(1..15);
                if current_inventory > 0 {
                    println!("NOISY |:| SELL | Random card: {:?} | Price: {} | current_inventory: {}", random_card, price, current_inventory);
                    self.send_order(price, Direction::Sell, &random_card, &book).await;
                }
            }
        }
    }

    pub async fn praying_mantis_sell(&self, seconds_left: u64, inventory: usize, book: Book, card: Card) {
        // - this is the byproduct of seeing what happens with TheHoarder. Instead of hoarding, which incurs a disproportionate amount of toxic flow,
        // we'll wait to the last minute and buy up the inventory of the goal card. While we wait for the last minute, we'll sell off the other inventory
        // - something of note for this strategy: it assumes that the market is right and that the goal card is the most valuable card,
        // if the participants are not rational / operating at a high skill level, this strategy will simply not work
        if seconds_left >= 60 {
            if inventory > 0 && book.ask.price >= 7 {
                self.send_order(book.ask.price - 1, Direction::Sell, &card, &book).await;
            }
        }
    }

    pub async fn praying_mantis_snipe(&self, seconds_left: u64, inventory: usize, book: Book, card: Card) {
        if seconds_left <= 60 {
            if book.ask.price <= 9 { // at this point in the game, we shouldn't expect to gain the most goal suits, so we'll try and pick off any < 10 to net +(10-price)
                self.send_order(book.ask.price, Direction::Buy, &card, &book).await;
            }
        }
    }

    pub async fn hoard(&self, seconds_left: u64, inventory: usize, book: Book, card: Card) {
        // - the goal for this strategy is to amass 6x of each card (add +16 cards in total), to mathematically guarantee a win and secure the pot
        // - the budget for each card: 7.5; which means, if, on average, each card is paid > 7.5 for, it loses money, and if < 7.5, it makes money
        // - this strategy and Seller go well together, however, the Seller gets a better deal, whereas TheHoarder still needs +8 other cards
        // - regardless though, this is all-or-nothing - high-risk, low/medium-reward

        // - an interesting pattern emerges with this strategy, it's easy to gather 6x of cards that aren't the goal suit,
        // and as time goes on, the card which isn't yet maxxed out if almost certainly the goal card. knowing this, we can adjust our strategy mid game,
        // and sell off all of our inventory at the last minute and aggressively buy the goal card, which will likely be at a premium

        // - however, on the other side, perhaps we wait like a praying mantis, and then at the last minute, aggressively buy up the most expensive card,
        // as much as possible without dipping into the net negative territory. first, sell off *all* inventory at breakeven / slight premium (~5-7),
        // then wait till the last minute of the game and buy up the most expensive card, which will likely be the goal card
        // -- (side note) this won't work in a game where the participants are advanced, as they would have already known about this goal card ahead of you,
        // and buy up with more effective buying power

        if inventory < 6 { // we need to buy more
            // we're going to aggressively buy up inventory at first, assuming that information about the goal card is not known,
            // keeping the goal premium lower, and then as time goes on we'll pick up other inventory at a lower price on the offchance
            if seconds_left >= 120 {
                if book.ask.price <= 7 {
                    self.send_order(book.ask.price, Direction::Buy, &card, &book).await;
                } else {
                    if book.bid.price < 7 {
                        self.send_order(book.bid.price + 1, Direction::Buy, &card, &book).await;
                    }
                }
            } else if seconds_left > 60 && seconds_left < 120 {
                if book.ask.price <= 6 {
                    self.send_order(book.ask.price, Direction::Buy, &card, &book).await;
                } else {
                    if book.bid.price < 6 {
                        self.send_order(book.bid.price + 1, Direction::Buy, &card, &book).await;
                    }
                }
            } else {
                if book.ask.price <= 4 {
                    self.send_order(book.ask.price, Direction::Buy, &card, &book).await;
                } else {
                    if book.bid.price < 4 {
                        self.send_order(book.bid.price + 1, Direction::Buy, &card, &book).await;
                    }
                }
            }
        }
    }

    pub async fn sell_inventory(&self, seconds_left: u64, inventory: usize, book: Book, card: Card) {
        // - to net even with 5 players, the inventory must be sold at an average price of ~5
        // - we expect the worthless cards to be valued less and less as times goes on,
        // and the goal card to be valued more and more
        // - depending on the market participants, this process can vary in speed, but the general idea is
        // to follow this expectation and sell the inventory less and less - knowing the goal card will always be picked off
        // - strategy adaptation: see which inventory is picked off first, then quickly sell off the other inventory while
        // bidding back the picked off inventory (betting on the market bring right)
        if inventory > 0 {
            if seconds_left >= 180 {
                // market: 6, limit: 7
                if book.bid.price >= 6 {
                    self.send_order(book.bid.price, Direction::Sell, &card, &book).await;
                }
                self.send_order(8, Direction::Sell, &card, &book).await;
            } else if seconds_left > 120 && seconds_left < 180 {
                // market: 5, limit: 6
                if book.bid.price >= 5 {
                    self.send_order(book.bid.price, Direction::Sell, &card, &book).await;
                }
                self.send_order(6, Direction::Sell, &card, &book).await;
            } else if seconds_left > 60 && seconds_left < 120 {
                // market: 4, limit: 5
                if book.bid.price >= 4 {
                    self.send_order(book.bid.price, Direction::Sell, &card, &book).await;
                }
                self.send_order(6, Direction::Sell, &card, &book).await;
            } else {
                // market: 3, limit: 4
                if book.bid.price >= 3 {
                    self.send_order(book.bid.price, Direction::Sell, &card, &book).await;
                }
                self.send_order(4, Direction::Sell, &card, &book).await;
            }
        }
    }

    pub async fn provide_spread(&self, seconds_left: u64, average_inventory: usize, inventory: usize, book: Book, card: Card) {
        // - spread should balance their book at the very least, inventory -> 0 or $$$, and if it's imbalanced, there's a higher chance it's worthless (due to asymmetric information)
        // - therefore, it should try to keep its book balanced to make up for the 0s it'll inevitably face at time 0
        // - buy more of the inventory that's lower than the average, sell more of the inventory that's higher than the average
        // - in other words, let's skew our quotes a little bit
        // - to do so, let's use "+2" vs "+1" and vice versa for the asks
        if inventory > 0 {
            if let Some(last_trade) = book.last_trade {
                if inventory > average_inventory {
                    self.send_order(last_trade + 1, Direction::Sell, &card, &book).await; // want to sell more
                } else {
                    self.send_order(last_trade + 2, Direction::Sell, &card, &book).await; // want to sell less
                }
            } else {
                if book.ask.price > 7 {
                    if inventory > average_inventory {
                        self.send_order(book.ask.price - 2, Direction::Sell, &card, &book).await; // want to sell more
                    } else {
                        self.send_order(book.ask.price - 1, Direction::Sell, &card, &book).await; // want to sell less
                    }
                }
            }
        } 

        if seconds_left > 20 { // we expect flow to gradually become more toxic as time goes on so we'll refrain from buying in these last 20 seconds
            if let Some(last_trade) = book.last_trade {
                if last_trade > 2 {
                    if inventory > average_inventory {
                        self.send_order(last_trade - 2, Direction::Buy, &card, &book).await; // want to buy less
                    } else {
                        self.send_order(last_trade - 1, Direction::Buy, &card, &book).await; // want to buy more
                    }
                } else {
                    self.send_order(1, Direction::Buy, &card, &book).await;
                }
            } else {
                if book.bid.price < 7 {
                    if inventory > average_inventory {
                        self.send_order(book.bid.price + 1, Direction::Buy, &card, &book).await; // want to buy less
                    } else {
                        self.send_order(book.bid.price + 2, Direction::Buy, &card, &book).await; // want to buy more
                    }
                }
            }
        }
        
    }

    pub async fn listen_to_events(&mut self) {
        
        let mut event_receiver: Receiver<Event> = self.event_receiver.subscribe();

        let diamonds_book: Arc<Mutex<Book>> = self.diamonds_book.clone();
        let spades_book: Arc<Mutex<Book>> = self.spades_book.clone();
        let hearts_book: Arc<Mutex<Book>> = self.hearts_book.clone();
        let clubs_book: Arc<Mutex<Book>> = self.clubs_book.clone();

        let inventory: Arc<Mutex<Inventory>> = self.inventory.clone();
        let trades: Arc<Mutex<Vec<Trade>>> = self.trades.clone();
        let trading: Arc<AtomicBool> = self.trading.clone();

        let name: PlayerName = self.name.clone();
        let verbose: bool = self.verbose;
        let timer = self.timer.clone();
        
        tokio::task::spawn(async move {
            loop {

                let event = event_receiver.recv().await.unwrap();
                match event {
                    Event::Update(update) => {
                        if let Some(trade) = update.trade { // push trade for historical reasons (if we want to analyze) & update inventory
                            let mut trade_lock = trades.lock().await;
                            trade_lock.push(trade.clone());

                            let mut inventory_lock = inventory.lock().await;
                            if trade.buyer == name {
                                inventory_lock.change(trade.card, true);
                            } else if trade.seller == name {
                                inventory_lock.change(trade.card, false);
                            }
                        }


                        let mut spades_lock = spades_book.lock().await;
                        *spades_lock = update.spades;
                        
                        let mut clubs_lock = clubs_book.lock().await;
                        *clubs_lock = update.clubs;

                        let mut diamonds_lock = diamonds_book.lock().await;
                        *diamonds_lock = update.diamonds;

                        let mut hearts_lock = hearts_book.lock().await;
                        *hearts_lock = update.hearts;

                    }
                    Event::DealCards(players_inventory) => {
                        let mut inventory_lock = inventory.lock().await;
                        *inventory_lock = players_inventory.get(&name).unwrap().clone();
                        
                        if verbose {
                            println!("{}[+] {:?} |:| Received cards: {:?}{}", CL::DullGreen.get(), name, inventory_lock, CL::End.get());
                        }
                        
                        trading.store(true, Ordering::Release);
                        let mut timer_lock = timer.lock().await;
                        *timer_lock = Instant::now();
                    },
                    Event::EndRound => {
                        trading.store(false, Ordering::Release);
                    }
                }

            }
        });
    }

}