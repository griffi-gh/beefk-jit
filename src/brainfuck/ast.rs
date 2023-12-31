use std::{collections::HashMap, vec, rc::Rc, cell::RefCell};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
  CellInc(i16),
  CellSet(u8),
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

fn parse_tree_unoptimized(code: &str) -> Rc<RefCell<BfOpBlock>> {
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

/// Gets called recursively on every Master or Loop block\
/// Returns true if any changes were made, in which case this function should be called again\
///
/// Panics:\
///  - If a Unit block is provided\
///  - If it feels like it
fn optimize_tree_recursive(block: Rc<RefCell<BfOpBlock>>) -> bool {
  let mut modified = false;

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
            //Cell difference or absolute value in case is_relative is false
            let mut cell_inc_or_value = 0;
            let mut is_relative = true;
            for effect in effects.iter() {
              match effect {
                Effect::CellInc(n) => {
                  cell_inc_or_value += *n
                },
                Effect::CellSet(v) => {
                  cell_inc_or_value = *v as i16;
                  is_relative = false;
                },
                _ => {
                  if is_relative {
                    if cell_inc_or_value != 0 {
                      opt_effects.push(Effect::CellInc(cell_inc_or_value));
                      cell_inc_or_value = 0;
                    }
                  } else {
                    opt_effects.push(Effect::CellSet(cell_inc_or_value as u8));
                    cell_inc_or_value = 0;
                    //XXX: is this needed? I'm confused
                    is_relative = true;
                  }
                  opt_effects.push(*effect);
                }
              }
            }
            if is_relative {
              if cell_inc_or_value != 0 {
                opt_effects.push(Effect::CellInc(cell_inc_or_value));
              }
            } else {
              opt_effects.push(Effect::CellSet(cell_inc_or_value as u8));
            }
            if *effects != opt_effects {
              modified = true;
            }
            *effects = opt_effects;
            //effects.iter().zip(opt_effects.iter()).all(|(a, b)| a == b);
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
        let keep = !unit.effects.is_empty() || unit.ptr_offset != 0;
        if !keep { modified = true }
        keep
      },
      _ => true
    }
  });

  // If the current block is Master or Loop, and there are consecutive Unit blocks,
  // merge them into the first ones, removing the others
  let mut merge_into: Option<Rc<RefCell<BfOpBlock>>> = None;
  //TODO fix this
  // blocks.retain_mut(|block| {
  //   match &mut *block.borrow_mut() {
  //     BfOpBlock::Unit(unit) => {
  //       if let Some(merge_into) = &merge_into {
  //         let BfOpBlock::Unit(merge_into_unit) = &mut *merge_into.borrow_mut() else {
  //           unreachable!()
  //         };
  //         for (&key, key_effects) in &unit.effects {
  //           merge_into_unit.effects.entry(key + merge_into_unit.ptr_offset).or_default().extend_from_slice(key_effects);
  //         }
  //         merge_into_unit.ptr_offset += unit.ptr_offset;
  //         modified = true;
  //         false
  //       } else {
  //         merge_into = Some(Rc::clone(block));
  //         true
  //       }
  //     },
  //     _ => {
  //       merge_into = None;
  //       true
  //     },
  //   }
  // });

  //Drop borrow_mut binding
  drop(binding);

  //Now, if the current block is loop and contains a single unit block that:
  // - does not change the pointer position
  // - only has a *single* effect that either adds or subs an odd value
  //Turn ourself into a Unit block that sets the cell to 0
  //This optimizes away loops like: [-]+++, and with multi-step optimization should reduce
  //[-]+++ to a single CellSet(3) effect
  //TODO: expand this optimization to moves, aka [->+<]
  let mut new_self = None;
  if let BfOpBlock::Loop(blocks) = &*block.borrow(){
    if blocks.len() == 1 {
      if let BfOpBlock::Unit(unit) = &*blocks[0].borrow() {
        if unit.ptr_offset == 0 && unit.effects.len() == 1 {
          let (&_, effects) = unit.effects.iter().next().unwrap();
          if effects.len() == 1 {
            let effect = &effects[0];
            if let Effect::CellInc(n) = effect {
              if n.abs() % 2 == 1 {
                //HACK: borrow checker workaround:
                new_self = Some(BfOpBlock::Unit(BfUnit {
                  effects: HashMap::from([(0, vec![Effect::CellSet(0)])]),
                  ptr_offset: 0,
                }));
              }
            }
          }
        }
      }
    }
  }
  if let Some(new_self) = new_self {
    //HACK: remove block from optimize_next list
    //since it just got turned into a Unit block, which is not optimizable
    //TODO: generate optimize_next list AFTER this step instead!
    optimize_next.retain(|b| !Rc::ptr_eq(b, &block));
    *block.borrow_mut() = new_self;
    modified = true;
  }

  for optimize_next in optimize_next {
    if optimize_tree_recursive(optimize_next) {
      modified = true;
    }
  }

  modified
}

pub fn parse_tree(code: &str) -> Rc<RefCell<BfOpBlock>> {
  let block = parse_tree_unoptimized(code);
  let mut iterations = 0;
  while optimize_tree_recursive(Rc::clone(&block)) {
    iterations += 1;
  }
  println!("Optimized in {} iteration(s)", iterations);
  block
}

/// Hacky function to pretty-print bf op blocks
pub fn debug_print_tree(block: Rc<RefCell<BfOpBlock>>, indent: usize) {
  let print_ident = |indent: usize| {
    for _ in 0..indent {
      print!("  ");
    }
  };
  match &*block.borrow() {
    BfOpBlock::Master(blocks) => {
      for block in blocks {
        debug_print_tree(Rc::clone(block), indent);
      }
    },
    BfOpBlock::Loop(blocks) => {
      print_ident(indent);
      println!("loop {{");
      for block in blocks {
        debug_print_tree(Rc::clone(block), indent + 1);
      }
      print_ident(indent);
      println!("}}");
    },
    BfOpBlock::Unit(unit) => {
      print_ident(indent);
      println!("unit {{");
      for (offset, effects) in unit.effects.iter() {
        print_ident(indent + 1);
        print!("p[{offset:+}]: ");
        for effect in effects {
          match effect {
            Effect::CellInc(change) => {
              print!("{change:+}; ");
            },
            Effect::CellSet(value) => {
              print!("={value};");
            },
            Effect::Output => {
              print!("output; ");
            },
            Effect::Input => {
              print!("input; ");
            }
          }
        }
        println!();
      }
      if unit.ptr_offset != 0 {
        print_ident(indent + 1);
        println!("p: {:+};", unit.ptr_offset);
      }
      print_ident(indent);
      println!("}}");
    }
  }
}
