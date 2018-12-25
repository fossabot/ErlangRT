//! Implements Erlang process, an independent computing unit of Erlang with
//! heap, stack, registers, and message queue.

use core::fmt;

use crate::{
  defs::ExceptionType,
  emulator::{
    code_srv::CodeServer,
    heap::{copy_term, Heap, DEFAULT_PROC_HEAP},
    mailbox::ProcessMailbox,
    mfa::MFArity,
    runtime_ctx, scheduler,
  },
  fail::RtResult,
  term::lterm::*,
};
use crate::emulator::mfa::MFASomething;

fn module() -> &'static str {
  "process: "
}

#[allow(dead_code)]
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum ProcessError {
  None,
  Exception(ExceptionType, LTerm),
}

impl fmt::Display for ProcessError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      ProcessError::None => write!(f, "NoError"),
      ProcessError::Exception(exc_type, exc_reason) => match exc_type {
        ExceptionType::Exit => write!(f, "exit({})", exc_reason),
        ExceptionType::Throw => write!(f, "throw({})", exc_reason),
        ExceptionType::Error => write!(f, "error({})", exc_reason),
      },
    }
  }
}

pub struct Process {
  pub pid: LTerm,
  // parent_pid: LTerm,

  // Scheduling and fail state
  /// Scheduling priority (selects the runqueue when this process is scheduled)
  pub prio: scheduler::Prio,
  /// Current scheduler queue where this process is registered
  pub current_queue: scheduler::Queue,

  // Execution Context, etc.
  /// Runtime context with registers, instruction pointer etc
  pub context: runtime_ctx::Context,
  /// How many X registers in the context are currently used
  // pub live: Word,

  // Memory
  pub heap: Heap,
  pub mailbox: ProcessMailbox,

  // Error handling
  /// Record result of last scheduled timeslice for this process
  /// (updated by the vm loop)
  pub timeslice_result: scheduler::SliceResult,
  pub error: ProcessError,
}

impl Process {
  // Call this only from VM, the new process must be immediately registered
  // in proc registry for this VM
  pub fn new(
    pid: LTerm,
    _parent_pid: LTerm,
    mfarity: &MFArity,
    prio: scheduler::Prio,
    code_server: &mut CodeServer,
  ) -> RtResult<Process> {
    assert!(pid.is_local_pid());
    assert!(_parent_pid.is_local_pid() || _parent_pid == LTerm::nil());

    // Process must start with some code location
    match code_server.lookup_and_load(mfarity) {
      Ok(ip) => {
        let p = Process {
          pid,

          // Scheduling
          prio,
          current_queue: scheduler::Queue::None,
          timeslice_result: scheduler::SliceResult::None,

          // Memory
          heap: Heap::new(DEFAULT_PROC_HEAP),
          mailbox: ProcessMailbox::new(),

          // Execution
          context: runtime_ctx::Context::new(ip),

          error: ProcessError::None,
        };
        Ok(p)
        // Ok(sync::Arc::new(sync::RwLock::new(p)))
      }
      Err(e) => Err(e),
    }
  }

  /// Copy args from mfargs-MFA-something into new process heap and set the
  /// registers to the arguments passed to spawn.
  pub fn set_spawn_args(&mut self, _mfargs: &MFASomething) {
    panic!("notimpl set_spawn_args for process")
  }

  /// Returns true if there was an error or exception during the last timeslice.
  #[inline]
  pub fn is_failed(&self) -> bool {
    self.error != ProcessError::None
  }

  #[allow(dead_code)]
  pub fn jump(
    &mut self,
    mfarity: &MFArity,
    code_server: &mut CodeServer,
  ) -> RtResult<()> {
    // TODO: Find mfa in code server and set IP to it
    match code_server.lookup_and_load(mfarity) {
      Ok(ip) => {
        self.context.ip = ip;
        Ok(())
      }
      Err(e) => Err(e),
    }
  }

  pub fn exception(&mut self, exc: ExceptionType, rsn: LTerm) -> LTerm {
    self.set_error(ProcessError::Exception(exc, rsn))
  }

  /// Sets error state from an opcode or a BIF. VM will hopefully check this
  /// immediately and finish the process or catch the error.
  fn set_error(&mut self, e: ProcessError) -> LTerm {
    panic!("{}{} set_error {}", module(), self.pid, e);
    //    self.error = e;
    //    LTerm::non_value()
  }

  //  pub fn clear_error(&mut self) {
  //    self.error = ProcessError::None;
  //  }

  /// Copy a message and put into process mailbox.
  pub fn deliver_message(&mut self, message: LTerm) {
    let m1 = copy_term::copy_to(message, &mut self.heap);
    self.mailbox.put(m1);
  }

  /// Ugly hack to mut-borrow the context without making borrow checker sad.
  /// We guarantee that this borrow will not outlive the process, or we will pay
  /// the price debugging the SIGSEGV.
  #[inline]
  pub fn get_context_p(&self) -> *mut runtime_ctx::Context {
    let p = &self.context as *const runtime_ctx::Context;
    p as *mut runtime_ctx::Context
  }
}
