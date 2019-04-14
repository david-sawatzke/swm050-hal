#![no_main]
#![no_std]

#[allow(unused)]
use panic_halt;

use swm050_hal as hal;

use crate::hal::prelude::*;
use crate::hal::swm050;

use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    if let Some(p) = swm050::Peripherals::take() {
        cortex_m::interrupt::free(move |cs| {
            let gpioa = p.GPIOA.split();

            /* (Re-)configure PA5 as output */
            let mut led = gpioa.pa_5.into_push_pull_output(&cs);

            loop {
                /* Turn PA5 on a million times in a row */
                for _ in 0..1_000_000 {
                    led.set_high();
                }
                /* Then turn PA5 off a million times in a row */
                for _ in 0..1_000_000 {
                    led.set_low();
                }
            }
        });
    }

    loop {
        continue;
    }
}
