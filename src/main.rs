#![forbid(unsafe_op_in_unsafe_fn)]

#[cfg(not(unix))]
compile_error!("lol nah");

mod jit;
mod bf;

use jit::exe::{Executable, ToFnPtr};

const CODE: &[u8] = &[0x0f, 0xaf, 0xff, 0x89, 0xf8, 0xc3];

fn main() {
  println!("Testing bf parsing and optimization:");
  println!("{:#?}", bf::parse_optimize("++++>--<[->+<]"));
  println!("Testing jit:");
  let mut block = Executable::new(4096);
  block[0..CODE.len()].copy_from_slice(CODE);
  let fn_ptr: unsafe extern fn(i32) -> i32 = unsafe { block.to_fn_ptr() };
  let result = unsafe { fn_ptr(5) };
  println!("{result}");
}
