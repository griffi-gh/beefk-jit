use std::{rc::Rc, cell::RefCell};
use crate::brainfuck::BfOpBlock;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
  Extern
}

pub trait CompilerImpl {
  fn supported() -> bool;
  fn compile(item: Rc<RefCell<BfOpBlock>>, target: Option<Target>) -> Vec<u8>;
}

pub struct DummyCompiler;
impl CompilerImpl for DummyCompiler {
  fn supported() -> bool { false }
  fn compile(_: Rc<RefCell<BfOpBlock>>, _: Option<Target>) -> Vec<u8> {
    panic!("dummy compiler called")
  }
}

pub mod x86_64;

#[cfg(target_arch="x86_64")]
pub use x86_64::Compiler as NativeCompiler;

#[cfg(not(any(target_arch="x86_64")))]
pub use DummyCompiler as NativeCompiler;
