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



pub struct TiltInventory {
    pub name: PlayerName,
    pub verbose: bool,
    pub timer: Arc<Mutex<Instant>>,
    pub spades_book: Arc<Mutex<Book>>,
    pub clubs_book: Arc<Mutex<Book>>,
    pub diamonds_book: Arc<Mutex<Book>>,
    pub hearts_book: Arc<Mutex<Book>>,
    pub inventory: Arc<Mutex<Inventory>>,
    pub trades: Arc<Mutex<Vec<Trade>>>,
    pub highest_card: Arc<Mutex<Card>>,
    pub lower_frequency: u64,
    pub higher_frequency: u64,
    pub event_receiver: Sender<Event>,
    pub order_sender: Arc<AsyncSender<Order>>,
    pub trading: Arc<AtomicBool>,
}

impl TiltInventory {
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
            highest_card: Arc::new(Mutex::new(Card::Spade)),
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

            // only buy lowest card we were dealt and aggressively sell everything else
            let goal_suit = self.highest_card.lock().await.clone();
            for card in [Card::Spade, Card::Club, Card::Diamond, Card::Heart].iter() {
                if *card != goal_suit {
                    // sell 
                    let (book, current_inventory) = match card {
                        Card::Spade => (spades_book.clone(), inventory.spades),
                        Card::Club => (clubs_book.clone(), inventory.clubs),
                        Card::Diamond => (diamonds_book.clone(), inventory.diamonds),
                        Card::Heart => (hearts_book.clone(), inventory.hearts),
                    };

                    if seconds_left > 30 {
                        if book.ask.price > 4 && current_inventory > 0 && book.ask.player_name != self.name {
                            self.send_order(book.ask.price - 1, Direction::Sell, &card).await;
                        }
                    } else {
                        if book.ask.player_name != self.name {
                            self.send_order(1, Direction::Sell, &card).await;
                        }
                    }
                }
            }

            let book = match goal_suit {
                Card::Spade => spades_book,
                Card::Club => clubs_book,
                Card::Diamond => diamonds_book,
                Card::Heart => hearts_book,
            };
            if book.bid.price < 8 && book.bid.player_name != self.name {
                self.send_order(book.bid.price + 1, Direction::Buy, &goal_suit).await;
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
        let highest_card = self.highest_card.clone();
        
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

                        // doesn't take into account ties for lowest card
                        let mut highest = (Card::Spade, 12); // 12 is the highest possible card value, given the lowest amount of players
                        if inventory_lock.spades < highest.1 {
                            highest = (Card::Spade, inventory_lock.spades);
                        }
                        if inventory_lock.clubs < highest.1 {
                            highest = (Card::Club, inventory_lock.clubs);
                        }
                        if inventory_lock.diamonds < highest.1 {
                            highest = (Card::Diamond, inventory_lock.diamonds);
                        }
                        if inventory_lock.hearts < highest.1 {
                            highest = (Card::Heart, inventory_lock.hearts);
                        }
                        let goal_suit = highest.0.get_goal_suit();
                        *highest_card.lock().await = highest.0;
                        
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