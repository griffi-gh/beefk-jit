#![forbid(unsafe_op_in_unsafe_fn)]

#[cfg(not(unix))]
compile_error!("non-unix-like systems are not supported");

use std::{rc::Rc, fs, env, time::Instant};

mod jit;
mod brainfuck;
mod compiler;

use jit::{Executable, ToFnPtr};
use compiler::{CompilerImpl, Target};

fn main() {
  let bf_code = fs::read_to_string(env::args().nth(1).unwrap()).expect("file read error");

  println!("=== Parsing and optimizing bf code...");
  println!("{bf_code}");
  let block = brainfuck::parse_tree(&bf_code);
  brainfuck::debug_print_tree(Rc::clone(&block), 0);

  println!("\n=== Running x86_64 codegen on the master block");
  let native_code = compiler::NativeCompiler::compile(
    Rc::clone(&block),
    Some(Target::Extern)
  );
  println!("{}",
    native_code.iter()
      .map(|b| format!("{:02x}", b).to_string())
      .collect::<Vec<String>>()
      .join(" ")
  );

  println!("\n=== Running the generated code:");
  let mut bf_memory = [0u8; 0xffff];
  let block = Executable::from(&native_code[..]);
  let fn_ptr: unsafe extern fn(*mut u8) = unsafe { block.to_fn_ptr() };
  let instant = Instant::now();
  unsafe { fn_ptr(bf_memory[0x100..].as_mut_ptr()) };
  let elapsed = instant.elapsed().as_secs_f64();

  println!("\nNyaa~ no segfault! (*＾▽＾)っ✨");
  println!("Execution time: {:.3}ms", elapsed * 1000.0);
  println!("\n=== bfmem state (showing first 30 bytes)");
  println!("{:02x?}", &bf_memory[0x100..(0x100 + 30)]);
}
