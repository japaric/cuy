//! can boot a Cortex-M4 system
//@ runner: $RUNNER
//@ target: $TARGET

#![no_std]
#![no_main]

m_rt::entry!(main);

fn main() -> ! {
    sh::exit()
}
