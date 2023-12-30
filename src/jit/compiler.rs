use std::{rc::Rc, cell::RefCell};
use crate::brainfuck::ast::{BfOpBlock, Effect};

/// add rbx, imm
fn add_to_rbx(code: &mut Vec<u8>, imm: i32) {
  match imm {
    0 => (), //no-op

    //inc rbx
    1 => {
      println!("inc rbx");
      code.extend([0x48, 0xff, 0xc3])
    },

    //dec rbx
    -1 => {
      println!("dec rbx");
      code.extend([0x48, 0xff, 0xcb])
    },

    //add rbx, imm8
    2..=0x7f => {
      println!("add rbx, {} ;(imm8)", imm as u8);
      code.extend([0x48, 0x83, 0xc3, imm as u8]);
    },

    //sub rbx, imm8
    -0x80..=-2 => {
      println!("sub rbx, {} ;(imm8)", imm as u8);
      code.extend([0x48, 0x83, 0xeb, (-imm) as u8]);
    }

    //add rbx, imm32
    imm if imm > 0 => {
      println!("add rbx, {} ;(imm32)", imm);
      code.extend([0x48, 0x81, 0xc3]);
      code.extend(imm.to_le_bytes());
    }

    //sub rbx, imm32
    _ => {
      println!("sub rbx, {} ;(imm32)", imm);
      code.extend([0x48, 0x81, 0xeb]);
      code.extend((-imm).to_le_bytes());
    }
  }
}

/// add byte ptr [rbx + offset], imm
fn add_to_ptr_rbx(code: &mut Vec<u8>, offset: i32, imm: i16) {
  match (offset, imm) {
    (_, 0) => (), //no-op

    //inc byte [rbx]
    (0, 1)  => {
      println!("inc byte ptr [rbx]");
      code.extend([0xfe, 0x03])
    }

    //inc byte [rbx + offset]
    (_, 1)  => {
      println!("inc byte ptr [rbx + {}]", offset as u8);
      code.extend([0xfe, 0x43, offset as u8])
    },

    //dec byte [rbx]
    (0, -1) => {
      println!("dec byte ptr [rbx]");
      code.extend([0xfe, 0x0b])
    },

    //dec byte [rbx + offset]
    (_, -1) => {
      println!("dec byte ptr [rbx + {}]", offset as u8);
      code.extend([0xfe, 0x4b, offset as u8])
    },

    //add byte [rbx], imm8
    (0, _) => {
      println!("add byte ptr [rbx], {}", imm as u8);
      code.extend([0x80, 0x03, imm as u8]);
    },

    //add byte [rbx + offset], imm8
    (_, _) => {
      println!("add byte ptr [rbx + {}], {}", offset as u8, imm as u8);
      code.extend([0x80, 0x43, offset as u8, imm as u8]);
    },
  }
}

/// je rel (near)
fn je32(code: &mut Vec<u8>, rel: i32) {
  println!("je {:+} ;(NEAR; imm32)", rel);
  code.extend([0x0f, 0x84]);
  code.extend(rel.to_le_bytes());
}

/// je rel (short/near)
fn je(code: &mut Vec<u8>, rel: i32) {
  match rel {
    0 => (), //no-op
    -0x80..=0x7f => {
      println!("je {:+} ;(SHORT; imm8)", rel);
      code.extend([0x74, rel as u8]);
    },
    _ => je32(code, rel),
  }
}

/// jne rel (short/near) with optional correction for instruction size
fn jne(code: &mut Vec<u8>, mut rel: i32, correct_for_instruction_size: bool) {
  if correct_for_instruction_size {
    rel -= 2;
  }
  match rel {
    0 => (), //no-op
    -0x80..=0x7f => {
      println!("jne {:+} ;(SHORT; imm8)", rel);
      code.extend([0x75, rel as u8]);
    },
    _ => {
      rel -= 4;
      println!("jne {:+} ;(NEAR; imm32)", rel);
      code.extend([0x0f, 0x85]);
      code.extend(rel.to_le_bytes());
    },
  }
}

//TODO: use bfil instead
fn compile_ast_recursive(
  item: Rc<RefCell<BfOpBlock>>,
  code: &mut Vec<u8>,
) {
  let item: &BfOpBlock = &item.borrow();
  match item {
    BfOpBlock::Loop(children) | BfOpBlock::Master(children) => {
      match item {
        BfOpBlock::Master(_) => (),
        BfOpBlock::Loop(_) => {
          println!("cmp DWORD PTR [rbx], 0");
          code.extend([0x83, 0x3b, 0x00]);
          je32(code, 0); //DEFERRED, *MUST* use JE32 DUE TO CONST SIZE!
        },
        _ => unreachable!()
      }
      let len_after_head = code.len();
      for child in children {
        compile_ast_recursive(Rc::clone(child), code)
      }
      match item {
        BfOpBlock::Master(_) => {
          // println!("ret");
          // code.push(0xc3); //ret
        },
        BfOpBlock::Loop(_) => {
          println!("cmp DWORD PTR [rbx], 0");
          code.extend([0x83, 0x3b, 0x00]);
          jne(code, len_after_head as i32 - code.len() as i32, true);
          let len_after_tail = code.len();
          let jp_diff = len_after_tail as i32 - len_after_head as i32;
          //Fullfill defer
          code[(len_after_head - 4)..len_after_head].copy_from_slice(
            jp_diff.to_le_bytes().as_slice()
          );
        },
        _ => unreachable!()
      }
    },
    BfOpBlock::Unit(unit) => {
      let mut keys: Vec<isize> = unit.effects.keys().copied().collect();
      keys.sort();
      for &key in &keys {
        // let prev_key = keys.get((key as usize).wrapping_sub(1)).copied().unwrap_or(0);
        // let ptr_diff = key - prev_key;
        // if ptr_diff != 0 {
        //   add_to_rbx(code, ptr_diff as i32);
        // }
        let effects = unit.effects.get(&key).unwrap();
        for effect in effects {
          match effect {
            &Effect::CellInc(by) => {
              add_to_ptr_rbx(code, key as i32, by);
            },
            _ => unimplemented!()
          }
        }
      }
      //add_to_rbx(code, (unit.ptr_offset - keys.last().copied().unwrap_or(0)) as i32);
      add_to_rbx(code, unit.ptr_offset as i32);
    }
  }
}

pub fn compile_ast(item: Rc<RefCell<BfOpBlock>>) -> Vec<u8> {
  let mut code = vec![];
  compile_ast_recursive(item, &mut code);
  code
}
