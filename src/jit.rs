pub struct Executable {
  memptr: *mut u8,
  size: usize
}

impl Executable {
  pub fn new(size: usize) -> Self {
    let page_size = page_size::get();
    let mut memptr: *mut libc::c_void = std::ptr::null_mut();
    unsafe {
      libc::posix_memalign(&mut memptr, page_size, size);
      libc::mprotect(memptr, size, libc::PROT_EXEC | libc::PROT_READ | libc::PROT_WRITE);
      libc::memset(memptr, 0xc3, size);
    }
    let memptr = memptr as *mut u8;
    Self { memptr, size }
  }
  
  pub fn get(&self) -> &[u8] {
    unsafe { std::slice::from_raw_parts(self.memptr, self.size) }
  }

  pub fn get_mut(&mut self) -> &mut [u8] {
    unsafe { std::slice::from_raw_parts_mut(self.memptr, self.size) }
  }

  pub unsafe fn execute<R>(&self) -> R {
    let f: fn() -> R = unsafe { std::mem::transmute(self.memptr) };
    f()
  }
}

impl Drop for Executable {
  fn drop(&mut self) {
    unsafe {
      libc::free(self.memptr as *mut libc::c_void);
    }
  }
}
