#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Player {
    Red, Yellow,
}

pub enum PlayResult {
    Win([u8; 4]),
    Error, Pass,
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

    pub fn play_col(&mut self, col: usize) -> PlayResult {
        let height = self.col_height(col);
        if height == Self::ROW { return PlayResult::Error; }

        let index = col * Self::ROW;
        self.grid[index + height] = Some(self.player_turn.into());
        let connect = self.connected(col as i8, height as i8);

        self.player_turn = match self.player_turn {
            Memory::RedRed => Memory::RedYellow,
            Memory::RedYellow => Memory::RedRed,
            Memory::YellowRed => Memory::YellowYellow,
            Memory::YellowYellow => Memory::YellowRed,
        };

        connect
    }

    fn connected(&self, col: i8, row: i8) -> PlayResult {
        const DIRS: [(i8, i8); 4] = [(0, -1), (1, 1), (1, 0), (1, -1)];
        let target = Some(self.player_turn.into());

        for (x, y) in DIRS {
            let (mut forward, mut backward) = (0, 0);

            loop {
                if backward == 3 { break; }
                backward += 1;

                let actual_col = col - x * backward;
                let actual_row = row - y * backward;

                if !(0..Self::COL as i8).contains(&actual_col) ||
                   !(0..Self::ROW as i8).contains(&actual_row) ||
                   self.grid[actual_col as usize * Self::ROW + actual_row as usize] != target
                {
                    backward -= 1;
                    break;
                }
            }

            loop {
                if forward == 3 - backward { break; }
                forward += 1;

                let actual_col = col + x * forward;
                let actual_row = row + y * forward;

                if !(0..Self::COL as i8).contains(&actual_col) ||
                   !(0..Self::ROW as i8).contains(&actual_row) ||
                   self.grid[actual_col as usize * Self::ROW + actual_row as usize] != target
                {
                    forward -= 1;
                    break;
                }
            }

            if forward + backward == 3 {
                return PlayResult::Win([
                    (col - x * backward) as u8,
                    (row - y * backward) as u8,
                    (col + x * forward) as u8,
                    (row + y * forward) as u8,
                ]);
            }
        }

        PlayResult::Pass
    }

    pub fn col_height(&self, col: usize) -> usize {
        let index = col as usize * Self::ROW;
        let chunk = &self.grid[index..index + Self::ROW];

        chunk.iter().take_while(|cell| cell.is_some()).count()
    }

    pub fn col_full(&self, col: usize) -> bool {
        self.col_height(col) == Self::ROW
    }

    pub fn grid(&self) -> [Option<Player>; Self::ROW * Self::COL] {
        self.grid
    }

    pub fn player_turn(&self) -> Player {
        self.player_turn.into()
    }

    pub fn restart(&mut self) {
        self.grid = [None; Self::COL * Self::ROW];
        self.player_turn = match self.player_turn {
            Memory::RedRed | Memory::RedYellow => Memory::YellowYellow,
            Memory::YellowRed | Memory::YellowYellow => Memory::RedRed,
        }
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

impl From<Memory> for Player {
    fn from(memory: Memory) -> Self {
        match memory {
            Memory::RedYellow | Memory::YellowYellow => Player::Yellow,
            Memory::RedRed | Memory::YellowRed => Player::Red,
        }
    }
}
