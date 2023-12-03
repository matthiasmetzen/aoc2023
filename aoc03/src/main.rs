#![feature(slice_as_chunks)]
#![feature(iter_array_chunks)]

use std::fs;

use burn::backend::{wgpu::OpenGl, Wgpu};
use burn::record::Record;
use burn::tensor::ops::ConvOptions;
use burn::tensor::Int;
use burn::tensor::{module::conv2d, Tensor};
use itertools::Itertools;

type Backend = Wgpu<OpenGl>;

fn main() {
    let input = fs::read_to_string("input.txt").unwrap();
    let schematic = parse_schematic(input.as_str());
    let parts = schematic.get_part_nums();

    println!("Sum of parts: {}", parts.iter().sum::<u64>());

    let ratios = schematic.get_gear_ratios();

    println!("Sum of ratios: {}", ratios.iter().sum::<u64>());
}

#[derive(Debug)]
struct Entry<T> {
    val: T,
    x: u64,
    y: u64,
}

#[derive(Debug)]
pub struct Schematic {
    numbers: Vec<Entry<u64>>,
    symbols: Vec<Entry<char>>,
    rows: usize,
    cols: usize,
}

pub trait Digits {
    fn count_digits(&self) -> usize;
}

impl Digits for f64 {
    fn count_digits(&self) -> usize {
        self.log10().floor() as usize + 1
    }
}

impl Digits for u64 {
    fn count_digits(&self) -> usize {
        (*self as f64).count_digits()
    }
}

fn parse_schematic(input: &str) -> Schematic {
    let mut nums = vec![];
    let mut syms = vec![];

    for (y, line) in (0u64..).zip(input.lines()) {
        let line = line.trim();
        // parse nums
        let line_ptr = line.as_ptr() as u64;

        let num_splits = line
            .split(|c: char| !c.is_ascii_digit())
            .filter(|e| !e.is_empty());

        for num in num_splits {
            // calculate offset from beginning of line
            let val: u64 = num.parse().unwrap();
            let num_ptr = num.as_ptr() as u64;
            let x = num_ptr - line_ptr;
            nums.push(Entry {
                val,
                x,
                y: y as u64,
            })
        }

        //parse syms
        let sym_splits = line
            .split(|c: char| c.is_ascii_digit() || c == '.')
            .filter(|e| !e.is_empty());
        for sym in sym_splits {
            let val = sym.chars().next().unwrap();
            let sym_ptr = sym.as_ptr() as u64;
            let x = sym_ptr - line_ptr;
            syms.push(Entry {
                val,
                x,
                y: y as u64,
            })
        }
    }

    Schematic {
        numbers: nums,
        symbols: syms,
        cols: input.lines().next().unwrap().len(),
        rows: input.lines().count(),
    }
}

impl Schematic {
    fn as_tensors(&self) -> (Tensor<Backend, 4, Int>, Tensor<Backend, 4, Int>) {
        let mut nums_matrix = Tensor::<Backend, 4, Int>::zeros([1, 1, self.rows, self.cols]);

        for entry in self.numbers.iter() {
            let digits = entry.val.count_digits();
            let x = entry.x as usize;
            let y = entry.y as usize;
            nums_matrix = nums_matrix.slice_assign(
                [0..1, 0..1, y..y + 1, x..x + digits],
                Tensor::<Backend, 4, Int>::ones([1, 1, 1, digits]) * entry.val as u32,
            );
        }

        let mut syms_matrix = Tensor::<Backend, 4, Int>::zeros([1, 1, self.rows, self.cols]);
        for entry in self.symbols.iter() {
            let x = entry.x as usize;
            let y = entry.y as usize;
            syms_matrix = syms_matrix.slice_assign(
                [0..1, 0..1, y..y + 1, x..x + 1],
                Tensor::<Backend, 4, Int>::ones([1, 1, 1, 1]) * entry.val as u32,
            );
        }

        (nums_matrix, syms_matrix)
    }

    pub fn get_part_nums(&self) -> Vec<u64> {
        let (nums_matrix, syms_matrix) = self.as_tensors();

        let conv3x3 = Tensor::<Backend, 4>::ones([1, 1, 3, 3]);

        let conv_res = conv2d(
            syms_matrix.float().clamp(0., 1.),
            conv3x3,
            None,
            ConvOptions::new([1, 1], [1, 1], [1, 1], 1),
        )
        .clamp(0., 1.);

        let parts_tensor = conv_res.mul(nums_matrix.float());

        // clear duplicates
        let conv3x3 =
            Tensor::<Backend, 2>::from_floats([[0., 0., 0.], [0., 1., -1.], [0., 0., 0.]])
                .reshape([1, 1, 3, 3]);

        let parts_tensor = conv2d(
            parts_tensor,
            conv3x3,
            None,
            ConvOptions::new([1, 1], [1, 1], [1, 1], 1),
        )
        .int();

        let parts: Vec<_> = parts_tensor
            .into_data()
            .value
            .into_iter()
            .filter(|v| *v > 0)
            .map(|v| v as u64)
            .collect();

        parts
    }

