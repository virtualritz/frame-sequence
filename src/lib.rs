//! A simple parser for frame sequences strings.
//!
//! This will parse [`str`] describing a sequence of frames into
//! a [`Vec`]`<`[`isize`]`>` containing individual frame numbers.
//!
//! Mainly intended/useful for rendering/animation applications.
//!
//! # Example Frame Sequence Strings
//!
//! Individual frames:
//!
//! `1,2,3,5,8,13` ⟶ `[1, 2, 3, 5, 8, 13]`
//!
//! A sequence:
//!
//! `10-15` ⟶ `[10, 11, 12, 13, 14, 15]`
//!
//! With step size:
//!
//! `10-20@2` ⟶ `[10, 12, 14, 16, 18, 20]`
//!
//! Step size must be always positive.
//!
//! To get a sequence backwards specify the range in reverse:
//!
//! `42-33@3` ⟶ `[42, 39, 36, 33]`
//!
//! With binary splitting:
//!
//! `10-20@b` ⟶ `[10, 20, 15, 12, 17, 11, 13, 16, 18, 14, 19]`
//!
//! The last frame of a sequence will be omitted if
//! the specified step size does not touch it:
//!
//! `80-70@4` ⟶ `[80, 76, 72]`
use itertools::Itertools;
use pest::{
    error::Error,
    iterators::{Pair, Pairs},
    Parser,
};
use pest_derive::Parser;
use std::{cmp::Ordering, collections::HashSet};

#[derive(Parser)]
#[grammar = "frame_format_grammar.pest"]
struct FrameSequenceParser;

/// Parse a frame sequence string into a [`Vec`]`<`[`isize`]`>` of frames.
///
/// See the main page of the documentation for example `input` strings.
pub fn parse_frame_sequence(input: &str) -> Result<Vec<isize>, Error<Rule>> {
    FrameSequenceParser::parse(Rule::FrameSequenceString, input)
        .map(|token_tree| remove_duplicates(frame_sequence_token_tree_to_frames(token_tree)))
}

fn chop(seq: &mut Vec<isize>, result: &mut Vec<isize>, elements: usize) {
    if seq.len() < elements {
        let mut new_seq = seq
            .iter()
            .tuple_windows()
            .flat_map(|pair: (&isize, &isize)| {
                let left = *pair.0;
                let right = (*pair.0 + *pair.1) / 2;
                if left < right {
                    result.push(right);
                    vec![left, right]
                } else {
                    vec![left]
                }
            })
            .collect::<Vec<_>>();
        new_seq.push(*seq.last().unwrap());
        if new_seq.len() < elements {
            chop(&mut new_seq, result, elements);
        }
        *seq = new_seq;
    }
}

pub(crate) fn binary_sequence(range: (isize, isize)) -> Vec<isize> {
    match range.0.cmp(&range.1) {
        Ordering::Less => {
            let mut seq = vec![range.0, range.1];
            let mut result = seq.clone();
            chop(&mut seq, &mut result, (range.1 - range.0) as _);
            result
        }
        Ordering::Greater => {
            let mut seq = vec![range.1, range.0];
            let mut result = seq.clone();
            chop(&mut seq, &mut result, (range.0 - range.1) as _);
            result.reverse();
            result
        }
        Ordering::Equal => vec![range.0],
    }
}

fn frame_to_number(frame: Pair<Rule>) -> isize {
    frame.as_str().parse::<isize>().unwrap()
}

fn frame_sequence_token_tree_to_frames(pairs: Pairs<Rule>) -> Vec<isize> {
    pairs
        .into_iter()
        .flat_map(|pair| {
            match pair.as_rule() {
                Rule::FrameSequenceString | Rule::FrameSequence | Rule::FrameSequencePart => {
                    frame_sequence_token_tree_to_frames(pair.into_inner())
                }
                Rule::FrameRange => {
                    let mut pairs = pair.into_inner();
                    let left = frame_to_number(pairs.next().unwrap());
                    let right = frame_to_number(pairs.next().unwrap());

                    // Do we have an `@`?
                    if pairs.next().is_some() {
                        let pair = pairs.next().unwrap();
                        match pair.as_rule() {
                            Rule::PositiveNumber => {
                                let step = frame_to_number(pair);

                                match left.cmp(&right) {
                                    Ordering::Less => {
                                        (left..right + 1).step_by(step as _).collect::<Vec<_>>()
                                    }
                                    Ordering::Greater => (right..left + 1)
                                        .rev()
                                        .step_by(step as _)
                                        .collect::<Vec<_>>(),
                                    Ordering::Equal => vec![left],
                                }
                            }
                            Rule::BinarySequenceSymbol => binary_sequence((left, right)),
                            _ => unreachable!(),
                        }
                    } else if left < right {
                        (left..right + 1).collect::<Vec<_>>()
                    } else if right < left {
                        (right..left + 1).rev().collect::<Vec<_>>()
                    }
                    // left == right
                    else {
                        vec![left]
                    }
                }
                Rule::Frame => vec![frame_to_number(pair)],
                _ => vec![],
            }
        })
        .collect::<Vec<_>>()
}

fn remove_duplicates(elements: Vec<isize>) -> Vec<isize> {
    let mut set = HashSet::<isize>::new();
    elements
        .iter()
        .filter_map(|e| {
            if set.contains(e) {
                None
            } else {
                set.insert(*e);
                Some(*e)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_individual_frames() {
        use crate::parse_frame_sequence;
        let frames = parse_frame_sequence("1,2,3,5,8,13").unwrap();
        assert_eq!([1, 2, 3, 5, 8, 13], frames.as_slice());
    }

    #[test]
    fn test_frame_sequence() {
        use crate::parse_frame_sequence;
        let frames = parse_frame_sequence("10-15").unwrap();
        assert_eq!([10, 11, 12, 13, 14, 15], frames.as_slice());
    }

    #[test]
    fn test_fram_sequence_with_step() {
        use crate::parse_frame_sequence;
        let frames = parse_frame_sequence("10-20@2").unwrap();
        assert_eq!([10, 12, 14, 16, 18, 20], frames.as_slice());
    }

    #[test]
    fn test_frame_sequence_with_step_reversed() {
        use crate::parse_frame_sequence;
        let frames = parse_frame_sequence("42-33@3").unwrap();
        assert_eq!([42, 39, 36, 33], frames.as_slice());
    }

    #[test]
    fn test_binary_frame_sequence() {
        use crate::parse_frame_sequence;
        let frames = parse_frame_sequence("10-20@b").unwrap();
        assert_eq!(
            [10, 20, 15, 12, 17, 11, 13, 16, 18, 14, 19],
            frames.as_slice()
        );
    }
}
