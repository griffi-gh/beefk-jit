use std::{collections::HashMap, vec, rc::Rc, cell::RefCell};

#[derive(Clone, Copy, Debug)]
pub enum Effect {
  CellInc(i16),
  //CellSet(u8),
  Output,
  Input,
}

#[derive(Clone, Debug, Default)]
pub struct BfUnit {
  pub effects: HashMap<isize, Vec<Effect>>,
  pub ptr_offset: isize,
}

#[derive(Clone, Debug)]
pub enum BfOpBlock {
  Master(Vec<Rc<RefCell<BfOpBlock>>>),
  Loop(Vec<Rc<RefCell<BfOpBlock>>>),
  Unit(BfUnit),
}

fn parse_unoptimized(code: &str) -> Rc<RefCell<BfOpBlock>> {
  let master = Rc::new(RefCell::new(BfOpBlock::Master(vec![])));
  let mut stack = vec![];
  let mut current = Rc::clone(&master);
  let mut unit = BfUnit::default();

  let push_unit = |current: &mut BfOpBlock, unit: BfUnit| {
    match current {
      BfOpBlock::Master(blocks) | BfOpBlock::Loop(blocks) => {
        blocks.push(Rc::new(RefCell::new(BfOpBlock::Unit(unit))));
      },
      _ => unreachable!()
    }
  };

  for token in code.chars() {
    match token {
      '-' | '+' => {
        let change = match token {
          '+' => 1,
          '-' => -1,
          _ => unreachable!()
        };
        unit.effects
          .entry(unit.ptr_offset)
          .or_insert(vec![])
          .push(Effect::CellInc(change));
      },
      '<' | '>' => {
        let change = match token {
          '>' => 1,
          '<' => -1,
          _ => unreachable!()
        };
        unit.ptr_offset += change;
      },
      ',' => {
        unit.effects
          .entry(unit.ptr_offset)
          .or_insert(vec![])
          .push(Effect::Input);
      },
      '.' => {
        unit.effects
          .entry(unit.ptr_offset)
          .or_insert(vec![])
          .push(Effect::Output);
      },
      '[' => {
        push_unit(&mut current.borrow_mut(), std::mem::take(&mut unit));
        let new_current = match &mut *current.borrow_mut() {
          BfOpBlock::Master(blocks) | BfOpBlock::Loop(blocks) => {
            let loop_block = Rc::new(RefCell::new(BfOpBlock::Loop(vec![])));
            blocks.push(Rc::clone(&loop_block));
            loop_block
          },
          _ => unreachable!()
        };
        stack.push(Rc::clone(&current));
        current = new_current;
      },
      ']' => {
        push_unit(&mut current.borrow_mut(), std::mem::take(&mut unit));
        current = stack.pop().expect("Unmatched ]");
      }
      _ => ()
    }
  }
  push_unit(&mut current.borrow_mut(), unit);
  master
}

fn optimize(block: Rc<RefCell<BfOpBlock>>) {
  let mut binding = block.borrow_mut();
  let blocks = match &mut *binding {
    BfOpBlock::Master(blocks) | BfOpBlock::Loop(blocks) => blocks,
    _ => unreachable!()
  };

  let mut optimize_next = vec![];

  for block in blocks.iter_mut() {
    match &mut *block.borrow_mut() {
      BfOpBlock::Master(_) | BfOpBlock::Loop(_) => {
        optimize_next.push(Rc::clone(block));
      }
      BfOpBlock::Unit(unit) => {
        //Optimize block effects
        for (&_, effects) in unit.effects.iter_mut() {
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
  }

  //Remove empty blocks
  blocks.retain(|block| {
    match &*block.borrow() {
      BfOpBlock::Unit(unit) => {
        !unit.effects.is_empty() || unit.ptr_offset != 0
      },
      _ => true
    }
  });

  drop(binding);

  for optimize_next in optimize_next {
    optimize(optimize_next);
  }
}

//   //Clean up unit blocks that have no effect
//   blocks.retain(|x| {
//     if let BfOpBlock::Unit { effects, ptr_offset } = x {
//       !effects.is_empty() || *ptr_offset != 0
//     } else {
//       true
//     }
//   });
// }


pub fn parse(code: &str) -> Rc<RefCell<BfOpBlock>> {
  let mut block = parse_unoptimized(code);
  optimize(Rc::clone(&block));
  block
}
