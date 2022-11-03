use crate::waker_registration::CriticalSectionWakerRegistration;

pub mod waker_registration;

//
// Traits needed in some library (probably `cortex-m-interrupt`)
//

mod cortex_m_interrupt {
    use crate::waker_registration::CriticalSectionWakerRegistration;

    pub trait InterruptRegistration {
        const NUM_WAKERS: usize = 0;

        fn on_interrupt(wakers: &[CriticalSectionWakerRegistration]);
    }

    /// This is implemented by codegen, `T` is needed as this crate does not
    /// know the type of the PAC that will be used.
    pub trait InterruptHandle<T> {
        const VECTOR: T; // Holds vector name for compiletime errors

        fn activate(self) -> &'static [CriticalSectionWakerRegistration]; // Enable the registered interrupt

        fn override_priority(&mut self, priority: u8); // New priority
    }
}

//
// HAL impl
//
// This takes an interrupt handle and checks that the correct
// handler was registered.
//

pub mod pac {
    pub struct SPI0;

    pub enum Interrupt {
        Int1,
        Int2,
        Int3,
        Spi0,
    }
}

pub mod hal {
    use crate::{
        cortex_m_interrupt::{InterruptHandle, InterruptRegistration},
        waker_registration::CriticalSectionWakerRegistration,
    };

    pub use crate::pac;

    pub struct Spi {
        // For use in `Future`s created by the SPI
        wakers: &'static [CriticalSectionWakerRegistration],
    }

    impl Spi {
        pub fn new<Handle>(spi: pac::SPI0, interrupt_handle: Handle) -> Self
        where
            Handle: InterruptHandle<pac::Interrupt>,
        {
            // const_assert!(Handle::VECTOR == pac::Interrupt::SpiIsr);

            // setup the peripheral ...

            let wakers = interrupt_handle.activate();

            Spi { wakers }
        }
    }

    impl InterruptRegistration for Spi {
        const NUM_WAKERS: usize = 2;

        // It might have a dependency that you can't call `handle.activate()`
        // until peripheral setup is complete.
        fn on_interrupt(wakers: &[CriticalSectionWakerRegistration]) {
            // Doing stuff ...
        }
    }
}

//
// User code
//
pub fn test() {
    use crate::cortex_m_interrupt::{InterruptHandle, InterruptRegistration};

    // let handle = register_interrupt!(
    // 	hal::pac::Interrupt::Spi0, // Full path to interrupt to register to
    // 	hal::Spi, // Struct implementing `InterruptRegistration`
    // 	3, // Optional priority
    // );

    // => codegen

    let handle = {
        const NUM_WAKERS: usize = <hal::Spi as InterruptRegistration>::NUM_WAKERS;
        const NEW_AW: CriticalSectionWakerRegistration = CriticalSectionWakerRegistration::new();
        static WAKERS: [CriticalSectionWakerRegistration; NUM_WAKERS] = [NEW_AW; NUM_WAKERS];

        #[export_name = "Spi0"]
        #[allow(non_snake_case)]
        pub unsafe extern "C" fn interrupt() {
            <hal::Spi as InterruptRegistration>::on_interrupt(&WAKERS);
        }

        struct Handle(u8);

        impl InterruptHandle<hal::pac::Interrupt> for Handle {
            const VECTOR: hal::pac::Interrupt = hal::pac::Interrupt::Spi0;

            fn activate(self) -> &'static [CriticalSectionWakerRegistration] {
                // TODO: Poke the NVIC
                // - enable interrupts
                // - setup prio

                atomic_polyfill::compiler_fence(atomic_polyfill::Ordering::Release);

                &WAKERS
            }

            fn override_priority(&mut self, priority: u8) {
                self.0 = priority;
            }
        }

        Handle(3) // prio goes here
    };
}
