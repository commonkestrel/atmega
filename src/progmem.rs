use core::arch::asm;
use core::mem::size_of;
use core::mem::MaybeUninit;

/// Read a single byte from the program memory at the given address.
/// 
/// Essentially a wrapper around the `lpm` instruction
/// 
/// # Safety
/// Caller must ensure that the input address is below the 16 bit max.
/// This is because the ATmega328p does not have the `elpm` instruction.
pub unsafe fn read_byte(addr: *const u8) -> u8 {
    let byte: u8;

    asm!(
        "lpm {}, Z",
        out(reg) byte,
        in("Z") addr,
    );
    
    byte
}

pub unsafe fn read_bytes_raw(addr: *const u8, out: *mut u8, len: u8) {
    asm!(
        "   
            // Load value at Z into temp and post-increment Z
            lpm {1}, Z+
            // Write temp to data memory at X and post-increment X
            st X+, {1}
            // Decrement the loop counter in $0 (len)
            subi {0}, 1
            // Check whether the end has not been reached.
            // If not equal (brNE), jump back 8 bytes, or 4 instructions
            brne -8
        ",
        // The number of bytes to read
        inout(reg) len => _,
        // Temporary register
        out(reg) _,
        // Input address in Z, increments each cycle
        inout("Z") addr => _,
        // Output address in X, increments each cycle
        inout("X") out => _
    )
}

pub unsafe fn read_values_raw<T: Sized>(addr: *const T, out: *mut T, len: u8) {
    let type_size = size_of::<T>();

    // Check if loop is necessary.
    if len == 0 || type_size == 0 {
        return;
    }

    let bytes = type_size * len as usize;
    // Assert that the cast to u8 is safe.
    assert!(bytes <= u8::MAX as usize);
    let bytes = bytes as u8;

    asm!(
        "   
            // Load value at Z into temp and post-increment Z
            lpm {1}, Z+
            // Write temp to data memory at X and post-increment X
            st X+, {1}
            // Decrement the loop counter in $0 (len)
            subi {0}, 1
            // Check whether the end has not been reached.
            // If not equal (brNE), jump back 8 bytes, or 4 instructions
            brne -8
        ",
        // The number of bytes to read
        inout(reg) bytes => _,
        // Temporary register
        out(reg) _,
        // Input address in Z, increments each cycle
        inout("Z") addr => _,
        // Output address in X, increments each cycle
        inout("X") out => _
    )
}

pub unsafe fn read_value<T: Sized>(addr: *const T) -> T {
    let mut buf = MaybeUninit::<T>::uninit();

    let type_size = size_of::<T>();

    for i in 0..=type_size/u8::MAX as usize {
        read_values_raw(addr, buf.as_mut_ptr().offset((i*u8::MAX as usize) as isize), 1);
    }

    buf.assume_init()
}

pub struct ProgMem<T: Sized>(*const T);

impl<T: Sized> ProgMem<T> {
    /// Creates a new instance of `ProgMem` from the address of a constant value.
    /// 
    /// # Safety
    /// Caller must ensure the address is within the program memory address space.
    /// Is assured for values with the attribute `#[link_section = ".progmem.data"]`.
    pub const fn new(inner: *const T) -> ProgMem<T> {
        ProgMem(inner)
    }

    /// Loads the value contained in program memory.
    pub fn load(&self) -> T {
        unsafe { read_value(self.0) }
    }
}

unsafe impl<T: Sized> Send for ProgMem<T> {}
unsafe impl<T: Sized> Sync for ProgMem<T> {}

/// Allows for the storage of statics in the program memory,
/// commonly refered to as "progmem".
/// Stores the given expression into progmem and replaces type `T` with `ProgMem<T>`
/// 
/// # Example
/// ```rust,norun
/// progmem! {
///     static progmem VALUE: u8 = 42;
/// }
/// 
/// fn get_number() -> u8 {
///     VALUE.load()
/// }
/// ```
#[macro_export]
macro_rules! progmem {
    {   
        $(#[$attr:meta])*
        $vis:vis static progmem $name:ident: $ty:ty = $value:expr;
    } => {
        $(#[$attr])*
        $vis static $name: $crate::progmem::ProgMem<$ty> = {
            #[link_section = ".progmem.data"]
            $vis static $name: $ty = $value;
            $crate::progmem::ProgMem::new(
                ::core::ptr::addr_of!($name)
            )
        };
    }
}
