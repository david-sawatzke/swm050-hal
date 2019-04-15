use core::ops::Deref;

use embedded_hal::timer::{CountDown, Periodic};
use void::Void;

use crate::syscon::{ClockEnable, Clocks, Syscon};
use crate::time::Hertz;
pub(crate) type TimerRegisterBlock = swm050::tmrse0::RegisterBlock;

pub struct Timer<TIMER> {
    clocks: Clocks,
    timer: TIMER,
}

impl<TIMER> Timer<TIMER>
where
    TIMER: Deref<Target = TimerRegisterBlock> + ClockEnable,
{
    pub fn timer<T>(timer: TIMER, timeout: T, syscon: &mut Syscon) -> Timer<TIMER>
    where
        T: Into<Hertz>,
    {
        TIMER::enable(syscon);
        timer.intctrl.write(|w| w.ena().set_bit());
        // pause
        let mut timer = Timer {
            timer: timer,
            clocks: syscon.clocks,
        };
        timer.start(timeout);

        timer
    }

    pub fn release(self) -> TIMER {
        self.timer
    }
}

impl<TIMER> CountDown for Timer<TIMER>
where
    TIMER: Deref<Target = TimerRegisterBlock>,
{
    type Time = Hertz;

    /// Start the timer with a `timeout`
    fn start<T>(&mut self, timeout: T)
    where
        T: Into<Hertz>,
    {
        // pause
        self.timer.ctrl.write(|w| w.ena().clear_bit());

        // Clear overflow flag
        self.timer.intoflag.write(|w| unsafe { w.bits(0) });

        let frequency = timeout.into().0;
        // TODO If the ticks aren't halved, the periods are twice as long as they should be
        // But it works in delay?
        let ticks = self.clocks.timsclk().0 / frequency / 2;

        self.timer.tarval.write(|w| unsafe { w.bits(ticks) });
        self.timer.curval.write(|w| unsafe { w.bits(0) });

        // start counter
        self.timer.ctrl.write(|w| w.ena().set_bit());
    }

    /// Return `Ok` if the timer has wrapped
    /// Automatically clears the flag and restarts the time
    fn wait(&mut self) -> nb::Result<(), Void> {
        if self.timer.intoflag.read().bits() == 0 {
            Err(nb::Error::WouldBlock)
        } else {
            // Clear overflow flag
            self.timer.curval.read();
            Ok(())
        }
    }
}

impl<TIMER> Periodic for Timer<TIMER> where TIMER: Deref<Target = TimerRegisterBlock> {}