    fn get_gear_ratios(&self) -> Vec<u64> {
        let (nums_matrix, syms_matrix) = self.as_tensors();

        let gear_pos_mask = syms_matrix.clone().equal_elem('*' as u32);
        let num_pos_mask = nums_matrix.clone().greater_elem(0);
        let syms_matrix = syms_matrix
            .mask_fill(gear_pos_mask.clone().bool_not(), 0)
            .float()
            .clamp(0., 1.);

        let conv_gear3x3 = Tensor::<Backend, 2, Int>::from_ints([[1, 1, 1], [1, 0, 1], [1, 1, 1]])
            .float()
            .reshape([1, 1, 3, 3]);

        let gear_nums = conv2d(
            syms_matrix.clone(),
            conv_gear3x3.clone(),
            None,
            ConvOptions::new([1, 1], [1, 1], [1, 1], 1),
        );

        let gear_nums = gear_nums.mask_fill(num_pos_mask.bool_not(), 0);

        // clear duplicates
        let conv_clean3x3 =
            Tensor::<Backend, 2>::from_floats([[0., 0., 0.], [0., 1., -1.], [0., 0., 0.]])
                .reshape([1, 1, 3, 3]);

        let gear_connections = conv2d(
            gear_nums.clone(),
            conv_clean3x3,
            None,
            ConvOptions::new([1, 1], [1, 1], [1, 1], 1),
        )
        .clamp(0., 1.);
        println!("{}", gear_connections);
        //println!("{}", gear_nums);

        let gear_parts = gear_connections.clone().mul(nums_matrix.clone().float());

        // limit gears to 2 parts
        let gear_parts_count = conv2d(
            gear_connections.clone(),
            conv_gear3x3.clone(),
            None,
            ConvOptions::new([1, 1], [1, 1], [1, 1], 1),
        )
        .mask_fill(gear_pos_mask.clone().bool_not(), 0);

        let gear_pos_mask = gear_parts_count.clone().equal_elem(2.);
        /*let gear_pos = gear_parts_count
        .mask_fill(gear_pos_mask.clone().bool_not(), 0.)
        .clamp(0., 1.);*/

        let mask_vec: Vec<bool> = gear_pos_mask.into_data().value;
        let nums_vec: Vec<i32> = gear_parts.int().into_data().value;

        let mut ratios = vec![];
        for y in 0i32..self.rows as i32 {
            for x in 0i32..self.cols as i32 {
                if mask_vec[(y * self.cols as i32 + x) as usize] {
                    let ratio: u64 = (-1i32..=1)
                        .cartesian_product(-1i32..=1)
                        .map(|(j, i)| {
                            nums_vec
                                .get(((y + j) * self.cols as i32 + x + i) as usize)
                                .copied()
                                .unwrap_or_default() as u64
                        })
                        .filter(|v| *v > 0)
                        .product();

                    ratios.push(ratio);
                }
            }
        }

        ratios
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use test_case::test_case;

    #[test_case(
        "467..114..
        ...*......
        ..35..633.
        ......#...
        617*......
        .....+.58.
        ..592.....
        ......755.
        ...$.*....
        .664.598.."
    )]
    fn test_parse(lines: &str) {
        let schematic = parse_schematic(lines);
        dbg!(schematic);
    }

    #[test_case(
        "467..114..
        ...*......
        ..35..633.
        ......#...
        617*......
        .....+.58.
        ..592.....
        ......755.
        ...$.*....
        .664.598..",
        vec![467, 35, 633, 617, 592, 755, 664, 598]
    )]
    fn test_parts(lines: &str, desired: Vec<u64>) {
        let schematic = parse_schematic(lines);
        let part_nums = schematic.get_part_nums();

        assert_eq!(part_nums, desired);
    }

    #[test_case(
        "467..114..
        ...*......
        ..35..633.
        ......#...
        617*......
        .....+.58.
        ..592.....
        ......755.
        ...$.*....
        .664.598..",
        vec![16345, 451490]
    )]
    fn test_ratios(lines: &str, desired: Vec<u64>) {
        let schematic = parse_schematic(lines);
        let ratios = schematic.get_gear_ratios();

        assert_eq!(ratios, desired);
    }
}
