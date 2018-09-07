extern crate rand;
use rand::Rng;
use rand::SeedableRng;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Suit {
    Spades,
    Hearts,
    Clubs,
    Diamonds,
}

impl Suit {
    fn is_red(&self) -> bool {
        *self == Suit::Hearts || *self == Suit::Diamonds
    }
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Suit::Spades => write!(f, "\u{2660}"),
            Suit::Hearts => write!(f, "\u{2665}"),
            Suit::Clubs => write!(f, "\u{2663}"),
            Suit::Diamonds => write!(f, "\u{2666}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Card {
    Regular(Suit, u8),
    Joker(u8),
    Special(Suit),
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Card::Regular(s, 1) => write!(f, "{}A", s),
            Card::Regular(s, 13) => write!(f, "{}K", s),
            Card::Regular(s, 12) => write!(f, "{}Q", s),
            Card::Regular(s, 11) => write!(f, "{}J", s),
            Card::Regular(s, 10) => write!(f, "{}T", s),
            Card::Regular(s, n) => write!(f, "{}{}", s, n),
            Card::Joker(n) => write!(f, "J{}", n),
            Card::Special(s) => write!(f, "{}{}", s, if s == Suit::Spades { "J" } else { "Q" }),
        }
    }
}

impl Card {
    fn new(suit: Suit, num: u8) -> Self {
        assert!(1 <= num && num < 14);
        if num == 12 && suit == Suit::Diamonds {
            Card::Special(suit)
        } else if num == 11 && suit == Suit::Spades {
            Card::Special(suit)
        } else {
            Card::Regular(suit, num)
        }
    }
}

#[derive(Debug, Clone)]
struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    fn empty() -> Deck {
        Deck {
            cards: Vec::new(),
        }
    }

    fn new(jokers: u8) -> Deck {
        let mut res = Vec::new();
        for &suit in &[Suit::Spades, Suit::Hearts, Suit::Clubs, Suit::Diamonds] {
            for num in 1..14 {
                res.push(Card::new(suit, num));
            }
        }
        for i in 0..jokers {
            res.push(Card::Joker(i));
        }
        Deck {
            cards: res,
        }
    }

    fn shuffle<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        rng.shuffle(&mut self.cards);
    }

    fn push(&mut self, card: Card) {
        self.cards.push(card);
    }

    fn pop(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    fn take(&mut self, hand: &mut Hand) {
        for c in hand.cards.iter() {
            self.cards.push(*c);
        }
        hand.cards.clear();
    }
}

