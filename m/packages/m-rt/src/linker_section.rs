/// An output linker section
#[derive(Debug, PartialEq)]
pub struct LinkerSection {
    lower: usize,
    higher: usize,
}

impl LinkerSection {
    /// the `.vectors` section
    pub fn vectors() -> Self {
        unsafe extern "C" {
            static _vectors_lower: u8;
            static _vectors_higher: u8;
        }

        Self {
            lower: &raw const _vectors_lower as usize,
            higher: &raw const _vectors_higher as usize,
        }
    }

    /// the `.text` section
    pub fn text() -> Self {
        unsafe extern "C" {
            static _text_lower: u8;
            static _text_higher: u8;
        }

        Self {
            lower: &raw const _text_lower as usize,
            higher: &raw const _text_higher as usize,
        }
    }

    /// the `.rodata` section
    pub fn rodata() -> Self {
        unsafe extern "C" {
            static _rodata_lower: u8;
            static _rodata_higher: u8;
        }

        Self {
            lower: &raw const _rodata_lower as usize,
            higher: &raw const _rodata_higher as usize,
        }
    }

    /// the `.data` section
    pub fn data() -> Self {
        unsafe extern "C" {
            static _data_lower: u8;
            static _data_higher: u8;
        }

        Self {
            lower: &raw const _data_lower as usize,
            higher: &raw const _data_higher as usize,
        }
    }

    /// the `.bss` section
    pub fn bss() -> Self {
        unsafe extern "C" {
            static _bss_lower: u8;
            static _bss_higher: u8;
        }

        Self {
            lower: &raw const _bss_lower as usize,
            higher: &raw const _bss_higher as usize,
        }
    }

    /// the `.stack` section
    pub fn stack() -> Self {
        unsafe extern "C" {
            static _stack_lower: u8;
            static _stack_higher: u8;
        }

        Self {
            lower: &raw const _stack_lower as usize,
            higher: &raw const _stack_higher as usize,
        }
    }

    /// Returns the lower address boundary of the linker section
    pub fn lower(&self) -> usize {
        self.lower
    }

    /// Returns the higher address boundary of the linker section
    pub fn higher(&self) -> usize {
        self.higher
    }

    /// Returns the size of the linker section in bytes
    pub fn size(&self) -> usize {
        self.higher - self.lower
    }

    /// Checks whether the linker section contains the given `address`
    pub fn contains(&self, address: usize) -> bool {
        (self.lower..self.higher).contains(&address)
    }
}
