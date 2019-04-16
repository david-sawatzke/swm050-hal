use core::ops::Deref;

use embedded_hal::timer::{Cancel, CountDown, Periodic};
use void::Void;

use crate::delay::Delay;
use crate::syscon::{ClockEnable, Clocks, Syscon};
use crate::time::Hertz;
pub(crate) type TimerRegisterBlock = swm050::tmrse0::RegisterBlock;

pub struct Timer<TIMER> {
    clocks: Clocks,
    pub(crate) timer: TIMER,
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

/// Implement `CountDown` for `Delay`. This *doesn't* use timer overflow functionality,
/// it just works with the current tick value. That means, a wait that should be done
/// could block once again after approx. 120s minimum.
impl CountDown for Delay {
    type Time = Hertz;

    fn start<T>(&mut self, timeout: T)
    where
        T: Into<Hertz>,
    {
        let frequency = timeout.into().0;
        let ticks = self.scale * 1_000_000 / frequency;
        self.countdown = Some((ticks, unsafe { (*self.timer).curval.read().bits() }));
    }

    fn wait(&mut self) -> nb::Result<(), Void> {
        if let Some((ref ticks, ref start_count)) = self.countdown {
            if unsafe {
                (*(self.timer))
                    .curval
                    .read()
                    .bits()
                    .wrapping_sub(*start_count)
            } < *ticks
            {
                Err(nb::Error::WouldBlock)
            } else {
                let ticks = *ticks;
                // Refresh the start count, so this is periodic
                self.countdown = Some((ticks, unsafe { (*self.timer).curval.read().bits() }));
                Ok(())
            }
        } else {
            // The timer wasn't set yet
            Err(nb::Error::WouldBlock)
        }
    }
}

impl Cancel for Delay {
    type Error = ();
    fn cancel(&mut self) -> Result<(), ()> {
        self.countdown = None;
        Ok(())
    }
}
