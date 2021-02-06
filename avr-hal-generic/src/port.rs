use core::marker::PhantomData;

pub trait PinMode: crate::Sealed {}
pub mod mode {
    use core::marker::PhantomData;

    pub trait Io: crate::Sealed + super::PinMode {}

    pub struct Output;
    impl super::PinMode for Output {}
    impl Io for Output {}
    impl crate::Sealed for Output {}

    pub trait InputMode: crate::Sealed {}

    pub struct Input<IMODE> {
        pub(crate) _imode: PhantomData<IMODE>,
    }
    impl<IMODE: InputMode> super::PinMode for Input<IMODE> {}
    impl<IMODE: InputMode> Io for Input<IMODE> {}
    impl<IMODE: InputMode> crate::Sealed for Input<IMODE> {}

    pub struct Floating;
    impl InputMode for Floating {}
    impl crate::Sealed for Floating {}

    pub struct PullUp;
    impl InputMode for PullUp {}
    impl crate::Sealed for PullUp {}
}

pub trait PinOps {
    type Dynamic;

    fn into_dynamic(self) -> Self::Dynamic;

    unsafe fn out_set(&mut self);
    unsafe fn out_clear(&mut self);
    unsafe fn out_toggle(&mut self);
    unsafe fn out_get(&self) -> bool;

    unsafe fn in_get(&self) -> bool;

    unsafe fn make_output(&mut self);
    unsafe fn make_input(&mut self, pull_up: bool);
}

pub struct Pin<MODE, PIN> {
    pub(crate) pin: PIN,
    pub(crate) _mode: PhantomData<MODE>,
}

impl<PIN: PinOps> Pin<mode::Input<mode::Floating>, PIN> {
    #[doc(hidden)]
    pub fn new(pin: PIN) -> Self {
        Pin {
            pin,
            _mode: PhantomData,
        }
    }
}

impl<PIN: PinOps, MODE: mode::Io> Pin<MODE, PIN> {
    pub fn into_output(mut self) -> Pin<mode::Output, PIN> {
        unsafe { self.pin.make_output() };
        Pin {
            pin: self.pin,
            _mode: PhantomData,
        }
    }

    pub fn into_floating_input(mut self) -> Pin<mode::Input<mode::Floating>, PIN> {
        unsafe { self.pin.make_input(false) };
        Pin {
            pin: self.pin,
            _mode: PhantomData,
        }
    }

    pub fn into_pull_up_input(mut self) -> Pin<mode::Input<mode::PullUp>, PIN> {
        unsafe { self.pin.make_input(true) };
        Pin {
            pin: self.pin,
            _mode: PhantomData,
        }
    }

    pub fn downgrade(self) -> Pin<MODE, PIN::Dynamic> {
        Pin {
            pin: self.pin.into_dynamic(),
            _mode: PhantomData,
        }
    }
}

impl<PIN: PinOps> Pin<mode::Output, PIN> {
    #[inline]
    pub fn set_high(&mut self) {
        unsafe { self.pin.out_set() }
    }

    #[inline]
    pub fn set_low(&mut self) {
        unsafe { self.pin.out_clear() }
    }

    #[inline]
    pub fn toggle(&mut self) {
        unsafe { self.pin.out_toggle() }
    }

    #[inline]
    pub fn is_set_high(&self) -> bool {
        unsafe { self.pin.out_get() }
    }

    #[inline]
    pub fn is_set_low(&self) -> bool {
        !unsafe { self.pin.out_get() }
    }
}

impl<PIN: PinOps, IMODE: mode::InputMode> Pin<mode::Input<IMODE>, PIN> {
    #[inline]
    pub fn is_high(&self) -> bool {
        unsafe { self.pin.in_get() }
    }

    #[inline]
    pub fn is_low(&self) -> bool {
        !unsafe { self.pin.in_get() }
    }
}

