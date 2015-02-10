#![feature(test, core, collections)]

extern crate test;

use std::cmp::{max, min};
use std::collections::Bitv;

#[derive(Debug, PartialEq, Eq)]
pub enum StrSimError {
    DifferentLengthArgs
}

pub type HammingResult = Result<usize, StrSimError>;

pub fn hamming(a: &str, b: &str) -> HammingResult {
    if a.len() != b.len() {
        Err(StrSimError::DifferentLengthArgs)
    } else {
        Ok(a.chars()
            .zip(b.chars())
            .filter(|&(a_char, b_char)| a_char != b_char)
            .count())
    }
}

pub fn jaro(a: &str, b: &str) -> f64 {
    if a == b { return 1.0; }
    if a.len() == 0 || b.len() == 0 { return 0.0; }

    let search_range = max(0, (max(a.len(), b.len()) / 2) - 1);
    
    let mut b_consumed = Bitv::from_elem(b.len(), false);
    let mut matches = 0.0;

    let mut transpositions = 0.0;
    let mut b_match_index = 0;

    for (i, a_char) in a.chars().enumerate() {
        let min_bound = 
            // prevent integer wrapping
            if i > search_range {
                max(0, i - search_range)
            } else {
                0
            };
        let max_bound = min(b.len() - 1, i + search_range);

        for j in min_bound..max_bound + 1 {
            let b_char = b.char_at(j);
            if a_char == b_char && !b_consumed[j] {
                b_consumed.set(j, true);
                matches += 1.0;

                if j < b_match_index {
                    transpositions += 1.0;
                }
                b_match_index = j;

                break;
            }
        }
    }

    if matches == 0.0 {
        0.0
    } else {
        (1.0 / 3.0) * ((matches / a.len() as f64) +
                       (matches / b.len() as f64) +
                       ((matches - transpositions) / matches))
    }
}

// Does not limit the length of the common prefix
pub fn jaro_winkler(a: &str, b: &str) -> f64 {
    let jaro_distance = jaro(a, b);

    let prefix = a.chars()
                  .zip(b.chars())
                  .take_while(|&(a_char, b_char)| a_char == b_char)
                  .count();

    jaro_distance + (0.1 * prefix as f64 * (1.0 - jaro_distance))
}

