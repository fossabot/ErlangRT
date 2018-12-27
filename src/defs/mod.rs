//! Helper module defines types used everywhere in the VM runtime
pub mod sizes;
pub use self::sizes::*;
pub mod exc_type;

use std::{isize, usize};

/// Word is an unsigned machine-register sized word. Do not use for sizes and
/// counters, use usize instead.
pub type Word = usize;
pub type SWord = isize;
pub type Arity = usize;

// pub use term::immediate::SMALL_BITS;

#[cfg(target_pointer_width = "32")]
pub const WORD_BITS: usize = 32;
#[cfg(target_pointer_width = "32")]
pub const WORD_ALIGN_SHIFT: usize = 2;

#[cfg(target_pointer_width = "64")]
pub const WORD_BITS: usize = 64;
#[cfg(target_pointer_width = "64")]
pub const WORD_ALIGN_SHIFT: usize = 3;

/// This bit is set on boxed values which are CP pointers
pub const HIGHEST_BIT_CP: Word = 1 << (WORD_BITS - 1);

pub const WORD_BYTES: Word = WORD_BITS / 8;

pub const MAX_XREGS: Word = 256;
pub const MAX_FPREGS: Word = 8;

pub struct Reductions {}
impl Reductions {
  /// How many function-calls/heavier opcodes we process before the process will
  /// be scheduled out and give the way to other processes in the queue.
  pub const DEFAULT: isize = 200;

  // Costs are taken for different operations
  //

  /// Fetch is base "tax" for fetching an opcode and dispatching to its handler
  pub const FETCH_OPCODE_COST: isize = 1;
}

// / For n bytes calculate how many words are required to store this
//#[inline]
// pub const fn storage_bytes_to_words(n: Word) -> WordSize {
//  WordSize::new((n + WORD_BYTES - 1) / WORD_BYTES)
//}

#[allow(dead_code)]
#[inline]
pub fn pointer_diff<T>(a: *const T, b: *const T) -> usize {
  assert!(a >= b);
  let an = a as usize;
  let bn = b as usize;
  (an - bn) / core::mem::size_of::<T>()
}
