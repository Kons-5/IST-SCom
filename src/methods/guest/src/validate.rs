#[derive(Clone, Copy, PartialEq, Eq)]
enum Cell {
    Empty,
    Ship,
    Visited,
}

type Board = [[Cell; 10]; 10];

#[allow(dead_code)]
enum ShipType {
    Carrier,    // size 5
    Battleship, // size 4
    Destroyer,  // size 3
    Cruiser,    // size 2 (x2)
    Submarine,  // size 1 (x2)
}

impl ShipType {
    fn fleet() -> Vec<usize> {
        vec![5, 4, 3, 2, 2, 1, 1]
    }
}

pub fn validate_battleship_board(positions: &[u8]) -> bool {
    if positions.len() != 18 {
        return false;
    }

    let mut board = [[Cell::Empty; 10]; 10];
    for &pos in positions {
        if pos >= 100 {
            return false;
        }
        let x = (pos % 10) as usize;
        let y = (pos / 10) as usize;
        if board[y][x] != Cell::Empty {
            return false; // duplicate
        }
        board[y][x] = Cell::Ship;
    }

    let mut sizes_found = Vec::new();

    for y in 0..10 {
        for x in 0..10 {
            if board[y][x] == Cell::Ship {
                let size = explore_ship(&mut board, x, y);
                if size == 0 {
                    return false;
                }
                sizes_found.push(size);
            }
        }
    }

    sizes_found.sort_unstable();
    let mut expected = ShipType::fleet();
    expected.sort_unstable();

    sizes_found == expected
}

fn explore_ship(board: &mut Board, x: usize, y: usize) -> usize {
    let mut length = 0;

    let mut is_horizontal = false;
    let mut is_vertical = false;

    if x + 1 < 10 && board[y][x + 1] == Cell::Ship {
        is_horizontal = true;
    }
    if y + 1 < 10 && board[y + 1][x] == Cell::Ship {
        is_vertical = true;
    }

    if is_horizontal && is_vertical {
        return 0; // invalid L-shape
    }

    if is_horizontal {
        let mut j = x;
        while j < 10 && board[y][j] == Cell::Ship {
            board[y][j] = Cell::Visited;
            length += 1;

            // Check for vertical adjacency
            if y > 0 && board[y - 1][j] == Cell::Ship {
                return 0;
            }
            if y + 1 < 10 && board[y + 1][j] == Cell::Ship {
                return 0;
            }

            j += 1;
        }
    } else if is_vertical {
        let mut i = y;
        while i < 10 && board[i][x] == Cell::Ship {
            board[i][x] = Cell::Visited;
            length += 1;

            // Check for horizontal adjacency
            if x > 0 && board[i][x - 1] == Cell::Ship {
                return 0;
            }
            if x + 1 < 10 && board[i][x + 1] == Cell::Ship {
                return 0;
            }

            i += 1;
        }
    } else {
        board[y][x] = Cell::Visited;
        length = 1;

        // Check surroundings
        let neighbors = [
            (x.wrapping_sub(1), y),
            (x + 1, y),
            (x, y.wrapping_sub(1)),
            (x, y + 1),
        ];

        for (nx, ny) in neighbors {
            if nx < 10 && ny < 10 && board[ny][nx] == Cell::Ship {
                return 0;
            }
        }
    }

    length
}
