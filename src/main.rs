use std::{
    collections::HashSet,
    fmt::{Display, Formatter},
    num::ParseIntError,
    str::FromStr,
    time::Instant,
};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct Coord {
    row: u8,
    col: u8,
}

impl Coord {
    fn next(&self) -> Option<Self> {
        let next_col = if self.col < 8 { self.col + 1 } else { 0 };
        let next_row = if next_col == 0 {
            self.row + 1
        } else {
            self.row
        };
        if next_row < 9 {
            Some(Coord {
                row: next_row,
                col: next_col,
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
enum InvalidSudokuError {
    Unsolvable,
    InvalidRow(u8),
    InvalidCol(u8),
    InvalidHouse(Coord),
}

#[derive(Debug)]
enum ParseSudokuError {
    ParseInt(ParseIntError),
    InvalidSize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Sudoku {
    grid: Vec<Vec<Option<u8>>>,
}

impl FromStr for Sudoku {
    type Err = ParseSudokuError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn parse_row(line: &str) -> Result<Vec<Option<u8>>, ParseIntError> {
            line.chars()
                .map(|c| -> Result<Option<u8>, ParseIntError> {
                    Ok(match c {
                        '.' => None,
                        _ => Some(c.to_string().parse::<u8>()?),
                    })
                })
                .collect()
        }

        let grid = s
            .lines()
            .map(|l| parse_row(l).map_err(ParseSudokuError::ParseInt))
            .collect::<Result<Vec<Vec<Option<u8>>>, Self::Err>>()?;

        if grid.len() != 9 || grid[0].len() != 9 {
            return Err(ParseSudokuError::InvalidSize);
        }

        Ok(Sudoku { grid })
    }
}

impl Display for Sudoku {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for row in &self.grid {
            for cell in row {
                match cell {
                    None => write!(f, "."),
                    Some(n) => write!(f, "{n}"),
                }?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

impl Sudoku {
    /// Returns a solved sudoku based on the current state, or an error indicating unsolvable.
    fn solve(&self) -> Result<Self, InvalidSudokuError> {
        let mut sudoku = self.clone();

        sudoku.solve_rec(Coord { row: 0, col: 0 });

        match sudoku.validate() {
            Ok(_) => Ok(sudoku),
            Err(_) => Err(InvalidSudokuError::Unsolvable),
        }
    }

    fn solve_rec(&mut self, current_coord: Coord) -> bool {
        // The method of this is to try each of the possible numbers and continue on.
        // If there are no possible numbers, then we've hit a dead-end and return up the stack.

        // First check if there's a next coord
        let Some(next_coord) = current_coord.next() else {
            // If none after this, fill the last cell with what we have and return out.
            if let Some(n) = self.get_possible_numbers(current_coord).into_iter().next() {
                self.set(current_coord, n)
            }
            return true;
        };

        // Check if it's already populated
        if self.get(current_coord).is_some() {
            // Skip and continue on
            return self.solve_rec(next_coord);
        }

        // Loop through each of the possible numbers, trying it and continuing to the next cell.
        for n in self.get_possible_numbers(current_coord) {
            self.set(current_coord, n);

            // If this is returning true, that means we found our solution, keep returning up.
            if self.solve_rec(next_coord) {
                return true;
            }
        }

        // Solution not found, unset this cell and return false,
        // trying another possible number further up the chain.
        self.unset(current_coord);
        false
    }

    /// Validates the current state of the sudoku.
    fn validate(&self) -> Result<(), HashSet<InvalidSudokuError>> {
        let mut errors = HashSet::new();

        for n in 0..9 {
            if self.get_row(n).into_iter().sum::<u8>() != 45 {
                errors.insert(InvalidSudokuError::InvalidRow(n));
            }

            if self.get_col(n).into_iter().sum::<u8>() != 45 {
                errors.insert(InvalidSudokuError::InvalidCol(n));
            }

            let house_coord = Coord {
                row: n / 3,
                col: n % 3,
            };
            if self.get_house(house_coord).iter().sum::<u8>() != 45 {
                errors.insert(InvalidSudokuError::InvalidHouse(house_coord));
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(())
    }

    /// Gets the cell at the coord
    fn get(&self, coord: Coord) -> Option<u8> {
        self.grid[coord.row as usize][coord.col as usize]
    }

    /// Sets the cell to Some(value)
    fn set(&mut self, coord: Coord, value: u8) {
        self.grid[coord.row as usize][coord.col as usize] = Some(value);
    }

    /// Sets the cell to None
    fn unset(&mut self, coord: Coord) {
        self.grid[coord.row as usize][coord.col as usize] = None;
    }

    /// Gets all possible numbers at the given coordinate.
    fn get_possible_numbers(&self, coord: Coord) -> HashSet<u8> {
        // Get each set of numbers from row, col, and house.
        let row = self.get_row(coord.row);
        let col = self.get_col(coord.col);
        let house = self.get_house(Coord {
            row: coord.row / 3,
            col: coord.col / 3,
        });

        // Hashsets are pretty neat. Generate 1-9 hashset, and remove matching numbers.
        &(&(&(1..=9).collect() - &row) - &col) - &house
    }

    /// Gets all present numbers in a row.
    fn get_row(&self, index: u8) -> HashSet<u8> {
        self.grid[index as usize]
            .iter()
            .filter_map(|&n| n)
            .collect()
    }

    /// Gets all present numbers in a col.
    fn get_col(&self, index: u8) -> HashSet<u8> {
        self.grid
            .iter()
            .filter_map(|row| row[index as usize])
            .collect()
    }

    /// Gets all present numbers in the house at coord. Note this is a house coordinate,
    /// So Coord { row: 2, col: 1 } would return the bottom-middle house.
    fn get_house(&self, coord: Coord) -> HashSet<u8> {
        let mut house = HashSet::new();

        let row_start = coord.row * 3;
        let row_end = coord.row * 3 + 3;
        let col_start = coord.col * 3;
        let col_end = coord.col * 3 + 3;

        for row in row_start..row_end {
            for col in col_start..col_end {
                if let Some(n) = self.grid[row as usize][col as usize] {
                    house.insert(n);
                }
            }
        }

        house
    }
}

fn main() {
    let start_time = Instant::now();

    let sudoku = match include_str!("input.txt").parse::<Sudoku>() {
        Ok(result) => result,
        Err(error) => {
            println!("Error encountered while parsing: {error:?}");
            return;
        }
    };

    let solved_sudoku = match sudoku.solve() {
        Ok(result) => result,
        Err(error) => {
            println!("Error encounted while solving: {error:?}");
            return;
        }
    };

    println!("{solved_sudoku}");

    let end_time = Instant::now();
    let duration = end_time.duration_since(start_time);
    println!("Solved in {} milliseconds", duration.as_millis());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_easy() {
        let sudoku = include_str!("easy.txt").parse::<Sudoku>().unwrap();
        let result = sudoku.solve().unwrap();
        let expected = include_str!("easy_solved.txt").parse::<Sudoku>().unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn test_get_house() {
        fn house(input: &str) -> HashSet<u8> {
            input
                .chars()
                .map(|c| c.to_string().parse().unwrap())
                .collect()
        }

        let sudoku = include_str!("easy_solved.txt").parse::<Sudoku>().unwrap();
        let result = sudoku.get_house(Coord { row: 0, col: 0 });
        assert_eq!(house("894235167"), result);
        let result = sudoku.get_house(Coord { row: 0, col: 1 });
        assert_eq!(house("137468592"), result);
        let result = sudoku.get_house(Coord { row: 2, col: 2 });
        assert_eq!(house("947153682"), result);
    }

    #[test]
    fn test_validate() {
        let mut sudoku = include_str!("easy_solved.txt").parse::<Sudoku>().unwrap();
        assert!(sudoku.validate().is_ok());

        sudoku.grid[4][6] = Some(9);
        let expected: HashSet<InvalidSudokuError> = vec![
            InvalidSudokuError::InvalidRow(4),
            InvalidSudokuError::InvalidCol(6),
            InvalidSudokuError::InvalidHouse(Coord { row: 1, col: 2 }),
        ]
        .into_iter()
        .collect();
        assert_eq!(expected, sudoku.validate().unwrap_err())
    }

    #[test]
    fn test_get_possible_numbers() {
        let sudoku = include_str!("easy.txt").parse::<Sudoku>().unwrap();
        let result = sudoku.get_possible_numbers(Coord { row: 0, col: 0 });
        let expected: HashSet<u8> = vec![1, 2, 3, 4, 8].into_iter().collect();
        assert_eq!(expected, result);
        let result = sudoku.get_possible_numbers(Coord { row: 8, col: 0 });
        let expected: HashSet<u8> = vec![3, 4].into_iter().collect();
        assert_eq!(expected, result);
        let result = sudoku.get_possible_numbers(Coord { row: 8, col: 8 });
        let expected: HashSet<u8> = vec![9].into_iter().collect();
        assert_eq!(expected, result);
    }

    #[test]
    fn test_next_coord() {
        assert_eq!(
            Some(Coord { row: 0, col: 1 }),
            (Coord { row: 0, col: 0 }).next()
        );
        assert_eq!(
            Some(Coord { row: 1, col: 0 }),
            (Coord { row: 0, col: 8 }).next()
        );
        assert_eq!(
            Some(Coord { row: 8, col: 0 }),
            (Coord { row: 7, col: 8 }).next()
        );
        assert_eq!(None, (Coord { row: 8, col: 8 }).next());
    }
}
