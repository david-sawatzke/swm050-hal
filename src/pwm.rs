use core::ops::Deref;

use embedded_hal::PwmPin;
use swm050::{PORT, TMRSE1};

use crate::gpio;
use crate::gpio::Output;
use crate::syscon::{ClockEnable, Syscon};
use crate::time::Hertz;
use crate::timers::TimerRegisterBlock;

pub struct Pwm<TIMER, PIN> {
    timer: TIMER,
    pin: PIN,
    ticks: u16,
}

// TODO TMRSE0 isn't supported yet, since it shares pins with swd
// Not sure how to do this while ensuring that swd isn't accidentally disabled
impl Pwm<TMRSE1, gpio::gpioa::PA_7<Output>> {
    pub fn new<T>(
        timer: TMRSE1,
        pin: gpio::gpioa::PA_7<Output>,
        period: T,
        port: &mut PORT,
        syscon: &mut Syscon,
    ) -> Self
    where
        T: Into<Hertz>,
    {
        TMRSE1::enable(syscon);

        let frequency = period.into().0;
        let ticks = syscon.clocks.timsclk().0 / frequency;
        assert!(ticks < 0x10000);

        // Enable output
        port.porta_sel.modify(|_, w| w.pa07().tmrse1_out());
        timer.ctrl.write(|w| w.ena().set_bit().wmod().pwm());
        let mut pwm_pin = Pwm {
            timer: timer,
            pin: pin,
            ticks: ticks as u16,
        };
        pwm_pin.set_duty(0);
        pwm_pin
    }
}
impl Pwm<TMRSE1, gpio::gpioa::PA_7<Output>> {
    pub fn release(self, port: &mut PORT) -> (TMRSE1, gpio::gpioa::PA_7<Output>) {
        port.porta_sel.modify(|_, w| w.pa07().gpio());
        (self.timer, self.pin)
    }
}

// The pwm implementation is a bit curious.
// You can seperately define the high & low time, so the total period can be up to 2 * 2^16,
// but only 2^16 for the high/low time
impl<TIMER, GPIO> PwmPin for Pwm<TIMER, GPIO>
where
    TIMER: Deref<Target = TimerRegisterBlock>,
{
    type Duty = u16;

    fn disable(&mut self) {
        self.timer.ctrl.write(|w| w.ena().clear_bit());
    }

    fn enable(&mut self) {
        // pause
        self.timer.ctrl.write(|w| w.ena().set_bit());
    }

    fn get_duty(&self) -> Self::Duty {
        (self.timer.tarval.read().bits() & 0xFFFF) as u16
    }

    fn get_max_duty(&self) -> Self::Duty {
        self.ticks
    }

    fn set_duty(&mut self, duty: Self::Duty) {
        let low = duty - self.get_max_duty();
        self.timer
            .tarval
            .write(|w| unsafe { w.bits(duty as u32 | ((low as u32) << 16)) });
    }
}
