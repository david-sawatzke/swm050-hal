//! API for delays with the timer
//!
//! Please be aware of potential overflows.
//!
//! # Example
//!
//! TODO Look in the `examples/` directory

use cast::{u16, u32};
use core::ops::Deref;

use crate::syscon::{ClockEnable, Syscon};
use crate::timers::TimerRegisterBlock;
use embedded_hal::blocking::delay::{DelayMs, DelayUs};

#[derive(Clone, Copy)]
pub struct Delay {
    pub(crate) scale: u32,
    pub(crate) timer: *const TimerRegisterBlock,
    pub(crate) countdown: Option<(u32, u32)>,
}

// NOTE(unsafe) This only reads
unsafe impl Sync for Delay {}
// NOTE(unsafe) This only reads
unsafe impl Send for Delay {}

impl Delay {
    pub fn new<TIMER>(timer: TIMER, syscon: &mut Syscon) -> Delay
    where
        TIMER: Deref<Target = TimerRegisterBlock> + ClockEnable,
    {
        TIMER::enable(syscon);
        assert!(syscon.clocks.timsclk().0 >= 1_000_000);
        let scale = syscon.clocks.timsclk().0 / 1_000_000;

        // Count to the highest possible value
        unsafe { timer.tarval.write(|w| w.bits(0xFFFFFFFF)) };
        // Start counting
        timer.ctrl.write(|w| w.ena().set_bit());
        Delay {
            timer: &(*timer),
            scale,
            countdown: None,
        }
    }
}

impl DelayMs<u32> for Delay {
    // At 48 MHz, calling delay_us with ms * 1_000 directly overflows at 0x15D868 (just over the max u16 value)
    fn delay_ms(&mut self, mut ms: u32) {
        const MAX_MS: u32 = 0x0000_FFFF;
        while ms != 0 {
            let current_ms = if ms <= MAX_MS { ms } else { MAX_MS };
            self.delay_us(current_ms * 1_000);
            ms -= current_ms;
        }
    }
}

impl DelayMs<u16> for Delay {
    fn delay_ms(&mut self, ms: u16) {
        self.delay_us(u32::from(ms) * 1_000);
    }
}

impl DelayMs<u8> for Delay {
    fn delay_ms(&mut self, ms: u8) {
        self.delay_ms(u16(ms));
    }
}

impl DelayUs<u32> for Delay {
    fn delay_us(&mut self, us: u32) {
        let ticks = us * self.scale;

        let start_count = unsafe { (*(self.timer)).curval.read().bits() };

        while (unsafe {
            (*(self.timer))
                .curval
                .read()
                .bits()
                .wrapping_sub(start_count)
        }) < ticks
        {}
    }
}

impl DelayUs<u16> for Delay {
    fn delay_us(&mut self, us: u16) {
        self.delay_us(u32(us))
    }
}

impl DelayUs<u8> for Delay {
    fn delay_us(&mut self, us: u8) {
        self.delay_us(u32(us))
    }
}