#[macro_export]
macro_rules! impl_port_traditional {
    (
        enum Ports {
            $($PortName:ident: ($Port:ty, $port_port_reg:ident, $port_pin_reg:ident, $port_ddr_reg:ident),)+
        }

        $(#[$pins_attr:meta])*
        pub struct Pins {
            $($pin:ident: $Pin:ident = ($PinPort:ty, $PinPortName:ident, $pin_num:expr,
                                        $pin_port_reg:ident, $pin_pin_reg:ident,
                                        $pin_ddr_reg:ident),)+
        }
    ) => {
        pub use $crate::port::mode;
        pub type Pin<MODE, PIN = Dynamic> = $crate::port::Pin<MODE, PIN>;

        $(#[$pins_attr])*
        pub struct Pins {
            $(pub $pin: Pin<
                mode::Input<mode::Floating>,
                $Pin,
            >,)+
        }

        impl Pins {
            pub fn new(
                $(_: $Port,)+
            ) -> Self {
                Self {
                    $($pin: $crate::port::Pin::new(
                        $Pin { _private: (), }
                    ),)+
                }
            }
        }

        #[repr(u8)]
        pub enum DynamicPort {
            $($PortName,)+
        }

        pub struct Dynamic {
            port: DynamicPort,
            // We'll store the mask instead of the pin number because this allows much less code to
            // be generated for the trait method implementations.
            mask: u8,
        }

        impl Dynamic {
            fn new(port: DynamicPort, pin_num: u8) -> Self {
                Self {
                    port,
                    mask: 1 << pin_num,
                }
            }
        }

        impl $crate::port::PinOps for Dynamic {
            type Dynamic = Self;

            #[inline]
            fn into_dynamic(self) -> Self::Dynamic {
                self
            }

            #[inline]
            unsafe fn out_set(&mut self) {
                match self.port {
                    $(DynamicPort::$PortName => (*<$Port>::ptr()).$port_port_reg.modify(|r, w| {
                        w.bits(r.bits() | self.mask)
                    }),)+
                }
            }

            #[inline]
            unsafe fn out_clear(&mut self) {
                match self.port {
                    $(DynamicPort::$PortName => (*<$Port>::ptr()).$port_port_reg.modify(|r, w| {
                        w.bits(r.bits() & !self.mask)
                    }),)+
                }
            }

            #[inline]
            unsafe fn out_toggle(&mut self) {
                match self.port {
                    $(DynamicPort::$PortName => (*<$Port>::ptr()).$port_pin_reg.modify(|r, w| {
                        w.bits(r.bits() | self.mask)
                    }),)+
                }
            }

            #[inline]
            unsafe fn out_get(&self) -> bool {
                match self.port {
                    $(DynamicPort::$PortName => (*<$Port>::ptr()).$port_port_reg.read().bits()
                        & self.mask != 0,)+
                }
            }

            #[inline]
            unsafe fn in_get(&self) -> bool {
                match self.port {
                    $(DynamicPort::$PortName => (*<$Port>::ptr()).$port_pin_reg.read().bits()
                        & self.mask != 0,)+
                }
            }

            #[inline]
            unsafe fn make_output(&mut self) {
                match self.port {
                    $(DynamicPort::$PortName => (*<$Port>::ptr()).$port_ddr_reg.modify(|r, w| {
                        w.bits(r.bits() | self.mask)
                    }),)+
                }
            }

            #[inline]
            unsafe fn make_input(&mut self, pull_up: bool) {
                match self.port {
                    $(DynamicPort::$PortName => (*<$Port>::ptr()).$port_ddr_reg.modify(|r, w| {
                        w.bits(r.bits() & !self.mask)
                    }),)+
                }
                if pull_up {
                    self.out_clear()
                } else {
                    self.out_set()
                }
            }
        }

        $(
            pub struct $Pin {
                _private: ()
            }

            impl $crate::port::PinOps for $Pin {
                type Dynamic = Dynamic;

                #[inline]
                fn into_dynamic(self) -> Self::Dynamic {
                    Dynamic::new(DynamicPort::$PinPortName, $pin_num)
                }

                #[inline]
                unsafe fn out_set(&mut self) {
                    (*<$PinPort>::ptr()).$pin_port_reg.modify(|r, w| {
                        w.bits(r.bits() | (1 << $pin_num))
                    })
                }

                #[inline]
                unsafe fn out_clear(&mut self) {
                    (*<$PinPort>::ptr()).$pin_port_reg.modify(|r, w| {
                        w.bits(r.bits() & !(1 << $pin_num))
                    })
                }

                #[inline]
                unsafe fn out_toggle(&mut self) {
                    (*<$PinPort>::ptr()).$pin_pin_reg.modify(|r, w| {
                        w.bits(r.bits() | (1 << $pin_num))
                    })
                }

                #[inline]
                unsafe fn out_get(&self) -> bool {
                    (*<$PinPort>::ptr()).$pin_port_reg.read().bits() & (1 << $pin_num) != 0
                }

                #[inline]
                unsafe fn in_get(&self) -> bool {
                    (*<$PinPort>::ptr()).$pin_pin_reg.read().bits() & (1 << $pin_num) != 0
                }

                #[inline]
                unsafe fn make_output(&mut self) {
                    (*<$PinPort>::ptr()).$pin_ddr_reg.modify(|r, w| {
                        w.bits(r.bits() | (1 << $pin_num))
                    })
                }

                #[inline]
                unsafe fn make_input(&mut self, pull_up: bool) {
                    (*<$PinPort>::ptr()).$pin_ddr_reg.modify(|r, w| {
                        w.bits(r.bits() & !(1 << $pin_num))
                    });
                    if pull_up {
                        self.out_clear()
                    } else {
                        self.out_set()
                    }
                }
            }
        )+
    };
}

#[macro_export]
macro_rules! renamed_pins {
    (
        type Pin = $PinType:ident;

        $(#[$pins_attr:meta])*
        pub struct Pins from $McuPins:ty {
            $($(#[$pin_attr:meta])* pub $pin:ident: $Pin:ty = $pin_orig:ident,)+
        }
    ) => {
        $(#[$pins_attr])*
        pub struct Pins {
            $($(#[$pin_attr])*
            pub $pin: $PinType<
                $crate::port::mode::Input<$crate::port::mode::Floating>,
                $Pin,
            >,)+
        }

        impl Pins {
            pub fn with_mcu_pins(pins: $McuPins) -> Self {
                Self {
                    $($pin: pins.$pin_orig,)+
                }
            }
        }
    };
}
