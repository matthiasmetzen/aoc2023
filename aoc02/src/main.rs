use std::fs;

fn main() {
    let input = fs::read_to_string("input.txt").expect("Could not read input file");
    let games = parse_input(input.as_str());

    let sum = sum_possible_ids(&games);
    println!("Sum of impossible game IDs: {}", sum);

    let sum_of_power = sum_of_power(&games);
    println!("Sum of game powers: {}", sum_of_power);
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Round {
    red: u64,
    green: u64,
    blue: u64,
}

impl Round {
    pub fn partial_max(&self, other: &Self) -> Self {
        let mut max = *self;
        max.red = max.red.max(other.red);
        max.green = max.green.max(other.green);
        max.blue = max.blue.max(other.blue);

        max
    }
}

pub struct Game {
    id: u64,
    rounds: Vec<Round>,
}

impl Game {
    pub fn is_impossible(&self) -> bool {
        self.rounds
            .iter()
            .any(|round| round.red > 12 || round.green > 13 || round.blue > 14)
    }

    pub fn min_required(&self) -> Round {
        let mut min = Round::default();
        for round in &self.rounds {
            min = min.partial_max(round);
        }

        min
    }

    pub fn get_power(&self) -> u64 {
        let min_req = self.min_required();
        min_req.red * min_req.green * min_req.blue
    }
}

fn parse_input(input: &str) -> Vec<Game> {
    let mut res = vec![];
    for line in input.lines() {
        let (game_part, color_part) = line.split_once(":").unwrap();
        let (_, idx) = game_part.split_once(" ").unwrap();
        let id: u64 = idx.parse().unwrap();

        let mut game = Game { id, rounds: vec![] };

        let rounds = color_part
            .trim()
            .split(";")
            .map(|color| color.trim().split(", "));

        for round in rounds {
            let mut rnd = Round::default();

            for color in round {
                let (amount, cname) = color.trim().split_once(" ").unwrap();
                let amount = amount.trim().parse().unwrap();
                match cname {
                    "red" => rnd.red = amount,
                    "green" => rnd.green = amount,
                    "blue" => rnd.blue = amount,
                    _ => panic!("invalid color"),
                }
            }

            game.rounds.push(rnd);
        }

        res.push(game);
    }

    res
}

fn sum_possible_ids(games: &Vec<Game>) -> u64 {
    games
        .iter()
        .filter(|game| !game.is_impossible())
        .map(|game| game.id)
        .sum()
}

fn sum_of_power(games: &Vec<Game>) -> u64 {
    games.iter().map(|game| game.get_power()).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    use test_case::test_case;

    #[test_case("Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green")]
    #[test_case("Game 2: 1 blue, 2 green; 3 green, 4 blue, 1 red; 1 green, 1 blue")]
    #[test_case("Game 3: 8 green, 6 blue, 20 red; 5 blue, 4 red, 13 green; 5 green, 1 red")]
    #[test_case("Game 4: 1 green, 3 red, 6 blue; 3 green, 6 red; 3 green, 15 blue, 14 red")]
    #[test_case("Game 5: 6 red, 1 blue, 3 green; 2 blue, 1 red, 2 green")]
    fn test_parser(line: &str) -> Result<(), ()> {
        let games = parse_input(line);

        assert!(games.len() == 1);
        Ok(())
    }

    #[test_case("Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green", 4, 2, 6)]
    #[test_case(
        "Game 2: 1 blue, 2 green; 3 green, 4 blue, 1 red; 1 green, 1 blue",
        1,
        3,
        4
    )]
    #[test_case(
        "Game 3: 8 green, 6 blue, 20 red; 5 blue, 4 red, 13 green; 5 green, 1 red",
        20,
        13,
        6
    )]
    #[test_case(
        "Game 4: 1 green, 3 red, 6 blue; 3 green, 6 red; 3 green, 15 blue, 14 red",
        14,
        3,
        15
    )]
    #[test_case("Game 5: 6 red, 1 blue, 3 green; 2 blue, 1 red, 2 green", 6, 3, 2)]
    fn test_min_req(line: &str, red: u64, green: u64, blue: u64) {
        let game = parse_input(line).pop().unwrap();
        let min_req = game.min_required();

        let desired = Round { red, green, blue };
        assert_eq!(min_req, desired);
    }

    #[test_case("Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green", 48)]
    #[test_case("Game 2: 1 blue, 2 green; 3 green, 4 blue, 1 red; 1 green, 1 blue", 12)]
    #[test_case(
        "Game 3: 8 green, 6 blue, 20 red; 5 blue, 4 red, 13 green; 5 green, 1 red",
        1560
    )]
    #[test_case(
        "Game 4: 1 green, 3 red, 6 blue; 3 green, 6 red; 3 green, 15 blue, 14 red",
        630
    )]
    #[test_case("Game 5: 6 red, 1 blue, 3 green; 2 blue, 1 red, 2 green", 36)]
    fn test_power(line: &str, power: u64) {
        let game = parse_input(line).pop().unwrap();

        assert_eq!(game.get_power(), power);
    }
}
