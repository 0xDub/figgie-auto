# Figgie Auto
An algorithmic sandbox for Jane Street's game, "Figgie". It's an excellent game that simulates "exciting elements of markets and trading". Coming from the HFT world, my strategy was to play the spread but due to laziness I figured I'd write up a program instead of learning the keybindings.

There is a lot to unpack in this game, especially when thinking about different types of market participants and how they operate. The most common route I'd imagine is to try and find the goal suit, which is the only suit that is awarded points each round. However, you could also go the route of providing liquidity to the participants, aiding in discovery. Whatever the case, I hope that the journey is fun

<hr>

### Development
- `event_driven`: This type of player makes a decision on each update. Possible branch of strategies fall under HFT
- `generic`: This player makes a decision once every few seconds (adjustable in `main.rs`). It's akin to a QR's setup

You can find barebones examples for both in the `player` folder.

<hr>

### Current Players
- `TiltInventory`: On being dealt a hand, it finds the highest card and assumes it's the common suit. It semi-aggressively bids on the predicted goal suit market while selling all their other cards
- `Spread`: A dumb market-maker, placing a wide quote range on all cards, attempting to profit off the order flow
- `Seller`: Quite conservative and defensive one, it attempts to sell all of its inventory and make up the cost of the ante
- `Noisy`: Consider this one as retail
- `PickOff`: An event-driven, opportunistic player - picking up cheap inventory in an attempt to sell it at a later price
- `TheHoarder`: The goal for this strategy is to amass 6x of each card to mathematically guarantee a win and secure the pot. High risk, low reward, yet the pitfalls are quite insightful
- `PrayingMantis`: A byproduct of `TheHoarder`'s pitfalls; like `Seller` it attempts to offload it's inventory then aggressively buys up inventory of the perceived goal suit, based on last trade price. It has its own insightful pitfalls as well

<hr>

To Jane Street: On the off chance you read this, I'd like to say I appreciate this game. If you'd like me to take it down, I completely understand and feel free to reach out

Inspired by: https://www.figgie.com/index.html