use std::path::Display;

use wasm_bindgen::prelude::wasm_bindgen;

use crate::{board::{Board, TetBoard}, kicks::{get_180_kicks, get_kicks}, piece::{piece_color_from_int, Direction, PieceColor, TetPiece}, vec2::Vec2};
#[wasm_bindgen]
pub struct u64_board(u64);
impl u64_board {
    pub fn new() -> u64_board {
        u64_board(0)
    }
    pub fn in_4h_bounds(&self, position: Vec2) -> bool {
        position.0 >= 0 && position.0 < 10 && position.1 >= 0 && position.1 < 4
    }
    pub fn in_6h_bounds(&self, position: Vec2) -> bool {
        position.0 >= 0 && position.0 < 10 && position.1 >= 0 && position.1 < 6
    }
    pub fn in_bounds(&self, position: Vec2) -> bool {
        position.0 >= 0 && position.0 < 10 && position.1 >= 0 && position.1 < 24
    }
    
    pub fn get_piece_color(&self) -> PieceColor {
        piece_color_from_int((self.0 & 15) as u8)
    }
    pub fn set_piece_color(&mut self, color: PieceColor) {
        self.0 = (self.0 & !15) | (color as u64);
    }
    pub fn as_array(&self) -> [u8; 240] {
        let mut base = [0; 240];
        for i in 0..60 {
            base[i] = ((self.0 >> i >> 4) & 1) as u8;
        }
        base
    }
    pub fn as_board(&self) -> TetBoard {
        let mut new_board = TetBoard::new();
        for i in 0..60 {
            new_board.set_tile(i % 10, 22 - (i / 10), (((self.0 >> i >> 4) & 1) * 8).try_into().unwrap());
        }
        new_board
    }
}
impl Board for u64_board {
    fn get_tile_array(&self) -> [u8; 240] {
        return self.as_array();
    }
    fn get_tile_matrix(&self) -> [[u8; 10]; 24] {
        let mut matrix: [[u8; 10]; 24] = [[0; 10]; 24];
        for y in 0..24 {
            for x in 0..10 {
                matrix[y][x] = self.get_tile(x.try_into().unwrap(), y.try_into().unwrap());
            }
        }
        return matrix;
    }
    fn from_int_array(arr: [u8; 240]) -> u64_board {
        let mut new_board = u64_board(0);
        for i in 0..60 {
            new_board.0 |= (arr[i] as u64) << i;
        }         

        return new_board;
    }
    fn from_4h_array(arr: [u8; 40]) -> u64_board {
        let mut tiles = [0; 240];
        for y in 0..4 {
            for x in 0..10 {
                tiles[y * 10 + x] = arr[y * 10 + x];
            }
        }
        return Self::from_int_array(tiles);
    } 
    fn tile_occupied(&self, x: isize, y: isize) -> bool {
        self.get_tile(x, y) != 0
    }

    
    fn clear_tile(&mut self, x: isize, y: isize) {
        self.0 &= !(1 << (4 + x + y * 10));
    }    

    fn in_bounds(&self, pos: Vec2) -> bool { 
        return pos.0 > -1 && pos.0 < 10 && pos.1 > -1 && pos.1 < 24
    }
    fn rotate_piece(&self , piece: &mut TetPiece, rotation: u8) -> bool {
        let mut test_piece = piece.clone();
        let mod_rot = rotation % 4;
        let old_rot: usize = piece.rotation as usize;
        let new_rot = (piece.rotation + mod_rot as i64 ) % 4;
        test_piece.rotation = Direction::from_int(new_rot.into());
        if mod_rot == 2 {
            // 180 rotation
            let kicks = get_180_kicks(*piece);
            let mut passed_tests = true;
            for i in 0..2 { 
                let shift: Vec2 = kicks[old_rot][i] - kicks[new_rot as usize][i];
                test_piece.position += shift;
                if self.does_collide(test_piece) {
                    test_piece.position -= shift;
                    passed_tests = false;
                } else {
                    piece.position = test_piece.position;
                    piece.rotation = test_piece.rotation;
                    return true;
                }
            }
            
        } else  {
            let kicks = get_kicks(*piece);
            let mut passed_tests = true;
            for i in 0..5 { 
                let old_offset = kicks[old_rot][i];
                let new_offset = kicks[new_rot as usize][i];
                
                let shift: Vec2 = kicks[old_rot][i] - kicks[new_rot as usize][i];
                test_piece.position += shift;
                
                // print!("===========NEW ROT===========\n");
                // println!("Old offset: {:?}, New offset: {:?}", old_offset, new_offset);
                // println!("Attempting to rotate with offset {:?}", shift);
                // println!("{:?}", Vec2(10,23) - test_piece.position);
                // println!("{}", field::Field::new(*self, test_piece));
                if self.does_collide(test_piece) {
                    test_piece.position -= shift;
                    passed_tests = false;
                } else {
                    piece.position = test_piece.position;
                    piece.rotation = test_piece.rotation;
                    return true;
                }
                
            }
            
        }
        return false;
        
    }

    fn das_piece(&self, piece: &mut TetPiece, direction: Direction) { 
        match direction { 
            Direction::East => {
                for i in 0..11 {
                    piece.position += Vec2(1, 0);
                    if self.does_collide(*piece) {
                        piece.position -= Vec2(1, 0);
                        break;
                    }
                }
            }
            Direction::West => {
                for i in 0..11 {
                    piece.position += Vec2(-1, 0);
                    if self.does_collide(*piece) {
                        piece.position -= Vec2(-1, 0);
                        break;
                    }
                }
            }
            Direction::South => {
                for i in 0..23 {
                    if self.does_collide(*piece) {
                        
                        piece.position += Vec2(0, 1);
                        break;
                    }
                    piece.position -= Vec2(0, 1);
                }
            }
            _ => {}
        }
        
    }
    fn apply_gravity(&mut self, piece: &mut TetPiece) {
        piece.position += Vec2(0, 1);
        if self.does_collide(*piece) {
            piece.position -= Vec2(0, 1);
        }
    }

    fn get_tile(&self, x: isize, y: isize) -> u8 {
        if self.in_6h_bounds(Vec2(x.try_into().unwrap(), y.try_into().unwrap())) {
            ((self.0 >> (4 + x + y * 10)) & 1) as u8
        } else {
            1
        }
        
    }

    fn set_tile(&mut self, x: isize, y: isize, new: u8) {
        if self.in_6h_bounds(Vec2(x.try_into().unwrap(), y.try_into().unwrap())) {
            self.0 = (self.0 & !(1 << (4 + x + y * 10))) | ((new as u64) << (4 + x + y * 10));
        }
    }

    fn does_collide(&self, piece: TetPiece) -> bool { 
        let minos = piece.get_minos();
        if piece.position.1 > 6 {
            return true;
        } else {
            for mino in minos {
                
                if self.get_tile(mino.0.try_into().unwrap(), mino.1.try_into().unwrap()) == 1 {
                    return true;
                }
            }
        }
        return false;
    }
    fn can_place(&self, piece: TetPiece) -> bool {
        if self.does_collide(piece) {
           false 
        } else {
            let mut test = piece.clone();
            test.apply_gravity(1);
            self.does_collide(test)
        }
    }
}
impl std::fmt::Display for u64_board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_board())
    }
}