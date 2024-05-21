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

            let _seconds_left = 240 - self.timer.lock().await.elapsed().as_secs();

            let inventory = self.inventory.lock().await.clone();

            let spades_book = self.spades_book.lock().await.clone();
            let clubs_book = self.clubs_book.lock().await.clone();
            let diamonds_book = self.diamonds_book.lock().await.clone();
            let hearts_book = self.hearts_book.lock().await.clone();

            println!("{}{:?} | Inventory |:| Spades: {} | Clubs: {} | Diamonds: {} | Hearts: {}{}", CL::Dull.get(), self.name, inventory.spades, inventory.clubs, inventory.diamonds, inventory.hearts, CL::End.get());

            // with the above information, we can now decide what to do
            // core logic goes here

            match self.name {
                PlayerName::Taker => {
                    self.pick_off(inventory.spades, spades_book, Card::Spade).await;
                    self.pick_off(inventory.clubs, clubs_book, Card::Club).await;
                    self.pick_off(inventory.diamonds, diamonds_book, Card::Diamond).await;
                    self.pick_off(inventory.hearts, hearts_book, Card::Heart).await;
                },
                PlayerName::Noisy => {
                    self.noisy_trader(inventory, spades_book, clubs_book, diamonds_book, hearts_book, &mut rng).await;
                },
                PlayerName::Seller => {
                    self.sell_inventory(inventory.spades, spades_book, Card::Spade).await;
                    self.sell_inventory(inventory.clubs, clubs_book, Card::Club).await;
                    self.sell_inventory(inventory.diamonds, diamonds_book, Card::Diamond).await;
                    self.sell_inventory(inventory.hearts, hearts_book, Card::Heart).await;
                },
                PlayerName::Spread => {
                    self.provide_spread(inventory.spades, spades_book, Card::Spade).await;
                    self.provide_spread(inventory.clubs, clubs_book, Card::Club).await;
                    self.provide_spread(inventory.diamonds, diamonds_book, Card::Diamond).await;
                    self.provide_spread(inventory.hearts, hearts_book, Card::Heart).await;
                },
                _ => {}
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(rng.gen_range(self.lower_frequency..self.higher_frequency))).await; // promote some sort of fairness, HFT route should go event-driven
        }
    }



    pub async fn send_order(&self, price: usize, direction: Direction, card: &Card) {
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

    pub async fn pick_off(&self, inventory: usize, book: Book, card: Card) {
        if inventory <= 1 {
            if book.ask.price < 3 && book.ask.price != 0 {
                self.send_order(book.ask.price, Direction::Buy, &card).await;
                self.send_order(book.ask.price + 3, Direction::Sell, &card).await;
            }
        }
    }

    pub async fn noisy_trader(&self, inventory: Inventory, spades_book: Book, clubs_book: Book, diamonds_book: Book, hearts_book: Book, rng: &mut StdRng,) {
        let is_buy = rand::random::<bool>();
        match is_buy {
            true => {
                let (random_card, current_inventory, current_price) = match rng.gen_range(1..=4) {
                    1 => (Card::Spade, inventory.spades, spades_book.bid.price),
                    2 => (Card::Club, inventory.clubs, clubs_book.bid.price),
                    3 => (Card::Diamond, inventory.diamonds, diamonds_book.bid.price),
                    4 => (Card::Heart, inventory.hearts, hearts_book.bid.price),
                    _ => (Card::Spade, 0, 0) // this should never happen
                };

                let price = rng.gen_range(1..15);
                if current_inventory < 4 && price > current_price {
                    println!("NOISY |:| BUY | Random card: {:?} | Price: {}", random_card, price);
                    self.send_order(price, Direction::Sell, &random_card).await;
                }
            },
            false => {
                let (random_card, current_inventory, current_price) = match rng.gen_range(1..=4) {
                    1 => (Card::Spade, inventory.spades, spades_book.ask.price),
                    2 => (Card::Club, inventory.clubs, clubs_book.ask.price),
                    3 => (Card::Diamond, inventory.diamonds, diamonds_book.ask.price),
                    4 => (Card::Heart, inventory.hearts, hearts_book.ask.price),
                    _ => (Card::Spade, 0, 0) // this should never happen
                };

                let price = rng.gen_range(1..15);
                if current_inventory > 0 && price < current_price {
                    println!("NOISY |:| SELL | Random card: {:?} | Price: {} | current_inventory: {}", random_card, price, current_inventory);
                    self.send_order(price, Direction::Sell, &random_card).await;
                }
            }
        }
    }

    pub async fn sell_inventory(&self, inventory: usize, book: Book, card: Card) {
        if inventory > 0 {
            if let Some(last_trade) = book.last_trade {
                let price = (last_trade as f32 * 1.5).round() as usize;
                if price < book.ask.price && book.ask.player_name != self.name {
                    self.send_order(price, Direction::Sell, &card).await;
                }
            } else {
                if 6 < book.ask.price && book.ask.player_name != self.name {
                    self.send_order(book.ask.price - 1, Direction::Sell, &card).await;
                }
            }
        }
    }

    pub async fn provide_spread(&self, inventory: usize, book: Book, card: Card) {
        if inventory > 0 {
            if let Some(last_trade) = book.last_trade {

                if last_trade > 20 && book.ask.player_name != self.name {
                    self.send_order(20, Direction::Sell, &card).await;
                } else {
                    let price = (last_trade as f32 * 1.5).round() as usize;
                    if price <= book.ask.price {
                        self.send_order(book.ask.price - 1, Direction::Sell, &card).await;
                    }
                }
            } else {
                if 11 < book.ask.price && book.ask.player_name != self.name {
                    if book.ask.price > 20 {
                        self.send_order(20, Direction::Sell, &card).await;
                    } else {
                        self.send_order(book.ask.price - 1, Direction::Sell, &card).await;
                    }
                }
            }
        }
        if book.bid.price <= 5 && book.bid.player_name != self.name {
            self.send_order(book.bid.price + 1, Direction::Buy, &card).await;
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
                                inventory_lock.change(trade.card, 1);
                            } else if trade.seller == name {
                                inventory_lock.change(trade.card, -1);
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