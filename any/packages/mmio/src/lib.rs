//! MMIO operations
//!
//! The focus of this library is *memory* safety. These are not concerns at this layer:
//! - race conditions on MMIO registers
//! - aliasing of MMIO "handles"
//!
//! As none of them are relevant to *memory* safety. Those concerns can be deal with
//! using privacy, i.e. the module system, types, synchronization and/or runtime checks at
//! higher abstraction layers.
//!
//! Bitfield access/masking can be layered on top of this library; it's not baked into
//! this library

#![no_std]

use core::marker::PhantomData;
use core::mem;

/// Safe to access read-only MMIO register
pub type SafeRoReg<const BASE_ADDRESS: usize, T> = Reg<BASE_ADDRESS, sealed::SafeReadOnly, T>;

/// Read-write MMIO register
pub type RwReg<const BASE_ADDRESS: usize, T> = Reg<BASE_ADDRESS, sealed::ReadWrite, T>;

/// Safe to access read-write MMIO register
pub type SafeRwReg<const BASE_ADDRESS: usize, T> = Reg<BASE_ADDRESS, sealed::SafeReadWrite, T>;

/// Read-only MMIO contiguous registers
pub type SafeRoRegs<const BASE_ADDRESS: usize, const LEN: usize, T> =
    Regs<BASE_ADDRESS, LEN, sealed::SafeReadOnly, T>;

/// Read-write MMIO contiguous registers
pub type RwRegs<const BASE_ADDRESS: usize, const LEN: usize, T> =
    Regs<BASE_ADDRESS, LEN, sealed::ReadWrite, T>;

/// Safe to access read-write MMIO contiguous registers
pub type SafeRwRegs<const BASE_ADDRESS: usize, const LEN: usize, T> =
    Regs<BASE_ADDRESS, LEN, sealed::SafeReadWrite, T>;

/// A MMIO register "handle"
///
/// A MMIO register is effectively a shared-memory based communication mechanism with a
/// peripheral hence we cannot truly talk about "ownership" of a MMIO register. This
/// handle is effectively just a valid pointer into said shared memory.
pub struct Reg<const BASE_ADDRESS: usize, K, T>
where
    T: RegisterDatum,
{
    _datum: PhantomData<T>,
    _kind: PhantomData<K>,
    offset: usize,
}

impl<const BASE_ADDRESS: usize, K, T> Reg<BASE_ADDRESS, K, T>
where
    T: RegisterDatum,
{
    /// Instantiates a MMIO register handle
    ///
    /// # Safety
    ///
    /// Caller must ensure that the address `BASE + offset` points into a valid MMIO register that
    /// holds a value of type/size `T`
    pub const unsafe fn new(offset: usize) -> Self {
        assert!((BASE_ADDRESS + offset).is_multiple_of(mem::size_of::<T>()));

        Self {
            offset,
            _datum: PhantomData,
            _kind: PhantomData,
        }
    }

    fn ptr(&self) -> *mut T {
        (BASE_ADDRESS + self.offset) as *mut T
    }
}

impl<const BASE_ADDRESS: usize, T> RwReg<BASE_ADDRESS, T>
where
    T: RegisterDatum,
{
    /// Reads the MMIO register
    pub fn read(&self) -> T {
        // SAFETY: constructor ensures this is a valid memory location; the side effect of the
        // read is deemed memory safe
        unsafe { self.ptr().read_volatile() }
    }

    /// Writes to the MMIO register
    ///
    /// # Safety
    ///
    /// Check reference manual for memory safety requirements
    pub unsafe fn write(&self, value: T) {
        // SAFETY: constructor ensures this is a valid memory location
        unsafe { self.ptr().write_volatile(value) }
    }
}

impl<const BASE_ADDRESS: usize, T> SafeRoReg<BASE_ADDRESS, T>
where
    T: RegisterDatum,
{
    /// Reads the MMIO register
    pub fn read(&self) -> T {
        // SAFETY: constructor ensures this is a valid memory location; the side effect of the
        // read is deemed memory safe
        unsafe { self.ptr().read_volatile() }
    }
}

impl<const BASE_ADDRESS: usize, T> SafeRwReg<BASE_ADDRESS, T>
where
    T: RegisterDatum,
{
    /// Reads the MMIO register
    pub fn read(&self) -> T {
        // SAFETY: constructor ensures this is a valid memory location; the side effect of the
        // read is deemed memory safe
        unsafe { self.ptr().read_volatile() }
    }

    /// Writes to the MMIO register
    ///
    /// # Safety
    ///
    /// Check reference manual for memory safety requirements
    pub fn write(&self, value: T) {
        // SAFETY: constructor ensures this is a valid memory location
        unsafe { self.ptr().write_volatile(value) }
    }
}

/// A handle to an array of MMIO registers
pub struct Regs<const BASE_ADDRESS: usize, const LEN: usize, K, T>
where
    T: RegisterDatum,
{
    _datum: PhantomData<T>,
    _kind: PhantomData<K>,
    offset: usize,
}

impl<const BASE_ADDRESS: usize, const LEN: usize, K, T> Regs<BASE_ADDRESS, LEN, K, T>
where
    T: RegisterDatum,
{
    /// Instantiates a MMIO register array handle
    ///
    /// # Safety
    ///
    /// Caller must ensure the pointer into address `BASE + offset` is a valid MMIO register that
    /// holds a value of type/size `T`
    pub const unsafe fn new(offset: usize) -> Self {
        assert!((BASE_ADDRESS + offset).is_multiple_of(mem::size_of::<T>()));

        Self {
            offset,
            _datum: PhantomData,
            _kind: PhantomData,
        }
    }

    /// Returns the register at the given `index`
    pub fn get(&self, index: usize) -> Option<Reg<BASE_ADDRESS, K, T>> {
        if index < LEN {
            Some(Reg {
                _datum: PhantomData,
                _kind: PhantomData,
                offset: self.offset + index * mem::size_of::<T>(),
            })
        } else {
            None
        }
    }

    /// Returns an iterator over the MMIO registers
    pub fn iter(&self) -> impl Iterator<Item = Reg<BASE_ADDRESS, K, T>> {
        // SAFETY: indexing within bounds
        (0..LEN).map(|index| unsafe { self.get(index).unwrap_unchecked() })
    }
}

/// The data stored
pub trait RegisterDatum: Copy + sealed::Sealed {}

impl RegisterDatum for usize {}
impl sealed::Sealed for usize {}

impl RegisterDatum for u32 {}
impl sealed::Sealed for u32 {}

impl RegisterDatum for u8 {}
impl sealed::Sealed for u8 {}

mod sealed {
    pub trait Sealed {}
    pub enum SafeReadOnly {}
    pub enum SafeReadWrite {}
    pub enum ReadWrite {}
}
