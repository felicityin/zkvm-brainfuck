pub mod concurrency;
mod logger;
mod prove;
mod span;
mod tracer;

pub use logger::*;
use p3_field::Field;
pub use prove::*;
pub use span::*;
pub use tracer::*;

use p3_maybe_rayon::prelude::{ParallelBridge, ParallelIterator};

pub const fn indices_arr<const N: usize>() -> [usize; N] {
    let mut indices_arr = [0; N];
    let mut i = 0;
    while i < N {
        indices_arr[i] = i;
        i += 1;
    }
    indices_arr
}

pub fn pad_to_power_of_two<const N: usize, T: Clone + Default>(values: &mut Vec<T>) {
    debug_assert!(values.len() % N == 0);
    let mut n_real_rows = values.len() / N;
    if n_real_rows < 16 {
        n_real_rows = 16;
    }
    values.resize(n_real_rows.next_power_of_two() * N, T::default());
}

/// Pad to a power of two, with an option to specify the power.
//
// The `rows` argument represents the rows of a matrix stored in row-major order. The function will
// pad the rows using `row_fn` to create the padded rows. The padding will be to the next power of
// of two of `size_log_2` is `None`, or to the specified `size_log_2` if it is not `None`. The
// function will panic of the number of rows is larger than the specified `size_log2`
pub fn pad_rows_fixed<R: Clone>(rows: &mut Vec<R>, row_fn: impl Fn() -> R) {
    let nb_rows = rows.len();
    let dummy_row = row_fn();
    rows.resize(next_power_of_two(nb_rows), dummy_row);
}

/// Returns the next power of two that is >= `n` and >= 16.
pub fn next_power_of_two(n: usize) -> usize {
    let mut padded_nb_rows = n.next_power_of_two();
    if padded_nb_rows < 16 {
        padded_nb_rows = 16;
    }
    padded_nb_rows
}

pub fn chunk_vec<T>(mut vec: Vec<T>, chunk_size: usize) -> Vec<Vec<T>> {
    let mut result = Vec::new();
    while !vec.is_empty() {
        let current_chunk_size = std::cmp::min(chunk_size, vec.len());
        let current_chunk = vec.drain(..current_chunk_size).collect::<Vec<T>>();
        result.push(current_chunk);
    }
    result
}

#[inline]
pub fn log2_strict_usize(n: usize) -> usize {
    let res = n.trailing_zeros();
    assert_eq!(n.wrapping_shr(res), 1, "Not a power of two: {n}");
    res as usize
}

pub fn par_for_each_row<P, F>(vec: &mut [F], num_elements_per_event: usize, processor: P)
where
    F: Send,
    P: Fn(usize, &mut [F]) + Send + Sync,
{
    // Split the vector into `num_cpus` chunks, but at least `num_cpus` rows per chunk.
    assert!(vec.len() % num_elements_per_event == 0);
    let len = vec.len() / num_elements_per_event;
    let cpus = num_cpus::get();
    let ceil_div = len.div_ceil(cpus);
    let chunk_size = std::cmp::max(ceil_div, cpus);

    vec.chunks_mut(chunk_size * num_elements_per_event).enumerate().par_bridge().for_each(
        |(i, chunk)| {
            chunk.chunks_mut(num_elements_per_event).enumerate().for_each(|(j, row)| {
                assert!(row.len() == num_elements_per_event);
                processor(i * chunk_size + j, row);
            });
        },
    );
}

/// Returns whether the `BF_DEBUG` environment variable is enabled or disabled.
///
/// This variable controls whether backtraces are attached to compiled circuit programs, as well
/// as whether cycle tracking is performed for circuit programs.
///
/// By default, the variable is disabled.
pub fn bf_debug_mode() -> bool {
    let value = std::env::var("BF_DEBUG").unwrap_or_else(|_| "false".to_string());
    value == "1" || value.to_lowercase() == "true"
}

/// Returns a vector of zeros of the given length. This is faster than vec![F::ZERO; len] which
/// requires copying.
///
/// This function is safe to use only for fields that can be transmuted from 0u32.
pub fn zeroed_f_vec<F: Field>(len: usize) -> Vec<F> {
    debug_assert!(std::mem::size_of::<F>() == 4);

    let vec = vec![0u32; len];
    unsafe { std::mem::transmute::<Vec<u32>, Vec<F>>(vec) }
}
