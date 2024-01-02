use std::{rc::Rc, cell::RefCell};
use crate::brainfuck::{BfOpBlock, Effect};
use super::{CompilerImpl, Target};

/// add rbx, imm
fn add_to_rbx(code: &mut Vec<u8>, imm: i32) {
  match imm {
    0 => (), //no-op

    //inc rbx
    1 => {
      // println!("inc rbx");
      code.extend([0x48, 0xff, 0xc3])
    },

    //dec rbx
    -1 => {
      // println!("dec rbx");
      code.extend([0x48, 0xff, 0xcb])
    },

    //add rbx, imm8
    2..=0x7f => {
      // println!("add rbx, {} ;(imm8)", imm as u8);
      code.extend([0x48, 0x83, 0xc3, imm as u8]);
    },

    //sub rbx, imm8
    -0x80..=-2 => {
      // println!("sub rbx, {} ;(imm8)", imm as u8);
      code.extend([0x48, 0x83, 0xeb, (-imm) as u8]);
    }

    //add rbx, imm32
    imm if imm > 0 => {
      // println!("add rbx, {} ;(imm32)", imm);
      code.extend([0x48, 0x81, 0xc3]);
      code.extend(imm.to_le_bytes());
    }

    //sub rbx, imm32
    _ => {
      // println!("sub rbx, {} ;(imm32)", imm);
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
      // println!("inc byte ptr [rbx]");
      code.extend([0xfe, 0x03])
    }

    //inc byte [rbx + offset]
    (_, 1)  => {
      // println!("inc byte ptr [rbx + {}]", offset as u8);
      code.extend([0xfe, 0x43, offset as u8])
    },

    //dec byte [rbx]
    (0, -1) => {
      // println!("dec byte ptr [rbx]");
      code.extend([0xfe, 0x0b])
    },

    //dec byte [rbx + offset]
    (_, -1) => {
      // println!("dec byte ptr [rbx + {}]", offset as u8);
      code.extend([0xfe, 0x4b, offset as u8])
    },

    //add byte [rbx], imm8
    (0, _) => {
      // println!("add byte ptr [rbx], {}", imm as u8);
      code.extend([0x80, 0x03, imm as u8]);
    },

    //add byte [rbx + offset], imm8
    (_, _) => {
      // println!("add byte ptr [rbx + {}], {}", offset as u8, imm as u8);
      code.extend([0x80, 0x43, offset as u8, imm as u8]);
    },
  }
}

/// je rel (near)
fn je32(code: &mut Vec<u8>, rel: i32) {
  // println!("je {:+} ;(NEAR; imm32)", rel);
  code.extend([0x0f, 0x84]);
  code.extend(rel.to_le_bytes());
}

/// je rel (short/near)
fn je(code: &mut Vec<u8>, rel: i32) {
  match rel {
    0 => (), //no-op
    -0x80..=0x7f => {
      // println!("je {:+} ;(SHORT; imm8)", rel);
      code.extend([0x74, rel as u8]);
    },
    _ => je32(code, rel),
  }
}

/// jne rel (short/near) with optional correction for instruction size
fn jne(code: &mut Vec<u8>, mut rel: i32, correct_for_instruction_size: bool) {
  if rel < 0 && correct_for_instruction_size {
    rel -= 2;
  }
  match rel {
    0 => (), //no-op
    -0x80..=0x7f => {
      // println!("jne {:+} ;(SHORT; imm8)", rel);
      code.extend([0x75, rel as u8]);
    },
    _ => {
      rel -= 4;
      // println!("jne {:+} ;(NEAR; imm32)", rel);
      code.extend([0x0f, 0x85]);
      code.extend(rel.to_le_bytes());
    },
  }
}

