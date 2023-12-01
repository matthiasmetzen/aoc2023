use std::{fs, io::Error};

fn main() -> Result<(), Error> {
    let input = fs::read_to_string("input.txt")?;

    let res = read_document(input.as_str());

    println!("Result: {}", res);

    Ok(())
}

enum NumberPos {
    Digit(usize, u8),
    String(usize, u8),
}

impl NumberPos {
    pub fn get_pos(&self) -> usize {
        match self {
            Self::Digit(pos, _) => *pos,
            Self::String(pos, _) => *pos,
        }
    }

    pub fn get_val(&self) -> u8 {
        match self {
            Self::Digit(_, val) => *val,
            Self::String(_, val) => *val,
        }
    }
}

fn read_calibration_number(line: &str) -> Option<u8> {
    let numbers = [
        "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
    ];

    let mut first = line.find(|c: char| c.is_ascii_digit()).map(|pos| {
        NumberPos::Digit(
            pos,
            line.chars().nth(pos).unwrap().to_digit(10).unwrap() as u8,
        )
    });

    for (idx, val) in numbers.iter().enumerate() {
        let pos = line.find(val);

        if let Some(p) = pos {
            if first.is_none() || p < first.as_ref().unwrap().get_pos() {
                first = Some(NumberPos::String(p, (idx + 1) as u8))
            }
        }
    }

    let mut last = line.rfind(|c: char| c.is_ascii_digit()).map(|pos| {
        NumberPos::Digit(
            pos,
            line.chars().nth(pos).unwrap().to_digit(10).unwrap() as u8,
        )
    });

    for (idx, val) in numbers.iter().enumerate() {
        let pos = line.rfind(val);

        if let Some(p) = pos {
            if last.is_none() || p > last.as_ref().unwrap().get_pos() {
                last = Some(NumberPos::String(p, (idx + 1) as u8))
            }
        }
    }

    Some(first?.get_val() * 10 + last?.get_val())
}

fn read_document(doc: &str) -> u32 {
    doc.lines()
        .map(|line| read_calibration_number(line).unwrap() as u32)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("1abc2", 12)]
    #[test_case("pqr3stu8vwx", 38)]
    #[test_case("a1b2c3d4e5f", 15)]
    #[test_case("treb7uchet", 77)]
    fn test_read_cfg_line(line: &str, desired: u8) {
        let res = read_calibration_number(line);

        assert_eq!(res, Some(desired));
    }

    #[test_case("two1nine", 29)]
    #[test_case("eightwothree", 83)]
    #[test_case("abcone2threexyz", 13)]
    #[test_case("xtwone3four", 24)]
    #[test_case("4nineeightseven2", 42)]
    #[test_case("zoneight234", 14)]
    #[test_case("7pqrstsixteen", 76)]
    #[test_case("eighthree", 83)]
    #[test_case("sevenine", 79)]
    fn test_read_with_spelled_digits(line: &str, desired: u8) {
        let res = read_calibration_number(line);

        assert_eq!(res, Some(desired));
    }
}
