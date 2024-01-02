use std::{rc::Rc, cell::RefCell};
use crate::brainfuck::BfOpBlock;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
  Extern
}

pub trait CompilerImpl {
  fn compile(item: Rc<RefCell<BfOpBlock>>, target: Option<Target>) -> Vec<u8>;
}

pub mod x86_64;

#[cfg(target_arch="x86_64")]
pub use x86_64::Compiler as NativeCompiler;
