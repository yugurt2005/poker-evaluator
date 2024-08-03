use std::{
    cmp,
    fs::File,
    io::{Read, Write},
};

use poker_indexer::Indexer;

const WORST: u16 = 7462;

const TABLE_5_SIZE: usize = 134459;
const TABLE_6_SIZE: usize = 962988;
const TABLE_7_SIZE: usize = 6009159;

pub struct Evaluator {
    table: Vec<u16>,

    indexer: Indexer,
}

impl Evaluator {
    pub fn new(path: String, load: bool) -> Self {
        if !load {
            let mut contents = Vec::new();

            File::open(path.clone() + "/classes.bin")
                .unwrap()
                .read_to_end(&mut contents)
                .unwrap();

            let classes = contents
                .chunks(2)
                .map(|chunk| {
                    let a = chunk[0] as u16;
                    let b = chunk[1] as u16;
                    a << 8 | b
                })
                .collect();

            let indexer_5 = Indexer::new(vec![5]);
            let indexer_6 = Indexer::new(vec![6]);
            let indexer_7 = Indexer::new(vec![7]);

            println!("Generating Tables...");
            let table_5 = Evaluator::gen_5(&indexer_5, &classes);
            println!("Table 5 Generated");
            let table_6 = Evaluator::gen_6(&indexer_5, &indexer_6, &table_5);
            println!("Table 6 Generated");
            let table_7 = Evaluator::gen_7(&indexer_6, &indexer_7, &table_6);
            println!("Table 7 Generated");

            let evaluator = Self {
                table: table_7,
                indexer: indexer_7,
            };

            evaluator.save(path);
            evaluator
        } else {
            Evaluator::load(path)
        }
    }

    fn load(path: String) -> Evaluator {
        let mut contents = Vec::new();

        File::open(path + "/table.bin")
            .unwrap()
            .read_to_end(&mut contents)
            .unwrap();

        let table = contents
            .chunks(2)
            .map(|chunk| {
                let a = chunk[0] as u16;
                let b = chunk[1] as u16;
                a << 8 | b
            })
            .collect();

        Evaluator {
            table,
            indexer: Indexer::new(vec![7]),
        }
    }

    fn save(&self, path: String) {
        let mut file = File::create(path.clone() + "/table.bin").unwrap();
        for x in &self.table {
            file.write_all(&x.to_be_bytes()).unwrap();
        }
    }

    fn index(cards: u64) -> usize {
        (0..52).fold(
            if (cards & ((1 << 13) - 1)).count_ones() == 5 {
                1
            } else {
                0
            },
            |acc, i| {
                if cards & 1 << (i / 4 + i % 4 * 13) != 0 {
                    acc * 13 + i / 4
                } else {
                    acc
                }
            },
        )
    }

    fn gen_5(indexer: &Indexer, classes: &Vec<u16>) -> Vec<u16> {
        let mut table = Vec::new();

        for i in 0..TABLE_5_SIZE {
            table.push(classes[Evaluator::index(indexer.unindex(i as u64)[0])]);
        }

        table
    }

    fn gen_6(indexer_5: &Indexer, indexer_6: &Indexer, table_5: &Vec<u16>) -> Vec<u16> {
        let mut table_6 = vec![WORST; TABLE_6_SIZE];

        for i in 0..TABLE_6_SIZE {
            let cards = indexer_6.unindex(i as u64)[0];

            let mut value = cards as i64;
            while value > 0 {
                let bit = value & -value;

                table_6[i] = cmp::min(
                    table_6[i],
                    table_5[indexer_5.index(vec![(cards - bit as u64) as u64]) as usize],
                );

                value -= bit;
            }
        }

        table_6
    }

    fn gen_7(indexer_6: &Indexer, indexer_7: &Indexer, table_6: &Vec<u16>) -> Vec<u16> {
        let mut table_7 = vec![WORST; TABLE_7_SIZE];

        for i in 0..TABLE_7_SIZE {
            let cards = indexer_7.unindex(i as u64)[0];

            let mut value = cards as i64;
            while value > 0 {
                let bit = value & -value;

                table_7[i] = cmp::min(
                    table_7[i],
                    table_6[indexer_6.index(vec![(cards - bit as u64) as u64]) as usize],
                );

                value -= bit;
            }
        }

        table_7
    }

    pub fn evaluate(&self, cards: u64) -> u16 {
        assert!(cards.count_ones() == 7);

        self.table[self.indexer.index(vec![cards]) as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        Evaluator::new("src/".to_string(), false);
    }

    #[test]
    fn test_evaluate() {
        let evaluator = Evaluator::new("src/".to_string(), true);

        let actual =
            evaluator.evaluate(1 << 12 | 1 << 11 | 1 << 10 | 1 << 9 | 1 << 8 | 1 << 25 | 1 << 38);
        let expect = 1;

        assert_eq!(actual, expect);
    }

    #[test]
    fn test_evaluate_2() {
        let evaluator = Evaluator::new("src/".to_string(), true);

        let actual =
            evaluator.evaluate(1 << 0 | 1 << 13 | 1 << 39 | 1 << 1 | 1 << 14 | 1 << 2 | 1 << 3);
        let expect = 322;

        assert_eq!(actual, expect);
    }
}
