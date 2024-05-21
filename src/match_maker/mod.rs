use super::{Card, Book, Inventory, Order, Event, Update, Trade, Direction, CL, PlayerName};
use tokio::sync::broadcast::Sender;
use rand::prelude::SliceRandom;
use kanal::AsyncReceiver;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::sync::Arc;
use rand::Rng;
use std::collections::HashMap;

pub struct MatchMaker {
    pub round: u32,
    pub player_names: Vec<PlayerName>,
    pub suits: [Card; 4],
    pub goal_suit: Card,
    pub common_suit: Card,
    pub player_points: HashMap<PlayerName, usize>,
    pub books: HashMap<Card, Book>,
    pub player_inventories: HashMap<PlayerName, Inventory>,
    pub event_sender: Sender<Event>,
    pub order_receiver: Arc<AsyncReceiver<Order>>,
    pub rng: StdRng,
}

impl MatchMaker {
    pub fn new(
        starting_balance: usize,
        player_names: Vec<PlayerName>,
        event_sender: Sender<Event>,
        order_receiver: Arc<AsyncReceiver<Order>>,
    ) -> Self {

        let mut player_inventories = HashMap::new();
        let mut player_points = HashMap::new();
        for player_name in &player_names {
            player_points.insert(player_name.clone(), starting_balance);
            player_inventories.insert(player_name.clone(), Inventory::new());
        }

        let mut books = HashMap::new();
        books.insert(Card::Spade, Book::new());
        books.insert(Card::Club, Book::new());
        books.insert(Card::Diamond, Book::new());
        books.insert(Card::Heart, Book::new());


        Self {
            round: 0,
            player_names,
            suits: [Card::Spade, Card::Club, Card::Diamond, Card::Heart],
            goal_suit: Card::Spade,
            common_suit: Card::Club,
            player_points,
            books,
            player_inventories,
            event_sender,
            order_receiver,
            rng: StdRng::from_entropy(),
        }
    }

    pub fn pick_new_common_suit(&mut self) {
        self.common_suit = self.suits[self.rng.gen_range(0..=3)].clone();
    }

    pub fn get_new_inventories(&mut self) {
        let mut cards: Vec<Card> = Vec::new();
        let (goal_suit, suit_1, suit_2) = self.common_suit.get_other_cards();
        self.goal_suit = goal_suit.clone();

        for _ in 0..12 { cards.push(self.common_suit.clone()) }

        // randomly pick one of the other 3 suits to be the one with 8 cards
        let already_lucky = false;
        for (idx, suit) in [suit_1, suit_2, goal_suit].iter().enumerate() {
            let lucky_eight = rand::random::<bool>();
            if idx == 2 && !already_lucky {
                for _ in 0..8 { cards.push(suit.clone()) }
            } else {
                if !already_lucky && lucky_eight {
                    for _ in 0..8 { cards.push(suit.clone()) }
                } else {
                    for _ in 0..10 { cards.push(suit.clone()) }
                }
            }
        }

        cards.shuffle(&mut self.rng); // randomly shuffle the cards

        let chunk_size = 40 / self.player_names.len();
        let chunks: Vec<&[Card]> = cards.chunks(chunk_size).collect();

        for (i, player_name) in self.player_names.iter().enumerate() {
            let mut player_inventory = Inventory::new();
            player_inventory.count(chunks[i].to_vec());
            self.player_inventories.insert(player_name.clone(), player_inventory.clone());
        }
    }



