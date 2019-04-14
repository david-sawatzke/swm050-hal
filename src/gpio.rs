//! General Purpose Input / Output

use core::marker::PhantomData;

// TODO Implement marker for af with PushPull or OpenDrain
/// Extension trait to split a GPIO peripheral in independent pins and registers
pub trait GpioExt {
    /// The parts to split the GPIO into
    type Parts;

    /// Splits the GPIO block into independent pins and registers
    // NOTE We don't need an rcc parameter because it's enabled by default
    fn split(self) -> Self::Parts;
}

trait GpioRegExt {
    fn is_low(&self, pos: u8) -> bool;
    fn is_set_low(&self, pos: u8) -> bool;
    fn set_high(&self, pos: u8);
    fn set_low(&self, pos: u8);
}

pub struct AF0;
pub struct AF1;
pub struct AF2;
pub struct AF3;
pub struct AF4;
pub struct AF5;
pub struct AF6;
pub struct AF7;

/// Input mode (type state)
pub struct Input<MODE> {
    _mode: PhantomData<MODE>,
}

/// Floating input (type state)
pub struct Floating;

/// Pulled up input (type state)
pub struct PullUp;

/// Output mode (type state)
pub struct Output<MODE> {
    _mode: PhantomData<MODE>,
}

/// Push pull output (type state)
pub struct PushPull;

use embedded_hal::digital::{toggleable, InputPin, OutputPin, StatefulOutputPin};

macro_rules! gpio_trait {
    ($gpiox:ident) => {
        impl GpioRegExt for crate::swm050::$gpiox::RegisterBlock {
            fn is_low(&self, pos: u8) -> bool {
                self.dat.read().bits() & (1 << pos) != 0
            }

            fn is_set_low(&self, pos: u8) -> bool {
                self.dat.read().bits() & (1 << pos) == 0
            }

            fn set_high(&self, pos: u8) {
                unsafe { self.dat.modify(|r, w| w.bits(r.bits() | (1 << pos))) };
            }

            fn set_low(&self, pos: u8) {
                unsafe { self.dat.modify(|r, w| w.bits(r.bits() & !(1 << pos))) };
            }
        }
    };
}

gpio_trait!(gpioa);

#[allow(unused)]
macro_rules! gpio {
    ($GPIOX:ident, $gpiox:ident, [
        $($PXi:ident: ($pxi:ident, $i:expr, $MODE:ty),)+
    ]) => {
        /// GPIO
        pub mod $gpiox {
            use core::marker::PhantomData;

            use crate::swm050::$GPIOX;
            use embedded_hal::digital::{toggleable, InputPin, OutputPin, StatefulOutputPin};

            use cortex_m::interrupt::CriticalSection;

            use super::{Floating, GpioExt, GpioRegExt, Input, Output, PullUp, PushPull};

            /// GPIO parts
            pub struct Parts {
                $(
                    /// Pin
                    pub $pxi: $PXi<$MODE>,
                )+
            }

            impl GpioExt for $GPIOX {
                type Parts = Parts;

                fn split(self) -> Parts {
                    Parts {
                        $(
                            $pxi: $PXi { _mode: PhantomData },
                        )+
                    }
                }
            }

            $(
                /// Pin
                pub struct $PXi<MODE> {
                    _mode: PhantomData<MODE>,
                }

                impl<MODE> $PXi<MODE> {
                    /// Configures the pin to operate as a floating input pin
                    pub fn into_floating_input(self, _cs: &CriticalSection) -> $PXi<Input<Floating>> {
                        unsafe {
                            &(*$GPIOX::ptr())
                                .dir
                                .modify(|r, w| w.bits(r.bits() & !(1 << $i)));
                        }
                        $PXi { _mode: PhantomData }
                    }

                    /// Configures the pin to operate as a pulled up input pin
                    pub fn into_pull_up_input(self, _cs: &CriticalSection) -> $PXi<Input<PullUp>> {
                        unimplemented!();
                        // unsafe {
                        //     &(*$GPIOX::ptr())
                        //         .dir
                        //         .modify(|r, w| w.bits(r.bits() & !(1 << $i)));
                        // }
                        // $PXi { _mode: PhantomData }
                    }

                    /// Configures the pin to operate as an push pull output pin
                    pub fn into_push_pull_output(self, _cs: &CriticalSection) -> $PXi<Output<PushPull>> {
                        unsafe {
                            &(*$GPIOX::ptr())
                                .dir
                                .modify(|r, w| w.bits(r.bits() | (1 << $i)));
                        }
                        $PXi { _mode: PhantomData }
                    }
                }

                impl<MODE> StatefulOutputPin for $PXi<Output<MODE>> {
                    fn is_set_high(&self) -> bool {
                        !self.is_set_low()
                    }

                    fn is_set_low(&self) -> bool {
                        unsafe { (*$GPIOX::ptr()).is_set_low($i) }
                    }
                }

                impl<MODE> OutputPin for $PXi<Output<MODE>> {
                    fn set_high(&mut self) {
                        unsafe { (*$GPIOX::ptr()).set_high($i) }
                    }

                    fn set_low(&mut self) {
                        unsafe { (*$GPIOX::ptr()).set_low($i) }
                    }
                }

                impl<MODE> toggleable::Default for $PXi<Output<MODE>> {}

                impl<MODE> InputPin for $PXi<Input<MODE>> {
                    fn is_high(&self) -> bool {
                        !self.is_low()
                    }

                    fn is_low(&self) -> bool {
                        unsafe { (*$GPIOX::ptr()).is_low($i) }
                    }
                }
            )+
        }
    }
}

gpio!(GPIOA, gpioa, [
    PA_0: (pa_0, 0, Input<Floating>),
    PA_1: (pa_1, 1, Input<Floating>),
    PA_2: (pa_2, 2, Input<Floating>),
    PA_3: (pa_3, 3, Input<Floating>),
    PA_4: (pa_4, 4, Input<Floating>),
    PA_5: (pa_5, 5, Input<Floating>),
    PA_6: (pa_6, 6, Input<Floating>),
    PA_7: (pa_7, 7, Input<Floating>),
    PA_8: (pa_8, 8, Input<Floating>),
    PA_9: (pa_9, 9, Input<Floating>),
]);
