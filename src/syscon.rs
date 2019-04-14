use crate::swm050::SYS;
use crate::time::Hertz;

/// Extension trait that sets up the `SYSCON` peripheral
pub trait SysconExt {
    /// Configure the clocks of the SYSCON peripheral
    fn configure(self) -> CFGR;
}

impl SysconExt for SYS {
    fn configure(self) -> CFGR {
        CFGR {
            timsclk: None,
            sclk: None,
            syscon: self,
        }
    }
}

/// Constrained syscon peripheral
pub struct Syscon {
    pub clocks: Clocks,
    pub(crate) regs: SYS,
}

pub struct CFGR {
    timsclk: Option<u32>,
    sclk: Option<u32>,
    syscon: SYS,
}

impl CFGR {
    pub fn timsclk<F>(mut self, freq: F) -> Self
    where
        F: Into<Hertz>,
    {
        self.timsclk = Some(freq.into().0);
        self
    }

    pub fn sclk<F>(mut self, freq: F) -> Self
    where
        F: Into<Hertz>,
    {
        self.sclk = Some(freq.into().0);
        self
    }

    pub fn freeze(self) -> Syscon {
        let sclk = self.sclk.map(|_| unimplemented!()).unwrap_or(18000000);
        let timsclk = self.timsclk.map(|_| unimplemented!()).unwrap_or(18000000);
        Syscon {
            clocks: Clocks {
                timsclk: Hertz(timsclk),
                sclk: Hertz(sclk),
            },
            regs: self.syscon,
        }
    }
}

/// Frozen clock frequencies
///
/// The existence of this value indicates that the clock configuration can no longer be changed
#[derive(Clone, Copy)]
pub struct Clocks {
    timsclk: Hertz,
    sclk: Hertz,
}

impl Clocks {
    /// Returns the frequency of the sysclock
    pub fn sclk(&self) -> Hertz {
        self.sclk
    }

    /// Returns the frequency of the timerclock
    pub fn timsclk(&self) -> Hertz {
        self.timsclk
    }
}

pub trait ClockEnable {
    fn enable(syscon: &mut Syscon);
}
macro_rules! clock_enable {
    ($PERIPH: ident, $field:ident) => {
        impl ClockEnable for swm050::$PERIPH {
            fn enable(syscon: &mut Syscon) {
                syscon.regs.pclk_en.modify(|_, w| w.$field().set_bit());
            }
        }
    };
}
clock_enable!(TMRSE0, tmrse0_clk);
clock_enable!(TMRSE1, tmrse1_clk);
clock_enable!(WDT, wdt_clk);
