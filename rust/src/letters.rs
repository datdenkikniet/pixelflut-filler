pub type Letter = [u8; 35];
pub const LETTER_HEIGHT: usize = 7;
pub const LETTER_WIDTH: usize = 5;

/*

pub const X: Letter = [0, 0, 0, 0, 0,
                       0, 0, 0, 0, 0,
                       0, 0, 0, 0, 0,
                       0, 0, 0, 0, 0,
                       0, 0, 0, 0, 0,
                       0, 0, 0, 0, 0,
                       0, 0, 0, 0, 0];

*/

pub const A: Letter = [0, 0, 1, 0, 0,
                       0, 1, 0, 1, 0,
                       0, 1, 0, 1, 0,
                       1, 0, 0, 0, 1,
                       1, 1, 1, 1, 1,
                       1, 0, 0, 0, 1,
                       0, 0, 0, 0, 0];


pub const B: Letter = [1, 1, 1, 0, 0,
                       1, 0, 0, 1, 0,
                       1, 0, 0, 1, 0,
                       1, 1, 1, 0, 0,
                       1, 0, 0, 1, 0,
                       1, 0, 0, 1, 0,
                       1, 1, 1, 0, 0];