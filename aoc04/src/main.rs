use std::{collections::HashSet, fs};

fn main() {
    let input = fs::read_to_string("input.txt").unwrap();

    let cards = parse(input.as_str());

    println!(
        "The cards are worth {} points!",
        cards.iter().map(|c| c.points()).sum::<u64>()
    );

    let cards = collect_cards(cards);

    println!(
        "Gained a total of {} cards!",
        cards.iter().map(|c| c.count).sum::<u64>()
    );
}

#[derive(Debug, Clone)]
struct Card {
    winning_nums: HashSet<u64>,
    my_nums: HashSet<u64>,
    count: u64,
}

impl Card {
    pub fn winning_nums(&self) -> Vec<u64> {
        self.my_nums
            .intersection(&self.winning_nums)
            .copied()
            .collect()
    }

    pub fn points(&self) -> u64 {
        let len = self.winning_nums().len();
        if len > 0 {
            2u64.pow(len as u32 - 1)
        } else {
            0
        }
    }
}

fn parse(doc: &str) -> Vec<Card> {
    doc.lines()
        .map(|line| line.trim().split_once(':').unwrap())
        .map(|(pre, rest)| {
            let _id = pre.trim_start_matches("Card ").parse::<u64>();
            let (win, own) = rest.split_once('|').unwrap();

            let win: HashSet<_> = win
                .split(' ')
                .filter(|v| !v.is_empty())
                .map(|v| v.parse::<u64>().unwrap())
                .collect();

            let own: HashSet<_> = own
                .split(' ')
                .filter(|v| !v.is_empty())
                .map(|v| v.parse::<u64>().unwrap())
                .collect();

            Card {
                winning_nums: win,
                my_nums: own,
                count: 1,
            }
        })
        .collect()
}

fn collect_cards(cards: Vec<Card>) -> Vec<Card> {
    let mut cards_res = cards.clone();
    for idx in 0..cards_res.len() {
        let card = &cards_res[idx];
        let num_winning = card.winning_nums().len();
        if num_winning == 0 {
            continue;
        }

        let cnt = card.count;

        for i in idx + 1..=idx + num_winning {
            if let Some(c) = cards_res.get_mut(i) {
                c.count += cnt;
            }
        }
    }

    cards_res
}

#[cfg(test)]
mod tests {
    use super::*;

    use test_case::test_case;

    #[test_case("Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53")]
    #[test_case("Card 2: 13 32 20 16 61 | 61 30 68 82 17 32 24 19")]
    #[test_case("Card 3:  1 21 53 59 44 | 69 82 63 72 16 21 14  1")]
    #[test_case("Card 4: 41 92 73 84 69 | 59 84 76 51 58  5 54 83")]
    #[test_case("Card 5: 87 83 26 28 32 | 88 30 70 12 93 22 82 36")]
    #[test_case("Card 6: 31 18 13 56 72 | 74 77 10 23 35 67 36 11")]
    fn test_parse(line: &str) {
        let cards = parse(line);

        assert!(cards.len() == 1);
    }

    #[test_case("Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53", vec![48, 83, 17, 86])]
    #[test_case("Card 2: 13 32 20 16 61 | 61 30 68 82 17 32 24 19", vec![32, 61])]
    #[test_case("Card 3:  1 21 53 59 44 | 69 82 63 72 16 21 14  1", vec![1, 21])]
    #[test_case("Card 4: 41 92 73 84 69 | 59 84 76 51 58  5 54 83", vec![84])]
    #[test_case("Card 5: 87 83 26 28 32 | 88 30 70 12 93 22 82 36", vec![])]
    #[test_case("Card 6: 31 18 13 56 72 | 74 77 10 23 35 67 36 11", vec![])]
    fn get_winning_nums(line: &str, mut desired: Vec<u64>) {
        let card = parse(line).pop().unwrap();

        let mut winning: Vec<_> = card.winning_nums();
        desired.sort();
        winning.sort();
        assert_eq!(winning, desired);
    }

    #[test_case("Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53" => 8)]
    #[test_case("Card 2: 13 32 20 16 61 | 61 30 68 82 17 32 24 19" => 2)]
    #[test_case("Card 3:  1 21 53 59 44 | 69 82 63 72 16 21 14  1" => 2)]
    #[test_case("Card 4: 41 92 73 84 69 | 59 84 76 51 58  5 54 83" => 1)]
    #[test_case("Card 5: 87 83 26 28 32 | 88 30 70 12 93 22 82 36" => 0)]
    #[test_case("Card 6: 31 18 13 56 72 | 74 77 10 23 35 67 36 11" => 0)]
    fn test_points(line: &str) -> u64 {
        let card = parse(line).pop().unwrap();

        card.points()
    }

    #[test_case(
        "Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53
        Card 2: 13 32 20 16 61 | 61 30 68 82 17 32 24 19
        Card 3:  1 21 53 59 44 | 69 82 63 72 16 21 14  1
        Card 4: 41 92 73 84 69 | 59 84 76 51 58  5 54 83
        Card 5: 87 83 26 28 32 | 88 30 70 12 93 22 82 36
        Card 6: 31 18 13 56 72 | 74 77 10 23 35 67 36 11"
        => 30
    )]
    fn test_collect_cards(doc: &str) -> u64 {
        let cards = parse(doc);

        let cards = collect_cards(cards);
        let cards_total: u64 = cards.iter().map(|c| c.count).sum();
        cards_total
    }
}
