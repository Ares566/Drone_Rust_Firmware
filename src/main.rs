mod log;

use crate::log::log::new_file_log;
use crate::log::Log;
// use rust_gpiozero::*;
use embedded_hal::blocking::delay::DelayMs;
use i2cdev::linux::LinuxI2CError;
use linux_embedded_hal::{Delay, I2cdev};
use mpu6050::{device::MOT_DETECT_STATUS, *};

// fn main() {
//     let mut log = new_file_log("firmware.log");
//     log.add_log("Start");
//     let led = LED::new(18); // sets a variable for the led pin
//     loop {
//         // starts a loop
//         led.on();
//         log.add_log("led.on");
//         sleep(Duration::from_secs(1)); // creates a 1 second pause
//         led.off();
//         log.add_log("led.off");
//         sleep(Duration::from_secs(1));
//     }

//     log.add_log("End");
// }

struct RPData {
    pitch: f32,
    roll: f32,
}
const MPU6050_KOEF_COMPL: f32 = 0.9934;

fn main() -> Result<(), Mpu6050Error<LinuxI2CError>> {
    let i2c = I2cdev::new("/dev/i2c-1").map_err(Mpu6050Error::I2c)?;

    let mut delay = Delay;
    let mut mpu = Mpu6050::new(i2c);

    mpu.init(&mut delay)?;

    let mut count: u16 = 0;
    let mut rp_data = RPData {
        pitch: 0.0,
        roll: 0.0,
    };
    loop {
        
        // get roll and pitch estimate
        let acc = mpu.get_acc_angles()?;
        let roll4macc: f32 = acc.data[0] / PI_180;
        let pitch4macc: f32 = acc.data[1] / PI_180;

        // get gyro data, scaled with sensitivity
        let gyro = mpu.get_gyro()?;
        
        rp_data.pitch += (gyro.data[1] / PI_180) * 0.75;
        rp_data.roll += (gyro.data[0] / PI_180) * 0.75;

        // коррекция дрейфа нуля гироскопа
        // Для корректировки углов воспользуемся комплементарным фильтром
        // A = (1-K)*Ag + K*Ac
        rp_data.pitch = rp_data.pitch * (1.0 - MPU6050_KOEF_COMPL) + pitch4macc * MPU6050_KOEF_COMPL;
        rp_data.roll = rp_data.roll * (1.0 - MPU6050_KOEF_COMPL) + roll4macc * MPU6050_KOEF_COMPL;

        println!("Углы: pitch={0} ,roll={1}", rp_data.pitch, rp_data.roll);
        println!("_____________________________________");

        delay.delay_ms(750u16);
        count += 1;
        if count > 500 {
            mpu.reset_device(&mut delay)?;
            break;
        }
    }
    Ok(())
}
