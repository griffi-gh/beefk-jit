#![forbid(unsafe_op_in_unsafe_fn)]
#[cfg(not(unix))] compile_error!("lol nah");

use std::{rc::Rc, cell::RefCell};

const BF_CODE: &str ="+>++>+++[->>+<<]++++";

// const CODE: &[u8] = &[0x0f, 0xaf, 0xff, 0x89, 0xf8, 0xc3];

mod jit;
use jit::executable::{Executable, ToFnPtr};

mod brainfuck;
use brainfuck::ast;

fn main() {
  println!("=== Parsing and optimizing bf code...");
  println!("{BF_CODE}");
  let block = ast::parse_tree(BF_CODE);
  ast::debug_print_tree(Rc::clone(&block), 0);

  println!("\n=== Running x86_64 codegen on the master block");
  let mut native_code = jit::compiler::compile_ast(Rc::clone(&block));
  jit::compiler::wrap_compiled(&mut native_code);
  // println!("{:02x?}", bf_code);
  println!("{}",
    native_code.iter()
      .map(|b| format!("{:02x}", b).to_string())
      .collect::<Vec<String>>()
      .join(" ")
  );

  println!("\n=== Running the generated code:");
  let mut bf_memory = Box::new([0u8; 30000]);
  let block = Executable::from(&native_code[..]);
  let fn_ptr: unsafe extern fn(*mut u8) = unsafe { block.to_fn_ptr() };
  unsafe { fn_ptr(bf_memory[..].as_mut_ptr()) };

  println!("Nyaa~ no segfault! (*＾▽＾)っ✨");
  println!("\n=== bfmem state (showing first 30 bytes)");
  println!("{:02x?}", &bf_memory[0..30]);

  // let mut block = Executable::new(4096);
  // block[0..CODE.len()].copy_from_slice(CODE);
  // let fn_ptr: unsafe extern fn(i32) -> i32 = unsafe { block.to_fn_ptr() };
  // let result = unsafe { fn_ptr(5) };
  // println!("{result}");
}
