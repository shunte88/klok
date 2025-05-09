extern crate machine_ip;
use chrono::{Local, Timelike};

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};
use linux_embedded_hal::I2cdev;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use signal_hook::{consts::signal::*, iterator::Signals};
use std::{sync::atomic::{AtomicBool, Ordering}, sync::Arc, thread, time::Duration};

fn main() {

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // Setup signal listener for SIGINT, SIGTERM, SIGHUP
    let mut signals = Signals::new(&[SIGINT, SIGTERM, SIGHUP]).unwrap();
    let handle = signals.handle();
    let signal_thread = thread::spawn(move || {
        for sig in signals.forever() {
            println!("Received signal {:?}, exiting...", sig);
            r.store(false, Ordering::SeqCst);
            break;
        }
    });

    // Setup I2C
    let i2c = I2cdev::new("/dev/i2c-1").unwrap();
    //let interface = ssd1306::I2CDisplayInterface::new(i2c);
    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();
    display.flush().unwrap();

    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    let ww = display.size().width as i32;

    while running.load(Ordering::SeqCst) {
        let now = Local::now();
        let time_string = now.format("%H:%M:%S").to_string();
        let date_string = now.format("%Y-%m-%d").to_string();
        let seconds = now.second();

        let bb = if seconds % 2 == 0 {
            Rectangle::new(Point::new(4, 4), Size::new(ww as u32 - 8, 54))
        } else {
            Rectangle::new(Point::new(19, 11), Size::new(ww as u32 - 38, 37))
        };

        // Clear display and lets get going
        display.clear();

        // Draw black rectangle with white border
        display
            .bounding_box()
            .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off))
            .draw(&mut display)
            .unwrap();

        bb.into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut display)
            .unwrap();

        // Centered time
        let time_pos = center_text(&time_string, ww);
        Text::new(&time_string, Point::new(time_pos, 20), style)
            .draw(&mut display)
            .unwrap();

        // Centered date
        let date_pos = center_text(&date_string, ww);
        Text::new(&date_string, Point::new(date_pos, 31), style)
            .draw(&mut display)
            .unwrap();

        let local_addr = machine_ip::get().unwrap();
        let ip_string = local_addr.to_string();
        let ip_pos = center_text(&ip_string, ww);
        Text::new(&ip_string, Point::new(ip_pos, 42), style)
            .draw(&mut display)
            .unwrap();

        display.flush().unwrap();
        thread::sleep(Duration::from_millis(250));
    }
    // Cleanup on exit
    println!("done.");
    display.clear();
    display.flush().unwrap();

    handle.close();
    signal_thread.join().unwrap();
}

fn center_text(text: &str, width: i32) -> i32 {
    let char_width = 6; // FONT_6X10 char width
    let text_width = (text.len() as i32) * char_width;
    (width - text_width) / 2
}
