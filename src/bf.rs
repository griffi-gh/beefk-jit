use std::{collections::HashMap, borrow::BorrowMut};

pub enum Effect {
  CellInc(i16),
  CellSet(u8),
  Output,
  Input,
}

#[derive(Default)]
pub struct BfOpBlock {
  pub effects: HashMap<isize, Vec<Effect>>,
  pub ptr_offset: isize,
  pub begin: Option<usize>,
}

fn parse(code: &str) -> Vec<BfOpBlock> {
  let mut blocks = vec![BfOpBlock::default()];
  for token in code.chars() {
    match token {
      '-' | '+' => {
        let change = match token {
          '+' => 1,
          '-' => -1,
          _ => unreachable!()
        };
        let block = blocks.last_mut().unwrap();
        block.effects
          .entry(block.ptr_offset)
          .or_insert(vec![])
          .push(Effect::CellInc(change));
      },
      '<' | '>' => {
        let change = match token {
          '>' => 1,
          '<' => -1,
          _ => unreachable!()
        };
        let block = blocks.last_mut().unwrap();
        block.ptr_offset += change;
      },
      ',' => {
        let block = blocks.last_mut().unwrap();
        block.effects
          .entry(block.ptr_offset)
          .or_insert(vec![])
          .push(Effect::Input);
      },
      '.' => {
        let block = blocks.last_mut().unwrap();
        block.effects
          .entry(block.ptr_offset)
          .or_insert(vec![])
          .push(Effect::Output);
      },
      _ => ()
    }
  }
  blocks
}

fn optimize(blocks: &mut Vec<BfOpBlock>) {
  for block in blocks {

  }
}

pub fn parse_optimize(code: &str) -> Vec<BfOpBlock> {
  let mut blocks = parse(code);
  optimize(&mut blocks);
  blocks
}
