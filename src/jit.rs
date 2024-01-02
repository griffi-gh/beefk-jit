pub struct Executable {
  memptr: *mut libc::c_void,
  size: usize
}

impl Executable {
  pub fn new(size: usize) -> Self {
    let memptr;
    unsafe {
      memptr = libc::mmap(
        core::ptr::null_mut(), 
        size, 
        libc::PROT_EXEC | libc::PROT_READ | libc::PROT_WRITE, 
        libc::MAP_PRIVATE | libc::MAP_ANONYMOUS, 
        -1, 0
      );
      assert_ne!(memptr, libc::MAP_FAILED);
      assert_ne!(memptr, core::ptr::null_mut());
      libc::memset(memptr, 0xcc, size);
    };
    Self { memptr, size }
  }

  pub fn from_slice(slice: &[u8]) -> Self {
    let mut new = Self::new(slice.len());
    new.copy_from_slice(slice);
    new
  }

  // pub fn resize(&mut self, size: usize) {
  //   let memptr = unsafe {
  //     libc::mremap(self.memptr, self.size, size, libc::MREMAP_MAYMOVE)
  //   };
  //   assert_ne!(memptr, libc::MAP_FAILED);
  //   assert_ne!(memptr, core::ptr::null_mut());
  //   self.memptr = memptr;
  //   self.size = size;
  // }

  pub fn get(&self) -> &[u8] {
    unsafe { core::slice::from_raw_parts(self.memptr as *const u8, self.size) }
  }

  pub fn get_mut(&mut self) -> &mut [u8] {
    unsafe { core::slice::from_raw_parts_mut(self.memptr as *mut u8, self.size) }
  }
}

impl Drop for Executable {
  fn drop(&mut self) {
    unsafe {
      assert_eq!(libc::munmap(self.memptr, self.size), 0);
    }
  }
}

// ToFnPtr impl

pub trait ToFnPtr<A, F> {
  unsafe fn to_fn_ptr(&self) -> F;
}

macro_rules! to_fn_ptr_impl {
  ($($arg:tt)*) => {
    impl<R, $($arg,)*> ToFnPtr<($($arg,)*), unsafe extern "C" fn($($arg,)*) -> R> for Executable {
      #[inline(always)]
      unsafe fn to_fn_ptr(&self) -> unsafe extern "C" fn($($arg,)*) -> R {
        unsafe { core::mem::transmute::<_, unsafe extern "C" fn($($arg,)*) -> R>(self.memptr) }
      }
    }
  };
}

macro_rules! to_fn_ptr_impl_recursive {
  () => {
    to_fn_ptr_impl!();
  };
  ($arg:tt $($rest:tt)*) => {
    to_fn_ptr_impl!($arg $($rest)*);
    to_fn_ptr_impl_recursive!($($rest)*);
  };
}

to_fn_ptr_impl_recursive!(ARG9 ARG8 ARG7 ARG6 ARG5 ARG4 ARG3 ARG2 ARG1 ARG0);

// misc. impls

impl Clone for Executable {
  fn clone(&self) -> Self {
    let mut new = Self::new(self.size);
    new.copy_from_slice(self.get());
    new
  }
}

impl From<&[u8]> for Executable {
  fn from(slice: &[u8]) -> Self { Self::from_slice(slice) }
}

impl core::ops::Deref for Executable {
  type Target = [u8];
  fn deref(&self) -> &Self::Target { self.get() }
}

impl core::ops::DerefMut for Executable {
  fn deref_mut(&mut self) -> &mut Self::Target { self.get_mut() }
}

impl AsRef<[u8]> for Executable {
  fn as_ref(&self) -> &[u8] { self.get() }
}

impl AsMut<[u8]> for Executable {
  fn as_mut(&mut self) -> &mut [u8] { self.get_mut() }
}

impl core::borrow::Borrow<[u8]> for Executable {
  fn borrow(&self) -> &[u8] { self.get() }
}

impl core::borrow::BorrowMut<[u8]> for Executable {
  fn borrow_mut(&mut self) -> &mut [u8] { self.get_mut() }
}
