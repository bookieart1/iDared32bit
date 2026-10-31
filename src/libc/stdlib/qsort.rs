/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! stdlib's qsort

use crate::abi::{CallFromHost, GuestFunction};
use crate::dyld::{export_c_func, FunctionExports};
use crate::mem::MutPtr;
use crate::Environment;

fn qsort(
    env: &mut Environment,
    base: MutPtr<u8>,
    nitems: u32,
    size: u32,
    compar: GuestFunction, // int (*compar)(const void *, const void*))
) {
    qsort_generic(
        env,
        nitems,
        &mut |env, i, j| {
            let i_ptr = base + i * size;
            let j_ptr = base + j * size;
            compar.call_from_host(env, (i_ptr.cast_const(), j_ptr.cast_const()))
        },
        &mut |env: &mut Environment, mut i: u32, mut j: u32| {
            if i == j {
                return;
            }
            if i > j {
                std::mem::swap(&mut i, &mut j);
            }
            let base_slice = env.mem.bytes_at_mut(base, nitems * size);
            let (_, slice_after_first) = base_slice.split_at_mut((i * size) as usize);
            let offset = (j * size - i * size) as usize;
            let (left, right) = slice_after_first.split_at_mut(offset);
            left[..size as usize].swap_with_slice(&mut right[..size as usize]);
        },
    );
}

pub fn qsort_generic<T, Compare, Swap>(
    user_data: &mut T,
    nitems: u32,
    compar: &mut Compare, // int (*compar)(const void *, const void*))
    swap: &mut Swap,
) where
    Compare: FnMut(&mut T, u32, u32) -> i32,
    Swap: FnMut(&mut T, u32, u32),
{
    if nitems < 2 {
        return;
    }

    // Use heapsort so already sorted, reverse-sorted, and duplicate-heavy
    // inputs retain O(n log n) behavior. Guest comparison functions are much
    // more expensive than native comparisons, so qsort's former first-pivot
    // quicksort caused severe stalls on common nearly sorted inputs.
    for root in (0..nitems / 2).rev() {
        sift_down(user_data, compar, swap, root, nitems);
    }
    for end in (1..nitems).rev() {
        swap(user_data, 0, end);
        sift_down(user_data, compar, swap, 0, end);
    }
}

fn sift_down<T, Compare, Swap>(
    user_data: &mut T,
    compar: &mut Compare, // int (*compar)(const void *, const void*))
    swap: &mut Swap,
    mut root: u32,
    end: u32,
) where
    Compare: FnMut(&mut T, u32, u32) -> i32,
    Swap: FnMut(&mut T, u32, u32),
{
    while root < end / 2 {
        let mut child = root * 2 + 1;
        if child + 1 < end && compar(user_data, child, child + 1) < 0 {
            child += 1;
        }
        if compar(user_data, root, child) >= 0 {
            return;
        }
        swap(user_data, root, child);
        root = child;
    }
}

pub const FUNCTIONS: FunctionExports = &[export_c_func!(qsort(_, _, _, _))];

#[cfg(test)]
mod tests {
    use super::qsort_generic;

    struct SortData {
        values: Vec<i32>,
        comparisons: usize,
    }

    fn sort(values: Vec<i32>) -> SortData {
        let mut data = SortData {
            values,
            comparisons: 0,
        };
        let len = data.values.len().try_into().unwrap();
        qsort_generic(
            &mut data,
            len,
            &mut |data, i, j| {
                data.comparisons += 1;
                match data.values[i as usize].cmp(&data.values[j as usize]) {
                    std::cmp::Ordering::Less => -1,
                    std::cmp::Ordering::Equal => 0,
                    std::cmp::Ordering::Greater => 1,
                }
            },
            &mut |data, i, j| data.values.swap(i as usize, j as usize),
        );
        data
    }

    #[test]
    fn orders_values() {
        let data = sort(vec![5, -2, 9, 5, 0, 1, -8]);
        assert_eq!(data.values, [-8, -2, 0, 1, 5, 5, 9]);
    }

    #[test]
    fn bounded_comparisons_for_ordered_inputs() {
        const LEN: i32 = 1356;
        let sorted = sort((0..LEN).collect());
        let reversed = sort((0..LEN).rev().collect());
        let equal = sort(vec![7; LEN as usize]);

        assert!(sorted.comparisons < 50_000, "{}", sorted.comparisons);
        assert!(reversed.comparisons < 50_000, "{}", reversed.comparisons);
        assert!(equal.comparisons < 50_000, "{}", equal.comparisons);
    }
}
