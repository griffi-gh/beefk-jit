#![forbid(unsafe_op_in_unsafe_fn)]
#[cfg(not(unix))] compile_error!("lol nah");

use std::{rc::Rc, cell::RefCell};

const BF_CODE: &str ="++++>--<[->++<]+++<[>]";

const CODE: &[u8] = &[0x0f, 0xaf, 0xff, 0x89, 0xf8, 0xc3];

mod jit;
use jit::exe::{Executable, ToFnPtr};

mod brainfuck;
use brainfuck::{BfOpBlock, Effect};

/// Hacky function to pretty-print bf op blocks
fn print_bf_block(block: Rc<RefCell<BfOpBlock>>, indent: usize) {
  let print_ident = |indent: usize| {
    for _ in 0..indent {
      print!("  ");
    }
  };
  match &*block.borrow() {
    BfOpBlock::Master(blocks) => {
      for block in blocks {
        print_bf_block(Rc::clone(block), indent);
      }
    },
    BfOpBlock::Loop(blocks) => {
      print_ident(indent);
      println!("loop {{");
      for block in blocks {
        print_bf_block(Rc::clone(block), indent + 1);
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
              println!("{change:+};");
            },
            Effect::Output => {
              println!("output;");
            },
            Effect::Input => {
              println!("input;");
            }
          }
        }
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

fn main() {
  println!("Testing bf parsing and optimization:");
  println!("{BF_CODE}");
  print_bf_block(brainfuck::parse(BF_CODE), 0);

  println!("Testing jit:");
  let mut block = Executable::new(4096);
  block[0..CODE.len()].copy_from_slice(CODE);
  let fn_ptr: unsafe extern fn(i32) -> i32 = unsafe { block.to_fn_ptr() };
  let result = unsafe { fn_ptr(5) };
  println!("{result}");
}
