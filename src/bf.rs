use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
pub enum Effect {
  CellInc(i16),
  //CellSet(u8),
  Output,
  Input,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum MetaType {
  #[default]
  Normal,
  LoopStart,
  LoopEnd,
}

#[derive(Clone, Debug)]
pub enum BfOpBlock {
  Unit {
    effects: HashMap<isize, Vec<Effect>>,
    ptr_offset: isize,
  },
  LoopStart {
    linkid: Option<usize>,
  },
  LoopEnd {
    linkid: Option<usize>,
  }
}

fn parse_unoptimized(code: &str) -> Vec<BfOpBlock> {
  // let mut counter = 0;
  // let mut loop_uuidstack = vec![];
  let mut blocks = vec![BfOpBlock::Unit {
    effects: HashMap::new(),
    ptr_offset: 0,
  }];
  for token in code.chars() {
    match token {
      '-' | '+' => {
        let change = match token {
          '+' => 1,
          '-' => -1,
          _ => unreachable!()
        };
        let BfOpBlock::Unit{ effects, ptr_offset} = blocks.last_mut().unwrap() else { unreachable!() };
        effects
          .entry(*ptr_offset)
          .or_insert(vec![])
          .push(Effect::CellInc(change));
      },
      '<' | '>' => {
        let change = match token {
          '>' => 1,
          '<' => -1,
          _ => unreachable!()
        };
        let BfOpBlock::Unit{ ptr_offset, .. } = blocks.last_mut().unwrap() else { unreachable!() };
        *ptr_offset += change;
      },
      ',' => {
        let BfOpBlock::Unit{ effects, ptr_offset } = blocks.last_mut().unwrap() else { unreachable!() };
        effects
          .entry(*ptr_offset)
          .or_insert(vec![])
          .push(Effect::Input);
      },
      '.' => {
        let BfOpBlock::Unit{ effects, ptr_offset } = blocks.last_mut().unwrap() else { unreachable!() };
        effects
          .entry(*ptr_offset)
          .or_insert(vec![])
          .push(Effect::Output);
      },
      '[' | ']' => {
        blocks.push(match token {
          '[' => BfOpBlock::LoopStart { linkid: None },
          ']' => BfOpBlock::LoopEnd { linkid: None },
          _ => unreachable!()
        });
        blocks.push(BfOpBlock::Unit {
          effects: HashMap::new(),
          ptr_offset: 0
        });
      },
      _ => ()
    }
  }
  blocks
}

fn optimize(blocks: &mut Vec<BfOpBlock>) {
  for block in blocks.iter_mut() {
    let BfOpBlock::Unit { effects, ptr_offset } = block else {
      continue
    };
    //Optimize block effects
    for (&_, effects) in effects.iter_mut() {
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

  //Clean up unit blocks that have no effect
  blocks.retain(|x| {
    if let BfOpBlock::Unit { effects, ptr_offset } = x {
      !effects.is_empty() || *ptr_offset != 0
    } else {
      true
    }
  });
}

fn link_loops(blocks: &mut [BfOpBlock]) {
  let mut loop_stack = vec![];
  for block_idx in 0..blocks.len() {
    match blocks[block_idx] {
      BfOpBlock::LoopStart { linkid: None } => {
        loop_stack.push(block_idx);
      },
      BfOpBlock::LoopEnd { linkid: None } => {
        let start_idx = loop_stack.pop().unwrap();
        blocks[start_idx] = BfOpBlock::LoopStart { linkid: Some(block_idx) };
        blocks[block_idx] = BfOpBlock::LoopEnd { linkid: Some(start_idx) };
      },
      _ => ()
    }
  }
}

pub fn parse(code: &str) -> Vec<BfOpBlock> {
  let mut blocks = parse_unoptimized(code);
  optimize(&mut blocks);
  link_loops(&mut blocks);
  blocks
}
