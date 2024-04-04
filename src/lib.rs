use csta::prelude::*;
use csta_derive::Randomizable;
use rand::prelude::*;

const L: usize = 4;

pub mod sim;

#[derive(Clone, Debug, Randomizable)]
pub enum Move {
    Up,
    Right,
    Down,
    Left,
}

#[derive(Debug, Default, Clone, Randomizable)]
pub struct Tile {
    #[rng(default)]
    pub exp: isize,
    #[rng(default)]
    pub is_merged: bool,
}

impl Eq for Tile {}

impl PartialEq for Tile {
    fn eq(&self, other: &Self) -> bool {
        self.exp.eq(&other.exp)
    }
}

impl Ord for Tile {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.exp.cmp(&other.exp)
    }
}

impl PartialOrd for Tile {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[macro_export]
macro_rules! avance {
    ($self:tt, $from:tt, $to:tt) => {
        if ($self.grid[$to].exp == $self.grid[$from].exp
            && (!$self.grid[$to].is_merged && !$self.grid[$from].is_merged))
        {
            $self.grid[$to].exp += 1;
            $self.grid[$from].exp = 0;
            $self.grid[$to].is_merged = true;
            $self.has_moved = true;
        } else if ($self.grid[$to].exp == 0) {
            $self.grid[$to].exp = $self.grid[$from].exp;
            $self.grid[$to].is_merged = $self.grid[$from].is_merged;
            $self.grid[$from].exp = 0;
            $self.grid[$from].is_merged = false;
            $self.has_moved = true;
        } else {
            break;
        }
    };
}

#[macro_export]
macro_rules! create_game {
    ( $( $i:tt )+ ) => {
        C2048 {
            grid: [ $( $i, )+ ].map(|n| Tile {
                exp: n,
                is_merged: false,
            }),
            has_moved: false,
        }
    };
}

#[derive(Debug, Clone, Default)]
pub struct C2048 {
    pub grid: [Tile; L * L],
    pub has_moved: bool,
    pub rng: ThreadRng,
}

impl Randomizable for C2048 {
    fn sample<D: Distribution<f64>, R: Rng + ?Sized>(_: &D, _: &mut R) -> Self {
        C2048::new()
    }
}

impl C2048 {
    pub fn new() -> Self {
        let mut c2048 = Self::default();
        c2048.spawn_tile(0.0);
        c2048.spawn_tile(0.0);
        c2048
    }

    pub fn spawn_tile(&mut self, chance: f64) {
        let random_exp = if self.rng.gen_bool(chance) { 2 } else { 1 };

        let random_tile = self
            .grid
            .iter_mut()
            .filter(|tile| tile.exp == 0)
            .choose(&mut self.rng);
        if let Some(tile) = random_tile {
            tile.exp = random_exp;
        }
    }

    pub fn highest(&self) -> &Tile {
        self.grid.iter().max().unwrap()
    }

    pub fn score(&self) -> usize {
        self.grid.iter().map(|t| 1 << t.exp).sum()
    }

    pub fn is_lose(&self) -> bool {
        if self.grid.iter().any(|tile| tile.exp == 0) {
            return false;
        }

        for x in 0..L - 1 {
            for y in 0..L - 1 {
                let i = x + y * L;
                if self.grid[i] == self.grid[i + 1] || self.grid[i] == self.grid[i + L] {
                    return false;
                }
                let i = L - 1 + y * L;
                if self.grid[i] == self.grid[i + L] {
                    return false;
                }
                let i = x + (L - 1) * L;
                if self.grid[i] == self.grid[i + 1] {
                    return false;
                }
            }
        }
        true
    }

    pub fn energy(&self) -> isize {
        let mut e = 0;
        for x in 0..L {
            for y in 0..L {
                let i = x + y * L;
                let exp = self.grid[i].exp;

                if exp == 0 {
                    e -= 1;
                    continue;
                } else {
                    e += exp;
                }

                // we get right, left, up and down.
                let right = if x + 1 < L {
                    Some(&self.grid[i + 1])
                } else {
                    None
                };
                let left = if x > 0 { Some(&self.grid[i - 1]) } else { None };
                let up = if y + 1 < L {
                    Some(&self.grid[i + L])
                } else {
                    None
                };
                let down = if y > 0 { Some(&self.grid[i - L]) } else { None };

                let xaxis = [up, down];
                let yaxis = [left, right];
                let directions = [xaxis, yaxis];

                for other in directions.iter().flatten().flatten() {
                    if other.exp == exp {
                        e -= exp;
                    } else {
                        e += exp;
                    }
                }

                for axis in directions {
                    if let [Some(j), Some(k)] = axis {
                        let j = j.exp;
                        let k = k.exp;

                        if (j == exp + 1 && k == exp - 1) || (j == exp - 1 && k == exp + 1) {
                            e -= exp;
                        } else {
                            e += exp;
                        }
                    }
                }
            }
        }
        e
    }

    pub fn reset(&mut self) {
        for tile in self.grid.iter_mut() {
            tile.is_merged = false;
        }
        self.has_moved = false;
    }

    pub fn clone_move(&self, mv: Move) -> Self {
        let mut clone = self.clone();
        match mv {
            Move::Up => clone.up(),
            Move::Right => clone.right(),
            Move::Down => clone.down(),
            Move::Left => clone.left(),
        }
        clone
    }

    pub fn left(&mut self) {
        for y in 0..L {
            for x in 1..L {
                let i = x + y * L;
                if self.grid[i].exp == 0 {
                    continue;
                }

                for c in 0..x {
                    let from = i - c;
                    let to = i - c - 1;
                    avance!(self, from, to);
                }
            }
        }
    }

    pub fn right(&mut self) {
        for y in 0..L {
            for x in (0..L - 1).rev() {
                let i = x + y * L;
                if self.grid[i].exp == 0 {
                    continue;
                }

                for c in 0..=2 - x {
                    let from = i + c;
                    let to = i + c + 1;
                    avance!(self, from, to);
                }
            }
        }
    }

    pub fn up(&mut self) {
        for x in 0..L {
            for y in (0..L - 1).rev() {
                let i = x + y * L;
                if self.grid[i].exp == 0 {
                    continue;
                }

                for c in 0..=2 - y {
                    let from = i + c * L;
                    let to = i + (c + 1) * L;
                    avance!(self, from, to);
                }
            }
        }
    }

    pub fn down(&mut self) {
        for x in 0..L {
            for y in 1..L {
                let i = x + y * L;
                if self.grid[i].exp == 0 {
                    continue;
                }

                for c in 0..y {
                    let from = i - c * L;
                    let to = i - (c + 1) * L;
                    avance!(self, from, to);
                }
            }
        }
    }
}

impl PartialEq for C2048 {
    fn eq(&self, other: &Self) -> bool {
        self.energy().eq(&other.energy())
    }
}

impl PartialOrd for C2048 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for C2048 {}

impl Ord for C2048 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.energy().cmp(&other.energy())
    }
}
