use std::{collections::HashMap, borrow::BorrowMut};

#[derive(Clone, Copy, Debug)]
pub enum Effect {
  CellInc(i16),
  //CellSet(u8),
  Output,
  Input,
}

#[derive(Default, Debug)]
pub struct BfOpBlock {
  pub effects: HashMap<isize, Vec<Effect>>,
  pub ptr_offset: isize,
  //pub begin: Option<usize>,
  //pub terminates: bool,
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
      '[' => {
        blocks.push(BfOpBlock::default());
      }
      _ => ()
    }
  }
  blocks
}

fn optimize(blocks: &mut Vec<BfOpBlock>) {
  for block in blocks {
    //Optimize block effects
    for (&offset, effects) in block.effects.iter_mut() {
      //Collapse all consecutive CellInc effects into one
      {
        let mut opt_effects = Vec::with_capacity(effects.len());
        let mut cell_inc = 0;
        for effect in effects.iter() {
          match effect {
            Effect::CellInc(n) => cell_inc += *n,
            _ => {
              if cell_inc != 0 {
                opt_effects.push(Effect::CellInc(cell_inc));
                cell_inc = 0;
              }
              opt_effects.push(*effect);
            }
          }
        }
        if cell_inc != 0 {
          opt_effects.push(Effect::CellInc(cell_inc));
        }
        *effects = opt_effects;
        //effects.shrink_to_fit();
      }
    }
  }
}

pub fn parse_optimize(code: &str) -> Vec<BfOpBlock> {
  let mut blocks = parse(code);
  optimize(&mut blocks);
  blocks
}
