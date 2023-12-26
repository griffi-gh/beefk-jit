#![forbid(unsafe_op_in_unsafe_fn)]

#[cfg(not(unix))]
compile_error!("lol nah");

mod jit;
mod bf;

use jit::Executable;

const CODE: &[u8] = &[0x48, 0xc7, 0xc0, 0x69, 0xcc, 0xbb, 0xaa];
fn main() {
  println!("code: {CODE:x?}");
  let mut block = Executable::new(4096);
  block.get_mut()[0..7].copy_from_slice(CODE);
  let x: i32 = unsafe { block.execute() };
  println!("function returned 0x{:x}", x);
}
