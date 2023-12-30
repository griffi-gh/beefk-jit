//! //XXX THIS IS WIP AND CURRENTLY UNUSED!

use super::ast::BfOpBlock;

#[derive(Clone, Copy, Debug)]
pub enum BfilOpcode {
  /// increment cell relative to ptr
  CellInc(isize, i16),
  /// set cell to absolute value
  CellSet(isize, u8),
  /// output cell relative to ptr
  Output(isize),
  /// input to cell relative to ptr
  Input(isize),
  /// position is not guaranteed to be up-to-date until final step!
  LoopStart(usize),
  /// position is not guaranteed to be up-to-date until final step!
  LoopEnd(usize),
}

/// Compile a brainfuck AST into a vector of bfil opcodes
/// # Panics:
/// Panics if the AST root is not a master block
pub fn compile_bfil(master: BfOpBlock) -> Vec<BfilOpcode> {
  assert!(matches!(master, BfOpBlock::Master(_)), "Not a master block");
  let mut opcodes = vec![];
  todo!();
  opcodes
}