pub fn levenshtein(a: &str, b: &str) -> usize {
    if a == b { return 0; }
    else if a.len() == 0 { return b.len(); }
    else if b.len() == 0 { return a.len(); }

    let mut prev_distances: Vec<usize> = Vec::with_capacity(b.len() + 1);
    let mut curr_distances: Vec<usize> = Vec::with_capacity(b.len() + 1);

    for i in 0..(b.len() + 1) { 
        prev_distances.push(i); 
        curr_distances.push(0);
    }

    for (i, a_char) in a.chars().enumerate() {
        curr_distances[0] = i + 1;

        for (j, b_char) in b.chars().enumerate() {
            let cost = if a_char == b_char { 0 } else { 1 };
            curr_distances[j + 1] = min(curr_distances[j] + 1,
                                        min(prev_distances[j + 1] + 1,
                                            prev_distances[j] + cost));
        }

        prev_distances.clone_from(&curr_distances);
    }

    curr_distances[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[test]
    fn hamming_empty() {
        match hamming("", "") {
            Ok(distance) => { assert_eq!(0, distance); },
            Err(why) => { panic!("{:?}", why); }
        } 
    }

    #[test]
    fn hamming_same() {
        match hamming("hamming", "hamming") {
            Ok(distance) => { assert_eq!(0, distance); },
            Err(why) => { panic!("{:?}", why); }
        }
    }

    #[test]
    fn hamming_diff() {
        match hamming("hamming", "hammers") {
            Ok(distance) => { assert_eq!(3, distance); },
            Err(why) => { panic!("{:?}", why); }
        }
    }

    #[test]
    fn hamming_unequal_length() {
        match hamming("ham", "hamming") {
            Ok(_) => { panic!(); },
            Err(why) => { assert_eq!(why, StrSimError::DifferentLengthArgs); }
        }
    }

    #[test]
    fn hamming_names() {
        match hamming("Friedrich Nietzs", "Jean-Paul Sartre") {
            Ok(distance) => { assert_eq!(14, distance); },
            Err(why) => { panic!("{:?}", why); }
        }
    }

    #[test]
    fn jaro_both_empty() {
       assert_eq!(1.0, jaro("", "")); 
    }

    #[test]
    fn jaro_first_empty() {
        assert_eq!(0.0, jaro("", "jaro"));
    }

    #[test]
    fn jaro_second_empty() {
        assert_eq!(0.0, jaro("distance", ""));
    }

    #[test]
    fn jaro_same() {
        assert_eq!(1.0, jaro("jaro", "jaro"));
    }

    #[test]
    fn jaro_diff_short() {
        assert!(0.767 - jaro("dixon", "dicksonx") < 0.001);
    }

    #[test]
    fn jaro_diff_no_transposition() {
        assert!(0.822 - jaro("dwayne", "duane") < 0.001);
    }

    #[test]
    fn jaro_diff_with_transposition() {
        assert!(0.944 - jaro("martha", "marhta") < 0.001);
    }

    #[test]
    fn jaro_names() {
        assert!((0.392 - jaro("Friedrich Nietzsche",
                              "Jean-Paul Sartre")) < 0.001);
    }

    #[test]
    fn jaro_winkler_both_empty() {
        assert_eq!(1.0, jaro_winkler("", ""));
    }

    #[test]
    fn jaro_winkler_first_empty() {
        assert_eq!(0.0, jaro_winkler("", "jaro-winkler"));
    }

    #[test]
    fn jaro_winkler_second_empty() {
        assert_eq!(0.0, jaro_winkler("distance", ""));
    }

    #[test]
    fn jaro_winkler_same() {
        assert_eq!(1.0, jaro_winkler("Jaro-Winkler", "Jaro-Winkler"));
    }

    #[test]
    fn jaro_winkler_diff_short() {
        assert!(0.813 - jaro_winkler("dixon", "dicksonx") < 0.001);
        assert!(0.813 - jaro_winkler("dicksonx", "dixon") < 0.001);
    }

    #[test]
    fn jaro_winkler_diff_no_transposition() {
        assert!(0.840 - jaro_winkler("dwayne", "duane") < 0.001);
    }

    #[test]
    fn jaro_winkler_diff_with_transposition() {
        assert!(0.961 - jaro_winkler("martha", "marhta") < 0.001);
    }

    #[test]
    fn jaro_winkler_names() {
        assert!((0.562 - jaro_winkler("Friedrich Nietzsche",
                                      "Fran-Paul Sartre")) < 0.001);
    }

    #[test]
    fn jaro_winkler_long_prefix() {
        assert!(0.911 - jaro_winkler("cheeseburger", "cheese fries") < 0.001);
    }

    #[test]
    fn jaro_winkler_more_names() {
        assert!(0.868 - jaro_winkler("Thorkel", "Thorgier") < 0.001);
    }

    #[test]
    fn jaro_winkler_length_of_one() {
        assert!(0.738 - jaro_winkler("Dinsdale", "D") < 0.001);
    }

    #[test]
    fn jaro_winkler_very_long_prefix() {
        assert!(1.0 - jaro_winkler("thequickbrownfoxjumpedoverx",
                                   "thequickbrownfoxjumpedovery") < 0.001);
    }

    #[test]
    fn levenshtein_empty() {
        assert_eq!(0, levenshtein("", ""));
    }

    #[test]
    fn levenshtein_same() {
        assert_eq!(0, levenshtein("levenshtein", "levenshtein"));
    }

    #[test]
    fn levenshtein_diff_short() {
        assert_eq!(3, levenshtein("kitten", "sitting"));
    }

    #[test]
    fn levenshtein_diff_with_space() {
        assert_eq!(5, levenshtein("hello, world", "bye, world"));
    }

    #[test]
    fn levenshtein_diff_longer() {
        let a = "The quick brown fox jumped over the angry dog.";
        let b = "Lorem ipsum dolor sit amet, dicta latine an eam.";
        assert_eq!(37, levenshtein(a, b));
    }

    #[test]
    fn levenshtein_first_empty() {
        assert_eq!(7, levenshtein("", "sitting"));
    }

    #[test]
    fn levenshtein_second_empty() {
        assert_eq!(6, levenshtein("kitten", ""));
    }

    #[bench]
    fn bench_hamming(b: &mut Bencher) {
        b.iter(|| hamming("Friedrich Nietzs", "Jean-Paul Sartre"));
    }

    #[bench]
    fn bench_levenshtein(b: &mut Bencher) {
        b.iter(|| levenshtein("Friedrich Nietzsche", "Jean-Paul Sartre"));
    }

    #[bench]
    fn bench_jaro(b: &mut Bencher) {
        b.iter(|| jaro("Friedrich Nietzsche", "Jean-Paul Sartre"));
    }
}