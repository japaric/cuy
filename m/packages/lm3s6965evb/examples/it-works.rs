//! can boot a Cortex-M3 system
//@ runner: $RUNNER
//@ target: $TARGET

#![no_std]
#![no_main]

m_rt::entry!(main);

fn main() -> ! {
    sh::exit()
}
