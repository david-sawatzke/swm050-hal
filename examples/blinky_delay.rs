#![no_main]
#![no_std]

#[allow(unused)]
use panic_halt;

use swm050_hal as hal;

use crate::hal::delay::Delay;
use crate::hal::prelude::*;
use crate::hal::swm050;

use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    if let Some(p) = swm050::Peripherals::take() {
        cortex_m::interrupt::free(move |cs| {
            let gpioa = p.GPIOA.split();

            let mut syscon = p.SYS.configure().freeze();
            /* (Re-)configure PA5 as output */
            let mut led = gpioa.pa_5.into_push_pull_output(&cs);

            /* Get delay provider */
            let mut delay = Delay::new(p.TMRSE1, &mut syscon);
            loop {
                led.set_high();
                delay.delay_ms(1_000_u16);
                led.set_low();
                delay.delay_ms(1_000_u16);
            }
        });
    }

    loop {
        continue;
    }
}
