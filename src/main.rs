use std::ops::Not;

use decider::{Evaluation, Mode, State, cache, choose};

#[derive(Clone, Eq, Hash, PartialEq, Debug, Copy)]
pub enum Player {
    X,
    O,
}

impl Not for Player {
    type Output = Player;
    fn not(self) -> Self::Output {
        match self {
            Player::X => Player::O,
            Player::O => Player::X,
        }
    }
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Copy)]
pub enum Field {
    Player(Player),
    Empty,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Copy)]
pub struct Board {
    fields: [[Field; 3]; 3],
    player: Player,
}

impl Board {
    pub fn new() -> Self {
        Self {
            fields: [[Field::Empty; 3]; 3],
            player: Player::X,
        }
    }

    pub fn set(&mut self, x: usize, y: usize, field: Field) {
        self.fields[x][y] = field;
    }

    pub fn get(&self, x: usize, y: usize) -> Field {
        self.fields[x][y]
    }

    pub fn is_full(&self) -> bool {
        self.fields
            .iter()
            .all(|row| row.iter().all(|field| *field != Field::Empty))
    }

    pub fn is_winner(&self, player: Player) -> bool {
        let wins = [
            [(0, 0), (0, 1), (0, 2)],
            [(1, 0), (1, 1), (1, 2)],
            [(2, 0), (2, 1), (2, 2)],
            [(0, 0), (1, 0), (2, 0)],
            [(0, 1), (1, 1), (2, 1)],
            [(0, 2), (1, 2), (2, 2)],
            [(0, 0), (1, 1), (2, 2)],
            [(0, 2), (1, 1), (2, 0)],
        ];
        wins.into_iter().any(|win| {
            win.iter()
                .all(|&(x, y)| self.get(x, y) == Field::Player(player))
        })
    }

    pub fn winner(&self) -> Option<Player> {
        if self.is_winner(Player::X) {
            Some(Player::X)
        } else if self.is_winner(Player::O) {
            Some(Player::O)
        } else {
            None
        }
    }
}

impl State for Board {
    type Decision = (usize, usize);
    fn decisions(&self) -> impl Iterator<Item = Self::Decision> {
        self.fields
            .iter()
            .enumerate()
            .flat_map(|(x, row)| row.iter().enumerate().map(move |(y, _)| (x, y)))
            .filter(move |&(x, y)| self.get(x, y) == Field::Empty)
    }

    fn choose(&self, decision: Self::Decision) -> Self {
        let mut new_board = self.clone();
        new_board.set(decision.0, decision.1, Field::Player(self.player));
        new_board.player = !self.player;
        new_board
    }
}

pub fn eval(player: Player) -> impl FnMut(Board) -> Option<((usize, usize), f64)> {
    choose(
        move |board: &Board| {
            if board.is_winner(player) {
                Evaluation::Value(100.0)
            } else if board.is_winner(!player) {
                Evaluation::Value(-100.0)
            } else if board.is_full() {
                Evaluation::Value(0.0)
            } else if board.player == player {
                Evaluation::Mode(Mode::Maximize)
            } else {
                Evaluation::Mode(Mode::Minimize)
            }
        },
        0.99,
    )
}

pub fn display(board: Board) -> String {
    let mut result = String::new();
    for row in board.fields.iter() {
        for field in row.iter() {
            result.push(match field {
                Field::Player(Player::X) => 'X',
                Field::Player(Player::O) => 'O',
                Field::Empty => ' ',
            });
        }
        result.push('\n');
    }
    result
}

fn main() {
    let mut board = Board::new();
    let mut player_x = eval(Player::X);
    let mut player_o = eval(Player::O);
    loop {
        println!("{}", display(board));
        if let Some(winner) = board.winner() {
            println!(
                "{} wins!",
                match winner {
                    Player::X => 'X',
                    Player::O => 'O',
                }
            );
            break;
        }
        if board.is_full() {
            println!("It's a draw!");
            break;
        }
        let (decision, value) = if board.player == Player::X {
            player_x(board.clone()).unwrap()
        } else {
            player_o(board.clone()).unwrap()
        };
        println!(
            "{} chooses {:?} with value {}",
            match board.player {
                Player::X => 'X',
                Player::O => 'O',
            },
            decision,
            value
        );
        board = board.choose(decision);
    }
}
