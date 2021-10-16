#![no_std]
#![no_main]

pub use gd32vf103xx_hal as hal;

use hal::eclic::{EclicExt, Level, Priority, TriggerType};
use hal::pac::ECLIC;
use hal::pac::{Interrupt, TIMER0};
use hal::timer::Timer;

use panic_halt as _;
use riscv_rt::entry;

// use longan_nano::hal::rtc::Rtc;
use longan_nano::hal::{pac, prelude::*};
use longan_nano::{lcd, lcd_pins};

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};

static mut COLOR: Rgb565 = Rgb565::RED;
static mut TIMER: Option<Timer<TIMER0>> = None;

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    // Configure clocks
    let mut rcu = dp
        .RCU
        .configure()
        .ext_hf_clock(8.mhz())
        .sysclk(108.mhz())
        .freeze();

    let mut afio = dp.AFIO.constrain(&mut rcu);

    let gpioa = dp.GPIOA.split(&mut rcu);
    let gpiob = dp.GPIOB.split(&mut rcu);

    let lcd_pins = lcd_pins!(gpioa, gpiob);
    let mut lcd = lcd::configure(dp.SPI0, lcd_pins, &mut afio, &mut rcu);
    let (width, height) = (lcd.size().width as i32, lcd.size().height as i32);

    // Clear screen
    Rectangle::new(Point::new(0, 0), Size::new(width as u32, height as u32))
        .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
        .draw(&mut lcd)
        .unwrap();

    // let pmu = dp.PMU;
    // let rtc_reg = dp.RTC;
    // let mut bkp = dp.BKP.configure(&mut rcu, &mut pmu);
    // let rtc = Rtc::rtc(rtc_reg, &mut bkp);

    unsafe {
        TIMER = Some(Timer::timer0(dp.TIMER0, 5.hz(), &mut rcu));
        TIMER.as_mut().unwrap().listen(hal::timer::Event::Update);
    }

    // eclic stuff
    ECLIC::reset();
    ECLIC::set_level(Interrupt::TIMER0_UP, Level::L15);
    ECLIC::set_priority(Interrupt::TIMER0_UP, Priority::P15);
    ECLIC::set_threshold_level(Level::L15);

    ECLIC::setup(
        Interrupt::TIMER0_UP,
        TriggerType::Level,
        Level::L15,
        Priority::P15,
    );

    unsafe {
        ECLIC::unmask(Interrupt::TIMER0_UP);
        riscv::interrupt::enable();
        riscv::asm::wfi(); // Sleep
    }

    loop {
        unsafe {
            // Draw color
            Rectangle::new(Point::new(0, 0), Size::new(width as u32, height as u32))
                .into_styled(PrimitiveStyle::with_fill(COLOR))
                .draw(&mut lcd)
                .unwrap();
            riscv::asm::wfi();
        }
        // riscv::interrupt::free(f)
    }
}

#[no_mangle]
fn TIMER0_UP() {
    unsafe {
        riscv::interrupt::disable();
        if COLOR == Rgb565::RED {
            COLOR = Rgb565::BLUE;
        } else {
            COLOR = Rgb565::RED;
        }
        TIMER.as_mut().unwrap().clear_update_interrupt_flag();
        riscv::interrupt::enable();
    }
}
