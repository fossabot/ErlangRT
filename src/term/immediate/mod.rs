//!
//! Low level term representation for compact heap storage
//!
//! Term bits are: `.... .... ..bb aaPP`
//!
//! Here "PP" are the primary tag, one of `primary_tag::Tag::Immediate`
//! And "aa" with size 2 bits, uses `Immediate1` bits.
//!
//! To use `Immediate2` bits set "aa" to `Immediate1::Immed2` and set "bb" to the
//!    desired value from `Immediate2` enum.
//!
mod imm1;
mod imm2;
mod imm3;

use term::primary;
use defs;
use defs::Word;

use std::mem::transmute;

pub use self::imm1::*;
pub use self::imm2::*;
pub use self::imm3::*;


//
// Construction
//

/// Create a raw value for a term from atom index
#[inline]
pub fn make_atom_raw(val: Word) -> Word {
  combine_imm2_prefix_and_val(val, IMM2_ATOM_PREFIX)
}

/// Create a raw value for a pid term from process index
#[inline]
pub fn make_pid_raw(pindex: Word) -> Word {
  combine_imm1_prefix_and_val(pindex, IMM1_PID_PREFIX)
}

/// Create a raw smallint value for a term from atom index
#[inline]
pub fn make_small_raw(val: Word) -> Word {
  combine_imm1_prefix_and_val(val, IMM1_SMALL_PREFIX)
}

#[inline]
pub fn make_xreg_raw(x: Word) -> Word {
  assert!(x < defs::MAX_XREGS);
  create_imm3(x, IMM3_XREG_PREFIX)
}

#[inline]
pub fn make_yreg_raw(x: Word) -> Word {
  create_imm3(x, IMM3_YREG_PREFIX)
}

#[inline]
pub fn make_fpreg_raw(x: Word) -> Word {
  assert!(x < defs::MAX_FPREGS);
  create_imm3(x, IMM3_FPREG_PREFIX)
}

#[inline]
pub fn make_label_raw(x: Word) -> Word {
  create_imm3(x, IMM3_LABEL_PREFIX)
}

//
// Checks
//

#[inline]
pub fn is_pid_raw(val: Word) -> bool {
  get_imm1_prefix(val) == IMM1_PID_PREFIX
}

#[inline]
pub fn is_atom_raw(val: Word) -> bool {
  get_imm2_prefix(val) == IMM2_ATOM_PREFIX
}


//
// Testing section
//
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_imm3_tags() {
    let n = IMM3_PREFIX;
    assert_eq!(primary::get(n), primary::Tag::Immediate);
    assert_eq!(get_imm1_tag(n), Immediate1::Immed2);
    assert_eq!(get_imm2_tag(n), Immediate2::Immed3);
  }

  #[test]
  fn test_imm3_new() {
    let label = 0b1000000;
    let n = make_label_raw(label);
    assert_eq!(primary::get(n), primary::Tag::Immediate);
    assert_eq!(get_imm1_tag(n), Immediate1::Immed2);
    assert_eq!(get_imm2_tag(n), Immediate2::Immed3);
    assert_eq!(get_imm3_tag(n), Immediate3::Label);
    assert_eq!(get_imm3_prefix(n), IMM3_PREFIX);
    assert_eq!(imm3_value(n), label);
  }
}
