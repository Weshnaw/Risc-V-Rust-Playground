#![no_std]
#![no_main]

/*
* Basic blinking LEDs example using mtime/mtimecmp registers
* for "sleep" in a loop. Blinks each led once and goes to the next one.
*/
mod device;

extern crate panic_halt;

use crate::device::Device;
use ads1x1x::{FullScaleRange, SlaveAddr};
use hifive1::hal::prelude::*;
use hifive1::sprintln;
use riscv_rt::entry;

use pid::Pid;

#[entry]
fn main() -> ! {
    let address = SlaveAddr::default();
    let range = FullScaleRange::Within4_096V;
    let mut device = Device::init(150.mhz().into(), address, range);

    sprintln!("Initialized...");
    let mut pid = Pid::new(0.1, 0.01, 0.02, 1., 1., 1., 1., 0.5);
    let mut current_value = 0.5;
    loop {
        let (a0, percent) = device.adc();
        if !(current_value - 0.01 < percent && percent < current_value + 0.02) {
            //pid = Pid::new(0.1, 0.01, 0.02, 1., 1., 1., 1., percent);
            current_value = percent;
        }
        sprintln!("A0: {:2}% ({})", percent, a0);
        let output = pid.next_control_output(percent);
        sprintln!("PID: {:2}%", output.output);
        let speed = (percent - 0.5) / 0.5;
        device.pwm_speed(speed);
        device.sleep(1_000);
    }
}
