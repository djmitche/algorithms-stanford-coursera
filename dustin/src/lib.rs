#![feature(test)]
extern crate test;

// Rust makes it hard to
//  * generate "scratch space" full of T's.
//  * not move too many times
//  * solution for T: Copy is easy; T: Clone is harder, arbitrary T...?
//
// TODO:
//  * parallelize with a thread pool?

use std::iter;
use std::mem;
use std::ptr;

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

/// Reorder the elements in `data` according to `indexes.  The two must have the same size
/// and indexes must be a one-to-one mapping from 0..len to 0..len.  Failure of either of
/// these invariants will cause undefined behavior.
unsafe fn reorder<T>(data: &mut [T], indexes: &[usize]) {
    let len = data.len();
    let mut scratch: Vec<T> = Vec::with_capacity(len);
    scratch.set_len(len);

    // copy data -> scratch so we can use it as a source for copying back
    ptr::copy_nonoverlapping(data.as_ptr(), scratch.as_mut_ptr(), len);

    // copy scratch back to data, applying the index mapping so that elements end
    // up in a sorted order
    for i in 0..len {
        ptr::copy_nonoverlapping(&scratch[indexes[i]], &mut data[i], 1);
    }

    // forget about all of the elements in `scratch` as they have been copied
    // back to `data`
    scratch.set_len(0);
}

impl<T> Mergesort for [T]
where T: Ord {
    fn mergesort(&mut self) {
        // sort into a list of indexes, allowing the many moves to apply only to usize values,
        // and not to the data being sorted
        let mut indexes: Vec<usize> = (0..self.len()).collect();
        let mut scratch: Vec<usize> = unsafe { iter::repeat(mem::uninitialized()).take(self.len()).collect() };
        mergesort(self, &mut indexes, &mut scratch);
        drop(scratch);

        // apply the reordering we've constructed
        unsafe {
            reorder(self, &indexes[..]);
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
        let v = vec![5, 0, 1];
        let (mut correct, mut mergesorted) = (v.clone(), v);
        correct.sort();
        mergesorted.mergesort();
        assert_eq!(correct, mergesorted);
    }

    #[test]
    fn sort_strings_simple() {
        let v = vec!["ghi".to_string(), "def".to_string(), "abc".to_string()];
        let (mut correct, mut mergesorted) = (v.clone(), v);
        correct.sort();
        mergesorted.mergesort();
        assert_eq!(correct, mergesorted);
    }

    proptest! {
        #[test]
        fn sort_ints(v in collection::vec(num::u64::ANY, 0..1000)) {
            let (mut correct, mut mergesorted) = (v.clone(), v);
            correct.sort();
            &mergesorted[..].mergesort();
            assert_eq!(correct, mergesorted);
        }

        #[test]
        fn sort_strings(v in collection::vec("[a-zA-Z]*", 0..100)) {
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
