use std::{fmt::LowerHex, str::FromStr};

use rand::{thread_rng, RngCore};

pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: Option<u8>,
}

impl LowerHex for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:02x}", self.r))?;
        f.write_str(&format!("{:02x}", self.g))?;
        f.write_str(&format!("{:02x}", self.b))?;
        if let Some(a) = self.a {
            f.write_str(&format!("{:02x}", a))?;
        }
        Ok(())
    }
}

impl Color {
    pub fn random() -> Self {
        Color {
            r: thread_rng().next_u32() as u8,
            g: thread_rng().next_u32() as u8,
            b: thread_rng().next_u32() as u8,
            a: Some(thread_rng().next_u32() as u8),
        }
    }
}

impl FromStr for Color {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let len = s.len();
        if len != 6 && len != 8 {
            return Err(String::from("Color must be 6 or 8 characters in length"));
        }

        let mut digits = Vec::new();
        for char in s.chars() {
            if let Some(value) = char.to_digit(16) {
                digits.push(value);
            } else {
                return Err(format!("Invalid hex digit {}", char));
            }
        }

        let mut color = Color {
            r: (digits[0] + digits[1] * 16) as u8,
            g: (digits[2] + digits[3] * 16) as u8,
            b: (digits[4] + digits[5] * 16) as u8,
            a: None,
        };

        if len == 8 {
            color.a = Some((digits[6] + digits[7] * 16) as u8)
        }

        Ok(color)
    }
}
