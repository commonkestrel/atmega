//! Allows for the storage of constants in program memory, often refered to as progmem.

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

/// Copy bytes at the given address in program memory to an address in data memory.
/// 
/// # Safety
/// Caller must make sure that `addr` is a valid address in program memory address space,
/// `out` is a valid address in data memory address space that is already allocated.
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

/// Read values stored in program memory to an allocated address in data memory.
/// 
/// # Safety
/// Caller must make sure that `addr` is a valid address in program memory address space,
/// and `out` is a valid address in data memory address space that is already allocated.
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

/// Read a single value of type `T` from an address in the program memory address space.
/// 
/// # Safety
/// Caller must make sure that the address is within the program memory address space,
/// and that there is a variable of type `T` stored there.
pub unsafe fn read_value<T: Sized>(addr: *const T) -> T {
    let mut buf = MaybeUninit::<T>::uninit();

    let type_size = size_of::<T>();

    for i in 0..=type_size/u8::MAX as usize {
        read_values_raw(addr, buf.as_mut_ptr().offset((i*u8::MAX as usize) as isize), 1);
    }

    buf.assume_init()
}

/// A wrapper for an address to a variable stored in program memory.
/// Can be used to read data stored in progmem.
/// Should be instantiated with the `progmem!` macro.
/// 
/// # Example
/// ```rust,no_run
/// progmem! {
///     progmem NUMBER: u8 = 42;
///     progmem SQUARES: [u8; 5] = [0, 1, 4, 9, 16];
/// }
/// ```
/// 
/// # Safety
/// Caller must ensure that the internal address points to a variable
/// stored in the program memory address space. This is assured for
/// variables with the `#[link_section = ".progmem.data"]` attribute.
pub struct ProgMem<T: Sized>(*const T);

impl<T: Sized> ProgMem<T> {
    /// Creates a new instance of `ProgMem` from the address of a constant value.
    /// 
    /// # Safety
    /// Caller must ensure the address is within the program memory address space.
    /// This is assured for values with the `#[link_section = ".progmem.data"]` attribute.
    pub const fn new(inner: *const T) -> ProgMem<T> {
        ProgMem(inner)
    }

    /// Loads the value contained in program memory.
    pub fn load(&self) -> T {
        unsafe { read_value(self.0) }
    }

    /// Read a byte offset from the base of the inner value.
    /// Can be used to read just the value at an index of an array.
    /// 
    /// # Example
    /// ```rust,no_run
    /// progmem! {
    ///     progmem SQUARES: [u8; 5] = [0, 1, 4, 9, 16];
    /// }
    /// 
    /// fn square(x: usize) -> u8 {
    ///     SQUARES.read_byte(x) // Will panic if outside of the array (0-4)
    /// }
    /// ```
    /// 
    /// # Panics
    /// Will panic if the offset is greater than the size of the stored variable.
    /// This is because otherwise this would read data outside of the variable.
    /// This would result in undefined behavior.
    /// 
    /// # Safety
    /// 
    pub fn read_byte(&self, offset: usize) -> u8 {
        // Make sure the memory is within the stored variable.
        assert!(size_of::<T>() > offset);

        unsafe {
            let addr = self.0.offset(offset as isize);
            
            let byte: u8;

            asm!(
                "lpm {}, Z",
                out(reg) byte,
                in("Z") addr,
            );

            byte
        }
    }
}

unsafe impl<T: Sized> Send for ProgMem<T> {}
unsafe impl<T: Sized> Sync for ProgMem<T> {}

/// Allows for the storage of statics in the program memory,
/// commonly refered to as "progmem".
/// 
/// Stores the given expression into progmem and replaces type `T` with `ProgMem<T>`
/// 
/// # Examples
/// Basic storage and load of a constant in progmem:
/// ```rust,no_run
/// progmem! {
///     progmem VALUE: u8 = 42;     
/// }
/// 
/// fn get_number() -> u8 {
///     VALUE.load()
/// }
/// ```
/// Storage of multiple values:
/// ```rust,no_run
/// progmem! {
///     progmem THREE: usize = 3;
///     progmem CUBES: [u8; 5] = [0, 1, 8, 27, 64];
///     progmem MAYBE: Option<bool> = None;
/// }
/// ```
/// Usage of arrays in progmem:
/// ```rust,no_run
/// progmem!{
///     progmem SQUARES: [usize; 5] = [0, 1, 4, 9, 16];
/// }
/// 
/// fn square(x: usize) -> usize {
///     SQUARES.read_byte(x) // Will panic if outside of the array (0-4)
/// }
/// ```
#[macro_export]
macro_rules! progmem {
    {   
        $(
            $(#[$attr:meta])*
            $vis:vis progmem $name:ident: $ty:ty = $value:expr;
        )*
    } => {
        $(
            $(#[$attr])*
            $vis static $name: $crate::progmem::ProgMem<$ty> = {
                #[link_section = ".progmem.data"]
                $vis static $name: $ty = $value;
                $crate::progmem::ProgMem::new(
                    ::core::ptr::addr_of!($name)
                )
            };
        )*
    }
}
