//
// Traits needed in some library (probably `cortex-m-interrupt`)
//

mod cortex_m_interrupt {
    /// This trait is implemented by the HAL.
    pub trait InterruptRegistration<Vector> {
        const VECTOR: Vector; // Holds vector name for compiletime errors

        fn on_interrupt();
    }

    /// This trait is implemented by the proc-macro on the token.
    pub unsafe trait InterruptToken<Periperhal> {}
}

//
// HAL impl
//
// This takes an interrupt token and checks that the correct
// handler was registered.
//

pub mod pac {
    pub struct SPI0;

    pub struct UART0;

    pub struct UART1;

    pub struct UART2;

    pub enum Interrupt {
        Int1,
        Int2,
        Int3,
        Spi0,
        Uart0_1,
        Uart2,
    }
}

pub mod hal {
    use crate::cortex_m_interrupt::{InterruptRegistration, InterruptToken};

    pub use crate::pac;

    pub struct Spi {
        // ...
    }

    impl Spi {
        pub fn new<Token>(spi: pac::SPI0, interrupt_token: Token) -> Self
        where
            Token: InterruptToken<Spi>,
        {
            Spi {}
        }
    }

    impl InterruptRegistration<pac::Interrupt> for Spi {
        const VECTOR: pac::Interrupt = pac::Interrupt::Spi0;

        fn on_interrupt() {
            // Doing stuff ...
        }
    }

    //
    // ---
    //

    pub struct Uart0 {}

    impl Uart0 {
        pub fn new<Token>(uart: pac::UART0, interrupt_token: Token) -> Self
        where
            Token: InterruptToken<Uart0>,
        {
            Uart0 {}
        }
    }

    impl InterruptRegistration<pac::Interrupt> for Uart0 {
        const VECTOR: pac::Interrupt = pac::Interrupt::Uart0_1;

        fn on_interrupt() {
            // Doing stuff ...
        }
    }

    pub struct Uart1 {}

    impl Uart1 {
        pub fn new<Token>(uart: pac::UART1, interrupt_token: Token) -> Self
        where
            Token: InterruptToken<Uart1>,
        {
            Uart1 {}
        }
    }

    impl InterruptRegistration<pac::Interrupt> for Uart1 {
        const VECTOR: pac::Interrupt = pac::Interrupt::Uart0_1;

        fn on_interrupt() {
            // Doing stuff ...
        }
    }

    pub struct Uart2 {}

    impl Uart2 {
        pub fn new<Token>(uart: pac::UART2, interrupt_token: Token) -> Self
        where
            Token: InterruptToken<Uart2>,
        {
            Uart2 {}
        }
    }

    impl InterruptRegistration<pac::Interrupt> for Uart2 {
        const VECTOR: pac::Interrupt = pac::Interrupt::Uart2;

        fn on_interrupt() {
            // Doing stuff ...
        }
    }
}

//
// User code
//
pub fn test() {
    use crate::cortex_m_interrupt::{InterruptRegistration, InterruptToken};

    //
    //
    // Single interrupt example
    //
    //

    // let token = register_interrupt!(
    //     hal::pac::Interrupt::Spi0, // Full path to interrupt to register to
    //     hal::Spi, // Struct implementing `InterruptRegistration`
    // );

    // => codegen

    let token = {
        const _CHECK: () = {
            match <hal::Spi as InterruptRegistration<pac::Interrupt>>::VECTOR {
                pac::Interrupt::Spi0 => {}
                _ => panic!("Wrong vector"),
            }
        };

        #[export_name = "Spi0"]
        #[allow(non_snake_case)]
        pub unsafe extern "C" fn interrupt() {
            <hal::Spi as InterruptRegistration<pac::Interrupt>>::on_interrupt();
        }

        struct Token(u8);

        unsafe impl InterruptToken<hal::Spi> for Token {}

        Token(3) // prio goes here
    };

    let spi = hal::Spi::new(pac::SPI0 {}, token);

    //
    //
    // Multi (shared) interrupt example
    //
    //

    // let token = register_interrupt!(
    //     hal::pac::Interrupt::Uart0_1, // Full path to interrupt to register to
    //     hal::Uart0, hal::Uart1, // Struct implementing `InterruptRegistration`
    // );

    // => codegen

    let token2 = {
        const _: () = {
            match <hal::Uart0 as InterruptRegistration<pac::Interrupt>>::VECTOR {
                pac::Interrupt::Uart0_1 => {}
                _ => panic!("Wrong vector"),
            }
        };

        const _: () = {
            match <hal::Uart1 as InterruptRegistration<pac::Interrupt>>::VECTOR {
                pac::Interrupt::Uart0_1 => {}
                _ => panic!("Wrong vector"),
            }
        };

        #[export_name = "Uart0_1"]
        #[allow(non_snake_case)]
        pub unsafe extern "C" fn interrupt() {
            <hal::Uart0 as InterruptRegistration<pac::Interrupt>>::on_interrupt();
            <hal::Uart1 as InterruptRegistration<pac::Interrupt>>::on_interrupt();
        }

        #[derive(Copy, Clone)]
        struct Token;

        unsafe impl InterruptToken<hal::Uart0> for Token {}
        unsafe impl InterruptToken<hal::Uart1> for Token {}

        Token {}
    };

    let uart0 = hal::Uart0::new(pac::UART0 {}, token2);
    let uart1 = hal::Uart1::new(pac::UART1 {}, token2);

    //
    //
    // Error example
    //
    //

    let uart2 = hal::Uart2::new(pac::UART2 {}, token2); // this fails
}
