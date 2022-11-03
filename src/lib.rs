//
// Traits needed in some library (probably `cortex-m-interrupt`)
//

mod cortex_m_interrupt {

    pub trait InterruptRegistration {
        fn on_interrupt();
    }

    /// This is implemented by codegen, `T` is needed as this crate does not
    /// know the type of the PAC that will be used.
    pub trait InterruptHandle<T> {
        const VECTOR: T; // Holds vector name for compiletime errors

        fn activate(self); // Enable the registered interrupt

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
    use crate::cortex_m_interrupt::{InterruptHandle, InterruptRegistration};

    pub use crate::pac;

    pub struct Spi {
        // ...
    }

    impl Spi {
        pub fn new<Handle>(spi: pac::SPI0, interrupt_handle: Handle) -> Self
        where
            Handle: InterruptHandle<pac::Interrupt>,
        {
            // const_assert!(Handle::VECTOR == pac::Interrupt::SpiIsr);

            // setup the peripheral ...

            interrupt_handle.activate();

            Spi {}
        }
    }

    impl InterruptRegistration for Spi {
        // It might have a dependency that you can't call `handle.activate()`
        // until peripheral setup is complete.
        fn on_interrupt() {
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
        #[export_name = "Spi0"]
        #[allow(non_snake_case)]
        pub unsafe extern "C" fn interrupt() {
            <hal::Spi as InterruptRegistration>::on_interrupt();
        }

        struct Handle(u8);

        impl InterruptHandle<hal::pac::Interrupt> for Handle {
            const VECTOR: hal::pac::Interrupt = hal::pac::Interrupt::Spi0;

            fn activate(self) {
                // TODO: Poke the NVIC
                // - enable interrupts
                // - setup prio

                atomic_polyfill::compiler_fence(atomic_polyfill::Ordering::Release);
            }

            fn override_priority(&mut self, priority: u8) {
                self.0 = priority;
            }
        }

        Handle(3) // prio goes here
    };
}
