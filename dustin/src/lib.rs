#![feature(test)]
extern crate test;

// Rust makes it hard to
//  * generate "scratch space" full of T's.
//  * not move too many times
//  * solution for T: Copy is easy; T: Clone is harder, arbitrary T...? without unsafe?
//
// TODO:
//  * parallelize with a thread pool?

use std::iter;
use std::ptr;
use std::slice;

pub trait Mergesort {
    fn mergesort(&mut self);
}

/// An iterator that picks indexes from two other iterators, based on which
/// has the smaller value in data.
struct Join<'a, T>
where T: Ord
{
    data: &'a [T],
    left: Box<iter::Peekable<Merge<'a, T>>>,
    right: Box<iter::Peekable<Merge<'a, T>>>,
}

impl<'a, T> Iterator for Join<'a, T>
where T: Ord
{
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        let left = self.left.peek();
        let right = self.right.peek();

        if let Some(l) = left {
            if let Some(r) = right {
                if self.data[*l] < self.data[*r] {
                    self.left.next()
                } else {
                    self.right.next()
                }
            } else {
                self.left.next()
            }
        } else {
            if let Some(_) = right {
                self.right.next()
            } else {
                None
            }
        }
    }
}

/// Either a merge of two smaller chunks, or a simple iterator over a sequence of
/// length 1 or 0.
enum Merge<'a, T>
where T: Ord
{
    Join(Join<'a, T>),
    Iter(slice::Iter<'a, usize>),
}

impl<'a, T> Iterator for Merge<'a, T>
where T:Ord
{
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        match self {
            Merge::Join(ref mut iter) => iter.next(),
            Merge::Iter(ref mut iter) => match iter.next() {
                None => None,
                Some(&idx) => Some(idx),
            },
        }
    }
}

/// Construct an iterator that will iterate over indexes in the range `indexes`
/// in order by the values they reference in data.
fn merge<'a, T>(data: &'a [T], indexes: &'a [usize]) -> Merge<'a, T>
where T: Ord
{
    let len = indexes.len();
    if len <= 1 {
        Merge::Iter(indexes.iter())
    } else {
        let midpoint = len / 2;

        Merge::Join(Join {
            data,
            left: Box::new(merge(data, &indexes[..midpoint]).peekable()),
            right: Box::new(merge(data, &indexes[midpoint..]).peekable()),
        })
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
        let indexes: Vec<usize> = (0..self.len()).collect();
        let indexes: Vec<usize> = merge(self, &indexes[..]).collect();

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