    pub async fn start(&mut self) {
        let round_duration = tokio::time::Duration::from_secs(30); // 4 minutes per round

        loop {
            let mut pot = 0;
            let ante = 200 / self.player_names.len();

            println!("{}==================== ROUND {} ===================={}", CL::Purple.get(), self.round, CL::End.get());
            println!("{} - Players: {}x{}", CL::Dull.get(), self.player_names.len(), CL::End.get());
            println!("{} - Ante: {}{}", CL::Dull.get(), ante, CL::End.get());
            println!("{} - Pot: 200{}", CL::Dull.get(), CL::End.get());
            
            let initial_points = self.player_points.clone();
            for (player, points) in self.player_points.iter_mut() {
                if *points < ante {
                    println!("[!] Player {:?} does not have enough points to play", player);
                    break;
                }
                *points -= ante;
                pot += ante;
            }

            self.pick_new_common_suit();
            self.get_new_inventories();

            println!("{} - Common suit: {:?}{}", CL::Dull.get(), self.common_suit, CL::End.get());
            println!("{} - Goal suit: {}{:?}{}{}", CL::Dull.get(), CL::LimeGreen.get(), self.goal_suit, CL::End.get(), CL::End.get());
            println!("");

            println!("{}[+] Dealing cards...{}\n", CL::DimLightBlue.get(), CL::End.get());
            
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await; // give the players a little bit to get ready
            
            if let Err(e) = self.event_sender.send(Event::DealCards(self.player_inventories.clone())) {
                println!("{}[!] Error sending deal cards event: {:?}{}", CL::Red.get(), e, CL::End.get());
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await; // give the players some time to order their cards

            // send out the book
            let book_event = Event::Update(Update {
                spades: self.books.get(&Card::Spade).unwrap().clone(),
                clubs: self.books.get(&Card::Club).unwrap().clone(),
                diamonds: self.books.get(&Card::Diamond).unwrap().clone(),
                hearts: self.books.get(&Card::Heart).unwrap().clone(),
                trade: None,
            });
            if let Err(e) = self.event_sender.send(book_event) {
                println!("[!] Error sending book event: {:?}", e);
            }


            let start = tokio::time::Instant::now();
            while start.elapsed() < round_duration {

                if let Ok(order) = self.order_receiver.recv().await {
                    if order.price == 0 { // No free lunches allowed
                        continue;
                    }

                    let book = self.books.get_mut(&order.card).unwrap();
                    let trade: Option<Trade> = match order.direction {
                        Direction::Buy => {
                            if order.price >= book.ask.price {
                                println!("{}[-] Aggressing Player: {:?} | {:?} |:| Matched buy order!{}", CL::Green.get(), order.player_name, order.card, CL::End.get());


                                // =-= Update the Inventories =-= //
                                // add to the buyer
                                let buyer_inventory = self.player_inventories.get_mut(&order.player_name).unwrap();
                                buyer_inventory.change(order.card.clone(), 1);

                                // subtract from the seller
                                let seller_id = book.ask.player_name.clone();
                                let seller_inventory = self.player_inventories.get_mut(&seller_id).unwrap();
                                seller_inventory.change(order.card.clone(), -1);


                                // =-= Update the Points =-= //
                                // add to the seller (current best player_id of the ask)
                                let seller_id = book.ask.player_name.clone();
                                let seller_points = self.player_points.get_mut(&seller_id).unwrap();
                                *seller_points += book.ask.price;

                                // subtract from the buyer
                                let buyer_id = order.player_name;
                                let buyer_points = self.player_points.get_mut(&buyer_id).unwrap();
                                *buyer_points -= book.ask.price;


                                // =-= Package Trade =-= //
                                book.last_trade = Some(book.ask.price);
                                let trade = Trade {
                                    card: order.card.clone(),
                                    price: book.ask.price,
                                    buyer: buyer_id,
                                    seller: seller_id,
                                };
                                Some(trade)

                            } else {
                                // check if this price beats the current best bid
                                if order.price > book.bid.price {
                                    // update the bid price and user_id
                                    book.bid.price = order.price;
                                    book.bid.player_name = order.player_name;
                                }
                                None
                            }
                        },
                        Direction::Sell => {
                            // check if the user has the inventory to sell this Card
                            let seller_inventory = self.player_inventories.get(&order.player_name).unwrap();
                            if seller_inventory.get(&order.card) == 0 {
                                println!("[!] {:?} | {:?} |:| Player does not have the inventory to sell this Card", order.player_name, order.card);
                                continue;
                            }

                            if order.price <= book.bid.price {
                                println!("{}[-] Aggressing Player: {:?} | {:?} |:| Matched sell order!{}", CL::Red.get(), order.player_name, order.card, CL::End.get());

                                // =-= Update the Inventories =-= //
                                // add to the buyer
                                let buyer_inventory = self.player_inventories.get_mut(&order.player_name).unwrap();
                                buyer_inventory.change(order.card.clone(), 1);

                                // subtract from the seller
                                let seller_id = book.bid.player_name.clone();
                                let seller_inventory = self.player_inventories.get_mut(&seller_id).unwrap();
                                seller_inventory.change(order.card.clone(), -1);


                                // =-= Update the Points =-= //
                                // add to the seller (current best player_id of the bid)
                                let seller_id = book.bid.player_name.clone();
                                let seller_points = self.player_points.get_mut(&seller_id).unwrap();
                                *seller_points += book.bid.price;

                                // subtract from the buyer
                                let buyer_id = order.player_name;
                                let buyer_points = self.player_points.get_mut(&buyer_id).unwrap();
                                *buyer_points -= book.bid.price;
                                

                                // =-= Package Trade =-= //
                                book.last_trade = Some(book.bid.price);
                                let trade = Trade {
                                    card: order.card.clone(),
                                    price: book.bid.price,
                                    buyer: buyer_id,
                                    seller: seller_id,
                                };
                                Some(trade)

                            } else {
                                // check if this price beats the current best bid
                                if order.price < book.ask.price {
                                    // update the bid price and user_id
                                    book.ask.price = order.price;
                                    book.ask.player_name = order.player_name;
                                }
                                None
                            }
                        },
                    };

                    if let Some(_) = trade.clone() {
                        // =-= Reset all the Books =-= //
                        self.books.get_mut(&Card::Spade).unwrap().reset_quotes();
                        self.books.get_mut(&Card::Club).unwrap().reset_quotes();
                        self.books.get_mut(&Card::Diamond).unwrap().reset_quotes();
                        self.books.get_mut(&Card::Heart).unwrap().reset_quotes();

                        // =-= Drain the Order Receiver =-= //
                        let drain_amount = self.order_receiver.len();
                        for _ in 0..drain_amount {
                            let _ = self.order_receiver.try_recv();
                        }
                    }

                    // =-= Print the Game =-= //
                    println!("\n{}=--------------------------------------------------------------------------={}", CL::Dull.get(), CL::End.get());

                    let spades = self.books.get(&Card::Spade).unwrap();
                    let clubs = self.books.get(&Card::Club).unwrap();
                    let diamonds = self.books.get(&Card::Diamond).unwrap();
                    let hearts = self.books.get(&Card::Heart).unwrap();
                    println!("{}Spades    |:| Bid: ({:?}, {:?}) | Ask: ({:?}, {:?}) |:| Last trade: {:?}{}", CL::DullTeal.get(), spades.bid.price,    spades.bid.player_name,    spades.ask.price,    spades.ask.player_name,    spades.last_trade, CL::End.get());
                    println!("{}Clubs     |:| Bid: ({:?}, {:?}) | Ask: ({:?}, {:?}) |:| Last trade: {:?}{}", CL::DullTeal.get(), clubs.bid.price,     clubs.bid.player_name,     clubs.ask.price,     clubs.ask.player_name,     clubs.last_trade, CL::End.get());
                    println!("{}Diamonds  |:| Bid: ({:?}, {:?}) | Ask: ({:?}, {:?}) |:| Last trade: {:?}{}", CL::DullTeal.get(), diamonds.bid.price,  diamonds.bid.player_name,  diamonds.ask.price,  diamonds.ask.player_name,  diamonds.last_trade, CL::End.get());
                    println!("{}Hearts    |:| Bid: ({:?}, {:?}) | Ask: ({:?}, {:?}) |:| Last trade: {:?}{}", CL::DullTeal.get(), hearts.bid.price,    hearts.bid.player_name,    hearts.ask.price,    hearts.ask.player_name,    hearts.last_trade, CL::End.get());
                    
                    let mut inventory_string = String::from("Points    |:| ");
                    for player_name in &self.player_names {
                        let player_points = self.player_points.get(player_name).unwrap();
                        inventory_string += &format!("{:?}: {} | ", player_name, player_points);
                    }
                    inventory_string.truncate(inventory_string.len() - 3);

                    println!("{}{}{}", CL::DullGreen.get(), inventory_string, CL::End.get());
                    println!("{}=--------------------------------------------------------------------------={}\n", CL::Dull.get(), CL::End.get());

                    let update = Update {
                        spades: self.books.get(&Card::Spade).unwrap().clone(),
                        clubs: self.books.get(&Card::Club).unwrap().clone(),
                        diamonds: self.books.get(&Card::Diamond).unwrap().clone(),
                        hearts: self.books.get(&Card::Heart).unwrap().clone(),
                        trade,
                    };
                    let update_event = Event::Update(update);

                    if let Err(e) = self.event_sender.send(update_event) {
                        println!("[!] Error sending update event: {:?}", e);
                    }
                }
            } 

            // =-= End the Round =-= //
            let end_round = Event::EndRound;
            if let Err(e) = self.event_sender.send(end_round) {
                println!("[!] Error sending end round event: {:?}", e);
            }

            println!("");
            println!("{}=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-={}", CL::Pink.get(), CL::End.get());
            println!("{}=---=---=---=---=---=---= Round over! =---=---=---=---=---=---={}", CL::Pink.get(), CL::End.get());
            println!("{}=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-={}", CL::Pink.get(), CL::End.get());
            println!("\n");
            
            println!("=------------ Game Details ------------=");
            println!("{} - Players: {}x{}", CL::Dull.get(), self.player_names.len(), CL::End.get());
            println!("{} - Ante: {}{}", CL::Dull.get(), ante, CL::End.get());
            println!("{} - Pot: {}{}", CL::Dull.get(), pot, CL::End.get());
            println!("{} - Common suit: {:?}{}", CL::Dull.get(), self.common_suit, CL::End.get());
            println!("{} - Goal suit: {}{:?}{}{}", CL::Dull.get(), CL::LimeGreen.get(), self.goal_suit, CL::End.get(), CL::End.get());
            println!("");

            self.round += 1;

            // calculate the scores, each player is awared goal_suit * 10
            // and the player with the most of the goal_suit is awarded 50

            // get each players inventory and if add their points, simulatentously subtracting from pot
            let mut winner: (PlayerName, usize) = (PlayerName::None, 0); // player_id, goal_cards
            let mut tied_winnders: Vec<PlayerName> = Vec::new(); // player_ids

            println!("=----------------------- Inventory -----------------------=");
            for player_name in &self.player_names {
                let inventory = self.player_inventories.get(player_name).unwrap();
                let player_points = self.player_points.get_mut(player_name).unwrap();
                let goal_cards = match self.goal_suit {
                    Card::Spade => inventory.spades,
                    Card::Club => inventory.clubs,
                    Card::Diamond => inventory.diamonds,
                    Card::Heart => inventory.hearts,
                };

                let (spade_color, club_color, diamond_color, heart_color) = match self.goal_suit {
                    Card::Spade => (CL::LimeGreen.get(), CL::Dull.get(), CL::Dull.get(), CL::Dull.get()),
                    Card::Club => (CL::Dull.get(), CL::LimeGreen.get(), CL::Dull.get(), CL::Dull.get()),
                    Card::Diamond => (CL::Dull.get(), CL::Dull.get(), CL::LimeGreen.get(), CL::Dull.get()),
                    Card::Heart => (CL::Dull.get(), CL::Dull.get(), CL::Dull.get(), CL::LimeGreen.get()),
                };

                println!("{}{}{:?}{} |:| Spades: {}{}x{} | Clubs: {}{}x{} | Diamonds: {}{}x{} | Hearts: {}{}x{}{}", CL::Dull.get(), CL::DimLightBlue.get(), player_name, CL::Dull.get(), spade_color, inventory.spades, CL::Dull.get(), club_color, inventory.clubs, CL::Dull.get(), diamond_color, inventory.diamonds, CL::Dull.get(), heart_color, inventory.hearts, CL::End.get(), CL::End.get());

                if goal_cards >= winner.1 {
                    if goal_cards == winner.1 {
                        tied_winnders.push(player_name.clone());
                    } else {
                        winner = (player_name.clone(), goal_cards);
                        tied_winnders.clear();
                    }
                }

                *player_points += goal_cards * 10;
                pot -= goal_cards * 10;
            }
            println!("");

            // if there's one winner, award them the pot
            // if there's a tie, split the pot evenly between the winners

            println!("=------------------------ Results ------------------------=");
            if tied_winnders.is_empty() {
                println!("{}[+] Player '{:?}' wins the whole pot of {} points{}", CL::Green.get(), winner.0, pot, CL::End.get());
                let winner_points = self.player_points.get_mut(&winner.0).unwrap();
                *winner_points += pot;
            } else {
                let split = pot / tied_winnders.len();
                println!("{}[+] Players tie for the pot of {} points{}\n", CL::Teal.get(), pot, CL::End.get());
                println!("{}------ Tied Players ------{}", CL::Dull.get(), CL::End.get());
                println!("{}{}{:?}{} | Goal Cards: {}x | Points: {}+{}x{}{}", CL::Dull.get(), CL::DimLightBlue.get(), winner.0, CL::Dull.get(), winner.1, CL::LimeGreen.get(), split, CL::End.get(), CL::End.get());
                for player_name in tied_winnders {
                    println!("{}{}{:?}{} | Goal Cards: {}x | Points: {}+{}x{}{}", CL::Dull.get(), CL::DimLightBlue.get(), player_name, CL::Dull.get(), winner.1, CL::LimeGreen.get(), split, CL::End.get(), CL::End.get());
                    let player_points = self.player_points.get_mut(&player_name).unwrap();
                    *player_points += split;
                }
            }
            println!("");

            println!("=--------------------- Updated Points --------------------=");
            let mut inventory_string = String::from("Points |:| ");
            for player_name in &self.player_names {
                let initial_points = initial_points.get(player_name).unwrap();
                let player_points = self.player_points.get(player_name).unwrap();
                let point_change: i32 = *player_points as i32 - *initial_points as i32;

                let change_color = match point_change {
                    x if x > 0 => CL::Green.get(),
                    x if x < 0 => CL::Red.get(),
                    _ => CL::Dull.get(),
                };

                inventory_string += &format!("{:?}: {} {}({}){} | ", player_name, player_points, change_color, point_change, CL::Dull.get());
            }
            inventory_string.truncate(inventory_string.len() - 3);
            println!("{}{}{}", CL::Dull.get(), inventory_string, CL::End.get());
            println!("");

            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

        }

    }

}