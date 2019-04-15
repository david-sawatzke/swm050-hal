#![no_main]
#![no_std]

#[allow(unused)]
use panic_halt;

use swm050_hal as hal;

use crate::hal::prelude::*;
use crate::hal::swm050;
use crate::hal::time::Hertz;
use crate::hal::timers::*;

use cortex_m::peripheral::Peripherals;
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    if let (Some(p), Some(_cp)) = (swm050::Peripherals::take(), Peripherals::take()) {
        cortex_m::interrupt::free(move |cs| {
            let gpioa = p.GPIOA.split();

            let mut syscon = p.SYS.configure().freeze();

            /* (Re-)configure PA5 as output */
            let mut led = gpioa.pa_5.into_push_pull_output(&cs);

            /* Get timer */
            let mut timer = Timer::timer(p.TMRSE0, Hertz(1), &mut syscon);
            loop {
                timer.start(Hertz(1));
                led.set_high();
                nb::block!(timer.wait()).unwrap();
                led.set_low();
                nb::block!(timer.wait()).unwrap();
                // Do "pwm"
                timer.start(Hertz(600));
                for _ in 0..200 {
                    led.set_high();
                    nb::block!(timer.wait()).unwrap();
                    nb::block!(timer.wait()).unwrap();
                    led.set_low();
                    nb::block!(timer.wait()).unwrap();
                }
            }
        });
    }

    loop {
        continue;
    }
}
