mod log;

use crate::log::log::new_file_log;
use crate::log::Log;
// use rust_gpiozero::*;
use embedded_hal::blocking::delay::DelayMs;
use i2cdev::linux::LinuxI2CError;
use linux_embedded_hal::{Delay, I2cdev};
use mpu6050::{device::MOT_DETECT_STATUS, *};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use troyka_hat::*;

//     let mut log = new_file_log("firmware.log");
//     log.add_log("Start");

#[derive(Copy, Clone)]
struct RPData {
    pitch: f32,
    roll: f32,
    yaw: f32,
}

const MPU6050_KOEF_COMPL: f32 = 0.9934;
const DELTA_TIME: u16 = 250;
fn main() -> Result<(), Mpu6050Error<LinuxI2CError>> {
    let (tx, rx) = mpsc::channel();

    let mut delay = Delay;

    let i2c = I2cdev::new("/dev/i2c-1").map_err(Mpu6050Error::I2c)?;
    let mut mpu = Mpu6050::new(i2c);

    let i2c = I2cdev::new("/dev/i2c-1")
        .map_err(TroykaHatError::I2c)
        .expect("Failed to i2c");
    let mut th = troyka_hat::TroykaHat::new(i2c);

    mpu.init(&mut delay)?;
    thread::spawn(|| reading_from_mpu(mpu, tx));

    th.init(&mut delay);
    th.pwm_freq(50);

    for rp_data in rx {
        
        let mid_serv_val: f32 = (28.0 + 8.0) / 2.0;
        let mut serv_val: u8 = (mid_serv_val + rp_data.roll / 6.9) as u8;

        if rp_data.roll > 45.0 {
            serv_val = 28
        }
        if rp_data.roll < -45.0 {
            serv_val = 8
        }

        th.analog_write(7, serv_val);

        println!(
            "Angles: pitch={0} ,roll={1}; Servo: {2}",
            rp_data.pitch, rp_data.roll, serv_val
        );
        println!("_____________________________________");
    }

    Ok(())
}

fn reading_from_mpu(mut mpu: Mpu6050<I2cdev>, tx: mpsc::Sender<RPData>) {
    let mut count: u16 = 0;

    let mut rp_data = RPData {
        pitch: 0.0,
        roll: 0.0,
        yaw: 0.0,
    };
    loop {
        // TODO MPU initial calibration

        // get roll and pitch estimate
        let acc = mpu.get_acc_angles().unwrap();
        let roll4macc: f32 = acc.data[0] / PI_180;
        let pitch4macc: f32 = acc.data[1] / PI_180;

        // get gyro data, scaled with sensitivity
        let gyro = mpu.get_gyro().unwrap();

        rp_data.pitch += (gyro.data[1] / PI_180) * (DELTA_TIME / 1000) as f32;
        rp_data.roll += (gyro.data[0] / PI_180) * (DELTA_TIME / 1000) as f32;
        rp_data.yaw += (gyro.data[2] / PI_180) * (DELTA_TIME / 1000) as f32;

        //учитываем параметр по Z если по нему есть движение
        if f32::abs(gyro.data[2]) > 0.0 {
            let _y: f32 = f32::sin(gyro.data[2]);
            rp_data.pitch += rp_data.roll * _y;
            rp_data.roll -= rp_data.pitch * _y;
        }

        // коррекция дрейфа нуля гироскопа
        // Для корректировки углов воспользуемся комплементарным фильтром
        // A = (1-K)*Ag + K*Ac
        rp_data.pitch =
            rp_data.pitch * (1.0 - MPU6050_KOEF_COMPL) + pitch4macc * MPU6050_KOEF_COMPL;
        rp_data.roll = rp_data.roll * (1.0 - MPU6050_KOEF_COMPL) + roll4macc * MPU6050_KOEF_COMPL;

        tx.send(rp_data).unwrap();
        thread::sleep(Duration::from_millis(DELTA_TIME as u64));

        count += 1;
        if count > 500 {
            //mpu.reset_device(&mut delay)?;
            break;
        }
    }
}
