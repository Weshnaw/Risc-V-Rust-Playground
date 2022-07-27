use ads1x1x::ic::{Ads1115, Resolution16Bit};
use ads1x1x::interface::I2cInterface;
use ads1x1x::mode::OneShot;
use ads1x1x::{channel, Ads1x1x, FullScaleRange, SlaveAddr};
use hifive1::hal::delay::Sleep;
use hifive1::hal::e310x::I2C0;
use hifive1::hal::e310x::PWM0;
use hifive1::hal::gpio::gpio0::{Pin1, Pin12, Pin13};
use hifive1::hal::gpio::{NoInvert, Output, Regular, IOF0};
use hifive1::hal::i2c::*;
use hifive1::hal::prelude::*;
use hifive1::hal::time::Hertz;
use hifive1::hal::DeviceResources;
use hifive1::pin;
use nb::block;

pub struct Device {
  sleep: Sleep,
  adc: Ads1x1x<
    I2cInterface<I2c<I2C0, (Pin12<IOF0<NoInvert>>, Pin13<IOF0<NoInvert>>)>>,
    Ads1115,
    Resolution16Bit,
    OneShot,
  >,
  pwm: PWM0,
  reverse_pin: Pin1<Output<Regular<NoInvert>>>,
}

impl Device {
  pub fn init(clock_speed: Hertz, i2c_address: SlaveAddr, adc_range: FullScaleRange) -> Self {
    let resources = DeviceResources::take().unwrap();
    let p = resources.peripherals;
    let pins = resources.pins;
    let clock = hifive1::clock::configure(p.PRCI, p.AONCLK, clock_speed);
    let clint = resources.core_peripherals.clint;
    let sleep = Sleep::new(clint.mtimecmp, clock);

    // Configure UART for stdout
    hifive1::stdout::configure(
      p.UART0,
      pin!(pins, uart0_tx),
      pin!(pins, uart0_rx),
      115_200.bps(),
      clock,
    );

    let sda = pin!(pins, i2c0_sda).into_iof0();
    let scl = pin!(pins, i2c0_scl).into_iof0();
    let i2c = I2c::new(p.I2C0, sda, scl, Speed::Normal, clock);
    let mut adc = Ads1x1x::new_ads1115(i2c, i2c_address);
    adc.set_full_scale_range(adc_range).unwrap();
    let pwm = p.PWM0;
    pin!(pins, dig8).into_inverted_iof1();
    let reverse_pin = pin!(pins, dig9).into_output();
    //let reverse_pin = pin!(pins, dig1).into_output();
    pwm.cfg.write(|w| w.enalways().bit(true));

    Device {
      sleep,
      adc,
      pwm,
      reverse_pin,
    }
  }

  pub fn sleep(&mut self, ms: u32) {
    self.sleep.delay_ms(ms);
  }

  pub fn pwm_speed(&mut self, speed: f32) {
    let mut percent = speed;
    if speed < 0. {
      self.reverse_pin.set_high().unwrap();
      percent *= -1.;
    } else {
      self.reverse_pin.set_low().unwrap();
    }

    let duty = (255.0 * percent.clamp(0., 1.)) as u16;

    self.pwm.cmp0.write(|w| unsafe { w.value().bits(duty) });
  }

  pub fn adc(&mut self) -> (i16, f32) {
    let max_value = 26_188.;
    let a0 = block!(self.adc.read(&mut channel::SingleA0)).unwrap();
    (a0, (a0 as f32 / max_value).clamp(0., 1.))
  }
}
