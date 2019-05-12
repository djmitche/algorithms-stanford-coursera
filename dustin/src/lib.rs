#![feature(test)]
extern crate test;

// Rust makes it hard to
//  * generate "scratch space" full of T's.
//  * not move too many times

pub trait Mergesort {
    fn mergesort(&mut self);
}

struct Merge<'a, T>
where T: Ord {
    elements: &'a [T],

    left: Box<Iter<'a, T>>,
    left_cache: Option<usize>,

    right: Box<Iter<'a, T>>,
    right_cache: Option<usize>,
}

enum Iter<'a, T>
where T: Ord {
    None,
    Singleton(&'a [T], usize),
    Merge(Merge<'a, T>),
}

enum Take {
    Left,
    Right,
    Neither,
}

impl<'a, T> Iterator for Iter<'a, T>
where T: Ord {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        match self {
            &mut Iter::None => None,
            &mut Iter::Singleton(_, i) => {
                *self = Iter::None;
                Some(i)
            },
            &mut Iter::Merge(ref mut merge) => {
                if merge.left_cache.is_none() {
                    merge.left_cache = merge.left.next();
                }
                if merge.right_cache.is_none() {
                    merge.right_cache = merge.right.next();
                }

                let take = match merge.left_cache {
                    Some(l) => match merge.right_cache {
                        Some(r) if merge.elements[l] < merge.elements[r] => Take::Left,
                        Some(_) => Take::Right,
                        None => Take::Left,
                    },
                    None => match merge.right_cache {
                        Some(_) => Take::Right,
                        None => Take::Neither,
                    },
                };

                match take {
                    Take::Left => {
                        let rv = merge.left_cache;
                        merge.left_cache = None;
                        rv
                    },
                    Take::Right => {
                        let rv = merge.right_cache;
                        merge.right_cache = None;
                        rv
                    },
                    Take::Neither => None,
                }
            }
        }
    }
}

/// Sort the values in `self`, using scratch space of the same length.
fn index_iter<'a, T>(input: &'a [T], left: usize, right: usize) -> Iter<'a, T>
where T: Ord {
    let len = right - left;

    match len {
        0 => Iter::None,
        1 => Iter::Singleton(input, left),
        _ => {
            let midpoint = left + len / 2;

            Iter::Merge(
                Merge {
                    elements: input,
                    left: Box::new(index_iter(input, left, midpoint)),
                    left_cache: None,
                    right: Box::new(index_iter(input, midpoint, right)),
                    right_cache: None,
                }
            )
        }
    }
}

impl<T> Mergesort for [T]
where T: Ord + Copy {
    fn mergesort(&mut self) {
        // make a copy of the input as a source, so we can write
        // directly back to the output
        let copy: Vec<T> = self.iter().map(|e| *e).collect();
        for (i, j) in index_iter(&copy[..], 0, copy.len()).enumerate() {
            self[i] = copy[j];
        }
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use proptest::{collection, num};
    use test::Bencher;
    use std::fs;
    use super::*;

    #[test]
    fn sort_vec() {
        let v = vec![5, 4, 3, 2, 1];
        let (mut correct, mut mergesorted) = (v.clone(), v);
        correct.sort();
        mergesorted.mergesort();
        assert_eq!(correct, mergesorted);
    }

    proptest! {
        #[test]
        fn sort_slices(v in collection::vec(num::u64::ANY, 0..1000)) {
            let (mut correct, mut mergesorted) = (v.clone(), v);
            correct.sort();
            &mergesorted[..].mergesort();
            assert_eq!(correct, mergesorted);
        }
    }

    #[bench]
    fn bench_lots_of_integers(b: &mut Bencher) {
        println!("hi");
        let mut input = fs::read("some_bytes").unwrap();

        let data = &mut input[..200];
        let mut expected = data.to_vec();
        expected.sort();

        b.iter(|| data.mergesort());

        assert_eq!(data, &expected[..]);
    }
}
