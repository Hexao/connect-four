#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Player {
    Red, Yellow,
}

#[derive(Clone, Copy)]
enum Memory {
    RedRed, RedYellow,
    YellowRed, YellowYellow,
}

#[derive(Clone, Copy)]
pub struct Game {
    player_turn: Memory,
    grid: [Option<Player>; Self::ROW * Self::COL],
}

impl Game {
    pub const COL: usize = 7;
    pub const ROW: usize = 6;

    pub fn play_col(&mut self, col: usize) -> bool {
        let height = self.col_filled(col);
        if height == Self::ROW { return false; }

        let index = col * Self::ROW;
        self.grid[index + height] = Some(self.player_turn.into());

        self.player_turn = match self.player_turn {
            Memory::RedRed => Memory::RedYellow,
            Memory::RedYellow => Memory::RedRed,
            Memory::YellowRed => Memory::YellowYellow,
            Memory::YellowYellow => Memory::YellowRed,
        };

        true
    }

    pub fn col_filled(&self, col: usize) -> usize {
        let index = col as usize * Self::ROW;
        let chunk = &self.grid[index..index + Self::ROW];

        chunk.iter().take_while(|&&cell| cell != None).count()
    }

    pub fn grid(&self) -> [Option<Player>; Self::ROW * Self::COL] {
        self.grid
    }

    pub fn player_turn(&self) -> Player {
        self.player_turn.into()
    }
}

impl Default for Game {
    fn default() -> Self {
        Self {
            player_turn: Memory::RedRed,
            grid: [None; Self::COL * Self::ROW],
        }
    }
}

impl Into<Player> for Memory {
    fn into(self) -> Player {
        match self {
            Memory::RedYellow | Memory::YellowYellow => Player::Yellow,
            Memory::RedRed | Memory::YellowRed => Player::Red,
        }
    }
}
