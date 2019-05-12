#![feature(test)]
extern crate test;

// Rust makes it hard to
//  * generate "scratch space" full of T's.
//  * not move too many times
//  * solution for T: Copy is easy; T: Clone is harder, arbitrary T...?
//
// TODO:
//  * parallelize with a thread pool?
//  * unsafe approach to move T's out of the way and back (uninitialized?)

pub trait Mergesort {
    fn mergesort(&mut self);
}

/// Sort the indexes for data, without actually moving the data
fn mergesort<T>(data: &[T], indexes: &mut [usize], scratch: &mut [usize])
where T: Ord {
    let len = indexes.len();
    if len <= 1 {
        return
    }

    assert_eq!(len, scratch.len());
    let midpoint = len / 2;
    mergesort(data, &mut indexes[..midpoint], &mut scratch[..midpoint]);
    mergesort(data, &mut indexes[midpoint..], &mut scratch[midpoint..]);

    let mut i = 0;
    let mut j = midpoint;
    let mut r = 0;
    while i < midpoint && j < len {
        if data[indexes[i]] < data[indexes[j]] {
            scratch[r] = indexes[i];
            i += 1;
        } else {
            scratch[r] = indexes[j];
            j += 1;
        }
        r += 1;
    }

    while i < midpoint {
        scratch[r] = indexes[i];
        i += 1;
        r += 1;
    }

    // we just don't bother copying items [j..] into scratch, instead leaving
    // them in place in indexes
    for i in 0..j {
        indexes[i] = scratch[i];
    }
}

impl<T> Mergesort for [T]
where T: Ord + Copy {
    fn mergesort(&mut self) {
        // we'll sort this data by index first, to avoid too many copies
        let mut indexes: Vec<usize> = (0..self.len()).collect();
        let mut scratch: Vec<usize> = std::iter::repeat(0).take(self.len()).collect();
        mergesort(self, &mut indexes, &mut scratch);

        // make a copy of the input as a source, so we can write
        // directly back to the output
        let copy: Vec<T> = self.iter().map(|e| *e).collect();
        for i in 0..self.len() {
            self[i] = copy[indexes[i]];
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
