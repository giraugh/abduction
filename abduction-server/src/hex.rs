use std::{fmt, str::FromStr};

use anyhow::anyhow;
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Store an `(q, r)` value for a hex location
///
/// see https://www.redblobgames.com/grids/hexagons
#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    PartialEq,
    Eq,
    Hash,
    derive_more::Add,
    derive_more::AddAssign,
    derive_more::Sub,
    derive_more::SubAssign,
    derive_more::From,
    derive_more::Into,
)]
#[qubit::ts]
pub struct AxialHex(isize, isize);

impl fmt::Display for AxialHex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{}", self.0, self.1)
    }
}

impl FromStr for AxialHex {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (x, y) = s
            .split_once(",")
            .ok_or(anyhow!("No comma delimeter in hex"))?;
        Ok(Self(x.parse()?, y.parse()?))
    }
}

impl AxialHex {
    pub const ZERO: AxialHex = AxialHex(0, 0);
    pub const EAST: AxialHex = AxialHex(1, 0);
    pub const WEST: AxialHex = AxialHex(-1, 0);
    pub const NORTH_EAST: AxialHex = AxialHex(1, -1);
    pub const NORTH_WEST: AxialHex = AxialHex(0, -1);
    pub const SOUTH_EAST: AxialHex = AxialHex(0, 1);
    pub const SOUTH_WEST: AxialHex = AxialHex(-1, 1);

    pub fn all_in_bounds(radius: isize) -> Vec<Self> {
        let mut result = Vec::new();
        for q in -radius..=radius {
            for r in -radius..=radius {
                let s = -q - r;
                if q.abs().max(r.abs()).max(s.abs()) <= radius {
                    result.push(Self(q, r));
                }
            }
        }
        result
    }

    /// Determine if a given hex is adjacent to this hex
    pub fn is_adjacent(&self, other: AxialHex) -> bool {
        self.neighbours().contains(&other)
    }

    /// Return all neighbouring hexes
    pub fn neighbours(&self) -> [AxialHex; 6] {
        let AxialHex(q, r) = *self;
        [
            AxialHex(q + 1, r),
            AxialHex(q + 1, r - 1),
            AxialHex(q, r - 1),
            AxialHex(q - 1, r),
            AxialHex(q - 1, r + 1),
            AxialHex(q, r + 1),
        ]
    }

    pub fn random_in_bounds(rng: &mut impl Rng, radius: isize) -> Self {
        let x = (rng.random_range(0..=2 * (radius as usize)) as isize) - radius;
        let min_y = isize::max(-radius, -x - radius);
        let max_y = isize::min(radius, -x + radius);
        let y = (rng.random_range(0..=(max_y - min_y) as usize) as isize) + min_y;
        let z = -x - y;

        Self(x, z)
    }

    /// Get a `(q, r, s)` cube coordinate by deriving the `s` value
    pub fn as_cube_coordinate(&self) -> (isize, isize, isize) {
        (self.0, self.1, -self.0 - self.1)
    }

    pub fn dist_to_origin(&self) -> isize {
        let (q, r, s) = self.as_cube_coordinate();
        (q.abs() + r.abs() + s.abs()) / 2 // TODO: do we lose too much accuracy here?
    }

    pub fn dist_to(&self, other: Self) -> isize {
        let delta = other - *self;
        let dq = delta.0.abs();
        let dr = delta.1.abs();
        (dq + dr + (dq + dr).abs()) / 2
    }

    pub fn within_bounds(&self, radius: isize) -> bool {
        self.dist_to_origin() <= radius
    }
}

/// Direction you can move on a hex grid
/// This makes a few assumptions about the grid
///  - Pointy topped hexagons
///  - Odd rows are shunted right
#[derive(Debug, Clone, Serialize, Copy)]
#[qubit::ts]
#[serde(rename_all = "snake_case")]
pub enum AxialHexDirection {
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

impl AxialHexDirection {
    /// NOTE: right now this only works with adjacent hexs and returns None in other cases
    pub fn direction_to(from: AxialHex, to: AxialHex) -> Option<Self> {
        let delta = to - from;

        match delta {
            // Same hex
            AxialHex::ZERO => None,

            // Each direction
            AxialHex::EAST => Some(AxialHexDirection::East),
            AxialHex::WEST => Some(AxialHexDirection::West),
            AxialHex::NORTH_EAST => Some(AxialHexDirection::NorthEast),
            AxialHex::NORTH_WEST => Some(AxialHexDirection::NorthWest),
            AxialHex::SOUTH_EAST => Some(AxialHexDirection::SouthEast),
            AxialHex::SOUTH_WEST => Some(AxialHexDirection::SouthWest),

            // Non-Adjacent
            _ => None,
        }
    }
}

impl From<AxialHexDirection> for AxialHex {
    fn from(value: AxialHexDirection) -> Self {
        match value {
            AxialHexDirection::East => AxialHex::EAST,
            AxialHexDirection::West => AxialHex::WEST,
            AxialHexDirection::NorthEast => AxialHex::NORTH_EAST,
            AxialHexDirection::NorthWest => AxialHex::NORTH_WEST,
            AxialHexDirection::SouthEast => AxialHex::SOUTH_EAST,
            AxialHexDirection::SouthWest => AxialHex::SOUTH_WEST,
        }
    }
}
