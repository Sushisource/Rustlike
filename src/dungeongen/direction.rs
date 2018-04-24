use std::slice::Iter;

#[derive(PartialEq, Debug, Clone, Copy, Eq, Hash)]
pub enum Direction {
  North,
  NorthEast,
  East,
  SouthEast,
  South,
  SouthWest,
  West,
  NorthWest,
}

impl Direction {
  pub fn iterator() -> Iter<'static, Direction> {
    static DIRECTIONS: [Direction; 8] = [
      Direction::North,
      Direction::NorthEast,
      Direction::East,
      Direction::SouthEast,
      Direction::South,
      Direction::SouthWest,
      Direction::West,
      Direction::NorthWest,
    ];
    DIRECTIONS.into_iter()
  }

  pub fn compass() -> &'static [Direction; 4] {
    static DIRECTIONS: [Direction; 4] = [
      Direction::North,
      Direction::East,
      Direction::South,
      Direction::West,
    ];
    &DIRECTIONS
  }

  pub fn to_tup(&self) -> (f32, f32) {
    match *self {
      Direction::North => (0.0, -1.0),
      Direction::NorthEast => (1.0, -1.0),
      Direction::East => (1.0, 0.0),
      Direction::SouthEast => (1.0, 1.0),
      Direction::South => (0.0, 1.0),
      Direction::SouthWest => (-1.0, 1.0),
      Direction::West => (-1.0, 0.0),
      Direction::NorthWest => (-1.0, -1.0),
    }
  }

  pub fn dir_from_tup(&self, other: (i32, i32)) -> (i32, i32) {
    let modifier = self.to_tup();
    (other.0 + modifier.0 as i32, other.1 + modifier.1 as i32)
  }

  pub fn opposite(&self) -> Direction {
    match *self {
      Direction::North => Direction::South,
      Direction::NorthEast => Direction::SouthWest,
      Direction::East => Direction::West,
      Direction::SouthEast => Direction::NorthWest,
      Direction::South => Direction::North,
      Direction::SouthWest => Direction::NorthEast,
      Direction::West => Direction::East,
      Direction::NorthWest => Direction::SouthEast,
    }
  }
}
