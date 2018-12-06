use emulator::function::{CallableLocation, FunEntry};
use emulator::heap::Heap;
use emulator::mfa::MFArity;
use fail::RtResult;
use rt_defs::{storage_bytes_to_words, Arity, Word};
use term::boxed::{BoxHeader, BOXTYPETAG_CLOSURE};
use term::lterm::*;

use core::ptr;
use fail::Error;
use std::mem::size_of;


const fn module() -> &'static str {
  "closure: "
}


/// Boxed `Closure` is placed on heap and referred via LTerm::p
#[allow(dead_code)]
pub struct Closure {
  pub header: BoxHeader,

  pub mfa: MFArity,
  pub dst: CallableLocation,
  pub nfree: usize, // must be word size to avoid alignment of the following data
  // frozen values follow here in memory after the main fields,
  // first is a field, rest will be allocated as extra bytes after
  pub frozen: LTerm,
}

impl Closure {
  #[inline]
  const fn storage_size(nfree: Word) -> Word {
    storage_bytes_to_words(size_of::<Closure>()) + nfree
  }


  fn new(mfa: MFArity, nfree: usize) -> Closure {
    let arity = Closure::storage_size(nfree) - 1;
    Closure {
      header: BoxHeader::new(BOXTYPETAG_CLOSURE, arity),
      mfa,
      dst: CallableLocation::NeedUpdate,
      nfree: nfree as Arity,
      frozen: LTerm::non_value(),
    }
  }


  pub unsafe fn create_into(hp: &mut Heap, fe: &FunEntry, frozen: &[LTerm]) -> RtResult<LTerm> {
    let n_words = Closure::storage_size(fe.nfree);
    let this = hp.alloc::<Closure>(n_words, false)?;

    assert_eq!(frozen.len(), fe.nfree as usize);
    println!(
      "{}new closure: {} frozen={} nfree={}",
      module(),
      fe.mfa,
      frozen.len(),
      fe.nfree
    );

    ptr::write(this, Closure::new(fe.mfa, fe.nfree));

    assert_eq!(frozen.len(), fe.nfree as usize);
    // step 1 closure forward, which will point exactly at the frozen location
    let dst = this.offset(1);
    ptr::copy(
      frozen.as_ptr() as *const Word,
      dst as *mut Word,
      fe.nfree as usize,
    );

    Ok(LTerm::make_boxed(this))
  }


  pub unsafe fn const_from_term(t: LTerm) -> RtResult<*const Closure> {
    helper_get_const_from_boxed_term::<Closure>(t, BOXTYPETAG_CLOSURE, Error::BoxedIsNotAClosure)
  }


  #[allow(dead_code)]
  pub unsafe fn mut_from_term(t: LTerm) -> RtResult<*mut Closure> {
    helper_get_mut_from_boxed_term::<Closure>(t, BOXTYPETAG_CLOSURE, Error::BoxedIsNotAClosure)
  }
}