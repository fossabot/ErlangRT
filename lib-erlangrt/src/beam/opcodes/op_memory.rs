use crate::{
  beam::disp_result::DispatchResult,
  emulator::{process::Process, runtime_ctx::Context},
  fail::{RtErr, RtResult},
  term::value::Term,
};
use crate::defs::sizes::WordSize;

/// Shared code for stack checks and allocations with an optional heap check.
#[inline]
fn gen_alloc(
  ctx: &mut Context,
  curr_p: &mut Process,
  stack_need: WordSize,
  heap_need: WordSize,
  live: usize,
  zero: bool,
) -> RtResult<()> {
  ctx.live = live;

  let hp = &mut curr_p.heap;

  if !hp.heap_has_available(heap_need) {
    return Err(RtErr::HeapIsFull("heap::gen_alloc"));
  }

  if hp.stack_have(stack_need + WordSize::one()) {
    // Stack has enough words, we can allocate unchecked
    if stack_need.words() > 0 {
      hp.stack_alloc_unchecked(stack_need, zero);
    }
    hp.stack_push_lterm_unchecked(ctx.cp.to_cp_term());
  } else {
    // Stack has not enough, invoke GC and possibly fail
    return Err(RtErr::HeapIsFull("heap::gen_alloc/stack"));
  }
  Ok(())
}

// Allocate `need` words on stack, in case of GC use `live` amount of registers.
// Structure: allocate_zero(need:int, live:int)
define_opcode!(_vm, ctx, curr_p,
  name: OpcodeAllocateZero, arity: 2,
  run: {
    gen_alloc(ctx, curr_p, WordSize::new(stack_need), WordSize::new(0), live, true)?;
    Ok(DispatchResult::Normal)
  },
  args: usize(stack_need), usize(live),
);

// Allocate `need` words on stack, in case of GC use `live` amount of registers.
define_opcode!(_vm, ctx, curr_p,
  name: OpcodeAllocate, arity: 2,
  run: {
    gen_alloc(ctx, curr_p, WordSize::new(stack_need), WordSize::new(0), live, false)?;
    Ok(DispatchResult::Normal)
  },
  args: usize(stack_need), usize(live),
);

// Allocate `stack_need` words on stack, check that there's available
// `heap_need` words on heap, in case of GC use `live` amount of registers.
// Structure: allocate_heap_zero(stack_need:int, heap_need: int, live:int)
define_opcode!(_vm, ctx, curr_p,
  name: OpcodeAllocateHeapZero, arity: 3,
  run: {
    gen_alloc(ctx, curr_p, WordSize::new(stack_need), WordSize::new(heap_need), live, true)?;
    Ok(DispatchResult::Normal)
  },
  args: usize(stack_need), usize(heap_need), usize(live),
);

// Allocate `need` words on heap, in case of GC use `live` amount of registers.
define_opcode!(_vm, ctx, curr_p,
  name: OpcodeAllocateHeap, arity: 3,
  run: {
    gen_alloc(ctx, curr_p, WordSize::new(stack_need), WordSize::new(heap_need), live, false)?;
    Ok(DispatchResult::Normal)
  },
  args: usize(stack_need), usize(heap_need), usize(live),
);

// Pop `cp` from the top of the stack and then deallocate additional `n_free`
// words from the stack.
// Structure: deallocate(n:int)
define_opcode!(_vm, ctx, curr_p,
  name: OpcodeDeallocate, arity: 1,
  run: {
    ctx.set_cp(curr_p.heap.stack_deallocate(free));
    Ok(DispatchResult::Normal)
  },
  args: usize(free),
);

// Check that there are `heap_need` words available on heap, otherwise run the
// GC using `live` amount of registers as a part of root set.
// Arg 'live' will be used for gc.
// Structure: test_heap(heap_need:int, live:int)
define_opcode!(_vm, _ctx, curr_p,
  name: OpcodeTestHeap, arity: 2,
  run: {
    curr_p.heap.ensure_size(WordSize::new(heap_need))?;
    Ok(DispatchResult::Normal)
  },
  args: usize(heap_need), IGNORE(live),
);

// Reduce the stack usage by N words, keeping CP on top of the stack.
// Remaining value is used for?
// Structure: trim(N:smallint, Remaining:smallint)
define_opcode!(_vm, _ctx, curr_p,
  name: OpcodeTrim, arity: 2,
  run: { Self::trim(curr_p, n_trim) },
  args: usize(n_trim), IGNORE(remaining),
);

impl OpcodeTrim {
  #[inline]
  pub fn trim(curr_p: &mut Process, n_trim: usize) -> RtResult<DispatchResult> {
    let hp = &mut curr_p.heap;
    let tmp_cp = hp.stack_deallocate(n_trim);
    // assume that after trimming the cp will fit back on stack just fine
    hp.stack_push_lterm_unchecked(tmp_cp);
    Ok(DispatchResult::Normal)
  }
}

// Set Y-register on stack to NIL, offset is encoded inside the term.
// Structure: init(yreg:special_yregister)
define_opcode!(_vm, ctx, curr_p,
  name: OpcodeInit, arity: 1,
  run: {
    curr_p.heap.set_y(y.get_reg_value(), Term::nil())?;
    Ok(DispatchResult::Normal)
  },
  args: yreg(y),
);