#[derive(Debug, Clone)]
struct Hand {
    cards: Vec<Card>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum WinCondition {
    FiveCards,
    TwentyFive,
    Special,
    Joker,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum HandSum {
    Win(WinCondition),
    NoWin(u8),
}

impl HandSum {
    fn no_win(&self) -> u8 {
        match *self {
            HandSum::NoWin(sum) => sum,
            _ => panic!("no_win() called on {:?}", self),
        }
    }
}

impl Hand {
    fn new() -> Self {
        Hand {
            cards: Vec::new(),
        }
    }

    fn can_accept(&self, card: Card) -> bool {
        let n = match card {
            Card::Regular(_, n) => n,
            Card::Joker(_) | Card::Special(_) => return true,
        };
        match self.hand_sum() {
            HandSum::Win(_) => panic!("can_accept() on winning hand"),
            HandSum::NoWin(sum) => sum + n <= 25
        }
    }

    fn accept(&mut self, card: Card) {
        assert!(self.can_accept(card));
        self.cards.push(card);
    }

    fn hand_sum(&self) -> HandSum {
        let mut sum = 0;
        let mut aces = 0;
        for c in self.cards.iter() {
            match c {
                Card::Regular(_, n) => {sum += n; if *n == 1 { aces += 1; }},
                Card::Joker(_) => return HandSum::Win(WinCondition::Joker),
                Card::Special(_) => return HandSum::Win(WinCondition::Special),
            }
        }
        if self.cards.len() == 5 {
            HandSum::Win(WinCondition::FiveCards)
        } else if sum == 25 || (sum == 12 && aces >= 1) {
            HandSum::Win(WinCondition::TwentyFive)
        } else {
            HandSum::NoWin(sum)
        }
    }
}

#[derive(Debug, Clone)]
struct Game {
    deck: Deck,
    discard: Deck,
    players: Vec<Hand>,
    round: usize,
}

#[derive(Debug, Clone)]
struct RoundResult {
    giver: usize,
    receiver: Option<usize>,
    card: Card,
    win: Option<WinCondition>,
}

impl RoundResult {
    fn describe(&self, g: &Game) -> String {
        match (self.receiver, self.win) {
            (Some(r), None) =>
                format!("{} {} to {} => {}",
                        self.giver, self.card, r, g.players[r].hand_sum().no_win()),
            (Some(r), Some(w)) =>
                format!("{} {} to {} => {:?}",
                        self.giver, self.card, r, w),
            (None, _) =>
                format!("{} {} to nobody", self.giver, self.card),
        }
    }
}

trait Strategy {
    fn choose(&mut self, giver: usize, hands: &Vec<Hand>, card: Card) -> usize;
}

impl Game {
    fn new(players: usize, jokers: u8) -> Self {
        let mut hands = Vec::new();
        hands.resize(players, Hand::new());
        Game {
            deck: Deck::new(jokers),
            discard: Deck::empty(),
            players: hands,
            round: 0,
        }
    }

    fn shuffle<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        self.deck.shuffle(rng);
    }

    fn pop_deck<R: Rng + ?Sized>(&mut self, rng: &mut R) -> Option<Card> {
        match self.deck.pop() {
            Some(c) => Some(c),
            None => {
                std::mem::swap(&mut self.deck, &mut self.discard);
                self.deck.shuffle(rng);
                self.deck.pop()
            }
        }
    }

    fn step<R: Rng + ?Sized, S: Strategy>(&mut self, rng: &mut R, strategy: &mut S)
            -> Option<RoundResult> {
        let card = match self.pop_deck(rng) {
            Some(c) => c,
            None => return None,
        };
        let is_red = match card {
            Card::Regular(s, _) => s.is_red(),
            Card::Joker(_) | Card::Special(_) => false,
        };
        let giver = self.round % self.players.len();
        let receiver = if is_red {
            let mut one = None;
            let mut n = 0;
            for (i, hand) in self.players.iter().enumerate() {
                if hand.can_accept(card) {
                    one = Some(i);
                    n += 1;
                }
            }
            match n {
                0 => None,
                1 => one,
                _ => {
                    let j = strategy.choose(giver, &self.players, card);
                    assert!(j < self.players.len());
                    assert!(self.players[j].can_accept(card));
                    Some(j)
                },
            }
        } else {
            let i = giver;
            if self.players[i].can_accept(card) {
                Some(i)
            } else {
                None
            }
        };
        let mut win = None;
        match receiver {
            Some(i) => match card {
                Card::Special(_) => {
                    win = Some(WinCondition::Special);
                    self.discard.push(card);
                },
                _ => {
                    self.players[i].accept(card);
                    if let HandSum::Win(cond) = self.players[i].hand_sum() {
                        self.discard.take(&mut self.players[i]);
                        win = Some(cond);
                    }
                }
            },
            None => self.discard.push(card),
        };
        self.round += 1;
        Some(RoundResult {
            giver: giver,
            receiver: receiver,
            card: card,
            win: win,
        })
    }
}

struct RandomStrategy {
    rng: rand::prng::XorShiftRng,
    tmp_players: Vec<usize>,
}

impl RandomStrategy {
    fn new() -> Self {
        RandomStrategy {
            rng: rand::prng::XorShiftRng::from_seed([60; 16]),
            tmp_players: Vec::new(),
        }
    }
}

impl Strategy for RandomStrategy {
    fn choose(&mut self, _giver: usize, hands: &Vec<Hand>, card: Card) -> usize {
        self.tmp_players.clear();
        for (i, hand) in hands.iter().enumerate() {
            if hand.can_accept(card) {
                self.tmp_players.push(i);
            }
        }
        *self.rng.choose(&self.tmp_players).unwrap()
    }
}

fn main() {
    let seed = 42;
    let players = 5;
    let jokers = 3;
    let mut strategy = RandomStrategy::new();

    let mut g = Game::new(players, jokers);
    let mut rng = rand::prng::XorShiftRng::from_seed([seed; 16]);
    g.shuffle(&mut rng);
    for _ in 0..1000 {
        let result = g.step(&mut rng, &mut strategy).expect("We're out of cards!");
        println!("{}", result.describe(&g));
    }
}