fn gen_set_cell(code: &mut Vec<u8>, key: i32, value: u8) {
  match value {
    0 => {
      // println!("xor al, al");
      code.extend([0x30, 0xC0]);
    }
    _ => {
      // println!("mov al, 0x{value:02x}");
      code.extend([0xb0, value]);
    }
  }
  match key {
    0 => {
      // println!("mov [rbx], al");
      code.extend([0x88, 0x03]);
    }
    -0x80..=0x7F => {
      // println!("mov [rbx + {}], al; (imm8)", key);
      code.extend([0x88, 0x43, key as u8]);
    }
    _ => {
      // println!("mov [rbx + {}], al; (imm32)", key);
      code.extend([0x88, 0x83]);
      code.extend(key.to_le_bytes());
    }
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
          // println!("; [[[");
          // println!("cmp byte ptr [rbx], 0");
          code.extend([0x80, 0x3b, 0x00]);
          // println!(";loop position is deferred!");
          je32(code, 0); //DEFERRED, *MUST* use JE32 DUE TO CONST SIZE!
        },
        _ => unreachable!()
      }
      let len_after_head = code.len();
      for child in children {
        compile_ast_recursive(Rc::clone(child), code)
      }
      match item {
        BfOpBlock::Master(_) => (),
        BfOpBlock::Loop(_) => {
          // println!("; ]]]");
          // println!("cmp byte ptr [rbx], 0");
          code.extend([0x80, 0x3b, 0x00]);
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
      // println!("; ***");

      let mut keys: Vec<isize> = unit.effects.keys().copied().collect();
      keys.sort();
      //if there's a key that matches final offset, move it to the end
      //This makes Optimized ptrs optimization possible
      //this is ok since all changes to memory within a single block can be considered parallel
      //and thus order doesn't matter
      let mut optimized_ptr = false;
      if unit.ptr_offset != 0 {
        if let Some(idx) = keys.iter().position(|&key| key == unit.ptr_offset) {
          if idx != keys.len() - 1 {
            keys.remove(idx);
            keys.push(unit.ptr_offset);
          }
          optimized_ptr = true;
        }
      }

      //Process keys
      for (idx, &key) in keys.iter().enumerate() {
        // if Optimized ptr, instead of setting rbx AFTER the Unit ends
        // set it BEFORE PROCESSING THE LAST KEY, saving a couple bytes
        let mut key_shift = 0;
        if optimized_ptr && idx == keys.len() - 1 {
          // println!("; optimized:");
          add_to_rbx(code, unit.ptr_offset as i32);
          // Due to the line above,
          // key memory accesses need to be shifted so that [rbx + key] is available at [rbx]
          // XXX: DO NOT ADD DIRECTLY TO KEY, SINCE IT'S ALSO USED TO INDEX INTO HASHMAP!
          key_shift = -key as i32;
        }

        // Materialize effects
        let effects = unit.effects.get(&key).unwrap();
        for effect in effects {
          match effect {
            &Effect::CellSet(value) => {
              gen_set_cell(code, key as i32 + key_shift, value);
            },
            &Effect::CellInc(by) => {
              add_to_ptr_rbx(code, key as i32 + key_shift, by);
            },
            //TODO optimize add
            Effect::Output => {
              // println!(
              //   "\
              //     mov rax, 1 ; OUTPUT \n\
              //     mov rdi, 1 \n\
              //     mov rdx, 1 \n\
              //     mov rsi, rbx \n\
              //     add rsi, {} ;(imm32) \n\
              //     syscall \
              //   ",
              //   key as i32 + key_shift
              // );
              code.extend([
                0x48, 0xC7, 0xC0, 0x01, 0x00, 0x00, 0x00, //mov rax, 1
                0x48, 0xC7, 0xC7, 0x01, 0x00, 0x00, 0x00, //mov rdi, 1
                0x48, 0xC7, 0xC2, 0x01, 0x00, 0x00, 0x00, //mov rdx, 1
                0x48, 0x89, 0xDE, //mov rsi, rbx
                0x48, 0x81, 0xC6, //add rsi, imm32
              ]);
              code.extend((key as i32 + key_shift).to_le_bytes().as_slice());
              code.extend([0x0F, 0x05]); //syscall
            }
            //TODO Effect::Input
            _ => unimplemented!()
          }
        }
      }

      //If not using optimized ptr optimization, just set the rbx AFTER the Unit ends
      if !optimized_ptr {
        add_to_rbx(code, unit.ptr_offset as i32);
      }
    }
  }
}

fn compile_ast(item: Rc<RefCell<BfOpBlock>>) -> Vec<u8> {
  let mut code = vec![];
  compile_ast_recursive(item, &mut code);
  code
}

fn wrap_extern(code: &mut Vec<u8>) {
  //mov rbp, rdi; at start
  code.reserve(4);
  code.insert(0, 0xfb);
  code.insert(0, 0x89);
  code.insert(0, 0x48);
  //ret; at the end
  code.push(0xC3);
}

pub struct Compiler;
impl CompilerImpl for Compiler {
  fn compile(item: Rc<RefCell<BfOpBlock>>, target: Option<super::Target>) -> Vec<u8> {
    let mut code = compile_ast(item);
    if target == Some(Target::Extern) {
      wrap_extern(&mut code)
    }
    code
  }
}
