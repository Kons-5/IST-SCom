/*
#[derive(Clone, Copy, PartialEq, Eq)]
enum Cell {
    Empty,
    Ship,
    Visited,
}

type Board = [[Cell; 10]; 10];

#[derive(Debug)]
enum ShipType {
    Carrier,     // size 5
    Battleship,  // size 4
    Destroyer,   // size 3
    Cruiser,     // size 2 (x2)
    Submarine,   // size 1 (x2)
}

impl ShipType {
    fn fleet() -> Vec<usize> {
        vec![5, 4, 3, 2, 2, 1, 1]
    }
}
*/

/*  Convert Vec<u8> to 10x10 board and validate it (Check with Prof)
pub fn validate_battleship_board(flat_board: &[u8]) -> bool {
    let mut array_board = [[0u8; 10]; 10];
    for i in 0..10 {
        for j in 0..10 {
            array_board[i][j] = flat_board[i * 10 + j];
        }
    }

}
*/
