mod consts;
use std::slice::Iter;

pub use consts::*;

pub struct LetterString<'a> {
    pub letters: Vec<&'a Letter>,
}

impl<'a> From<&str> for LetterString<'a> {
    fn from(string: &str) -> Self {
        let mut letters = Vec::new();
        for character in string.chars() {
            match character {
                'a' | 'A' => letters.push(&A),
                'b' | 'B' => letters.push(&B),
                'c' | 'C' => letters.push(&C),
                'd' | 'D' => letters.push(&D),
                'e' | 'E' => letters.push(&E),
                'g' | 'G' => letters.push(&G),
                'h' | 'H' => letters.push(&H),
                'l' | 'L' => letters.push(&L),
                'm' | 'M' => letters.push(&M),
                'n' | 'N' => letters.push(&N),
                'u' | 'U' => letters.push(&U),
                ' ' => letters.push(&SPACE),
                _ => letters.push(&UNKNOWN),
            }
        }
        Self { letters }
    }
}

impl<'a> LetterString<'a> {
    pub fn iter(&self) -> Iter<&Letter> {
        self.letters.iter()
    }
}
