#![forbid(unsafe_op_in_unsafe_fn)]
#[cfg(not(unix))] compile_error!("lol nah");

use std::{rc::Rc, cell::RefCell};

const BF_CODE: &str ="++++>--<[->++<]+++<[>]";

const CODE: &[u8] = &[0x0f, 0xaf, 0xff, 0x89, 0xf8, 0xc3];

mod jit;
use jit::executable::{Executable, ToFnPtr};

mod brainfuck;

fn main() {
  println!("Testing bf parsing and optimization:");
  println!("{BF_CODE}");
  brainfuck::debug_print_tree(brainfuck::parse_tree(BF_CODE), 0);

  println!("Testing jit:");
  let mut block = Executable::new(4096);
  block[0..CODE.len()].copy_from_slice(CODE);
  let fn_ptr: unsafe extern fn(i32) -> i32 = unsafe { block.to_fn_ptr() };
  let result = unsafe { fn_ptr(5) };
  println!("{result}");
}
