use crate::swm050::WDT;
use crate::syscon::{ClockEnable, Clocks, Syscon};
use crate::time::Hertz;
use embedded_hal::watchdog;

/// Watchdog instance
pub struct Watchdog {
    clocks: Clocks,
    wdt: WDT,
}

impl Watchdog {
    pub fn new(wdt: WDT, syscon: &mut Syscon) -> Self {
        WDT::enable(syscon);
        Self {
            wdt,
            clocks: syscon.clocks,
        }
    }
}

impl watchdog::Watchdog for Watchdog {
    /// Feed the watchdog, so that at least one `period` goes by before the next
    /// reset
    fn feed(&mut self) {
        self.wdt.crr.write(|w| w.crr().reset());
    }
}

impl watchdog::WatchdogEnable for Watchdog {
    type Time = Hertz;
    fn start<T>(&mut self, period: T)
    where
        T: Into<Hertz>,
    {
        let time = period.into();
        // TODO Verify function
        // As far as i understand the data sheet, it's basically like this:
        // time2 is used for mode 0, after the interrupt the counter is set
        // to timer2, otherwise timer1
        let ticks = self.clocks.sclk().0 / time.0;
        let mut timerticks = (ticks >> 16).next_power_of_two();
        let mut timer1 = 0;
        while timerticks != 0 {
            timer1 += 1;
            timerticks >>= 1;
        }
        // This shouldn't happen, but let's make sure
        assert!(timer1 < 16);
        self.wdt.torr.write(|w| w.top_init().bits(timer1 as u8));
        self.wdt.cr.write(|w| w.en().set_bit());
    }
}
