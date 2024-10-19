/// Access to gyroscope and accelerometer on the controller.
///
/// Compatible controllers including Playstation, Switch and Steam controllers include a gyroscope
/// and accelerometer to get the movement in space of the device.
///
/// Units used by SDL:
/// - Accelerometer is in m/s²
/// - Gyroscope is in radian per second
///
/// Axis when holding the controller:
/// - -x ... +x is left ... right
/// - -y ... +y is down ... up
/// - -z ... +z is forward ... backward
///
/// Rotations uses the standard anti-clockwise direction around the corresponding axis from above:
/// - -x ... +x is pitch towards up
/// - -y ... +y is yaw from right to left
/// - -z ... +z is roll from right to left
use crate::sys;


use crate::common::{validate_int, IntegerOrSdlError};
use crate::get_error;
use crate::SensorSubsystem;
use libc::c_char;
use std::ffi::{c_int, CStr};
use sys::sensor::{SDL_GetSensorData, SDL_Sensor, SDL_SensorType};
use sys::stdinc::SDL_free;

type SensorId = u32;

impl SensorSubsystem {
    /// Get a list of currently connected sensors.
    #[doc(alias = "SDL_GetSensors")]
    pub fn num_sensors(&self) -> Result<Vec<SensorId>, String> {
        let mut count: c_int = 0;
        let sensor_ids = unsafe { sys::sensor::SDL_GetSensors(&mut count) };

        if sensor_ids.is_null() {
            Err(get_error())
        } else {
            let ids = unsafe { std::slice::from_raw_parts(sensor_ids, count as usize) }
                .iter()
                .copied()
                .collect();
            unsafe { SDL_free(sensor_ids as *mut _) };
            Ok(ids)
        }
    }

    /// Attempt to open the sensor at index `sensor_id` and return it.
    #[doc(alias = "SDL_OpenSensor")]
    pub fn open(&self, sensor_id: SensorId) -> Result<Sensor, IntegerOrSdlError> {
        use crate::common::IntegerOrSdlError::*;
        let sensor_id = validate_int(sensor_id, "sensor_id")?;

        let sensor = unsafe { sys::sensor::SDL_OpenSensor(sensor_id.into()) };

        if sensor.is_null() {
            Err(SdlError(get_error()))
        } else {
            Ok(Sensor {
                subsystem: self.clone(),
                raw: sensor,
            })
        }
    }

    /// Force sensor update when not using the event loop
    #[inline]
    #[doc(alias = "SDL_UpdateSensors")]
    pub fn update(&self) {
        unsafe { sys::sensor::SDL_UpdateSensors() };
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorType {
    Unknown,
    Gyroscope,
    Accelerometer,
}

impl SensorType {
    pub fn from_ll(raw: i32) -> Self {
        match raw {
            x if x == SDL_SensorType::SDL_SENSOR_GYRO as i32 => SensorType::Gyroscope,
            x if x == SDL_SensorType::SDL_SENSOR_ACCEL as i32 => SensorType::Accelerometer,
            _ => SensorType::Unknown,
        }
    }
}

impl Into<SDL_SensorType> for SensorType {
    fn into(self) -> SDL_SensorType {
        match self {
            SensorType::Unknown => SDL_SensorType::SDL_SENSOR_UNKNOWN,
            SensorType::Gyroscope => SDL_SensorType::SDL_SENSOR_GYRO,
            SensorType::Accelerometer => SDL_SensorType::SDL_SENSOR_ACCEL,
        }
    }
}

/// Wrapper around the `SDL_Sensor` object
pub struct Sensor {
    subsystem: SensorSubsystem,
    raw: *mut SDL_Sensor,
}

impl Sensor {
    #[inline]
    pub const fn subsystem(&self) -> &SensorSubsystem {
        &self.subsystem
    }

    /// Return the name of the sensor or an empty string if no name
    /// is found.
    #[doc(alias = "SDL_GetSensorName")]
    pub fn name(&self) -> String {
        let name = unsafe { sys::sensor::SDL_GetSensorName(self.raw) };

        c_str_to_string(name)
    }

    #[doc(alias = "SDL_GetSensorID")]
    pub fn instance_id(&self) -> u32 {
        let result = unsafe { sys::sensor::SDL_GetSensorID(self.raw) };

        if result < 0 {
            // Should only fail if the joystick is NULL.
            panic!("{}", get_error())
        } else {
            result as u32
        }
    }

    /// Return the type of the sensor or `Unknown` if unsupported.
    #[doc(alias = "SDL_GetSensorType")]
    pub fn sensor_type(&self) -> SensorType {
        let result = unsafe { sys::sensor::SDL_GetSensorType(self.raw) };

        match result {
            SDL_SensorType::SDL_SENSOR_INVALID => {
                panic!("{}", get_error())
            }
            SDL_SensorType::SDL_SENSOR_UNKNOWN => SensorType::Unknown,
            SDL_SensorType::SDL_SENSOR_ACCEL => SensorType::Accelerometer,
            SDL_SensorType::SDL_SENSOR_GYRO => SensorType::Gyroscope,
        }
    }

    /// Get the current data from the sensor.
    ///
    /// Output depends on the type of the sensor. See module documentation for units and axis.
    #[doc(alias = "SDL_GetSensorType")]
    pub fn get_data(&self) -> Result<SensorData, IntegerOrSdlError> {
        let mut data = [0f32; 16];
        let result = unsafe { SDL_GetSensorData(self.raw, data.as_mut_ptr(), data.len() as i32) };

        if !result {
            Err(IntegerOrSdlError::SdlError(get_error()))
        } else {
            Ok(match self.sensor_type() {
                SensorType::Gyroscope => SensorData::Accel([data[0], data[1], data[2]]),
                SensorType::Accelerometer => SensorData::Accel([data[0], data[1], data[2]]),
                SensorType::Unknown => SensorData::Unknown(data),
            })
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SensorData {
    Gyro([f32; 3]),
    Accel([f32; 3]),
    Unknown([f32; 16]),
}

impl Drop for Sensor {
    #[doc(alias = "SDL_CloseSensor")]
    fn drop(&mut self) {
        unsafe { sys::sensor::SDL_CloseSensor(self.raw) }
    }
}

/// Convert C string `c_str` to a String. Return an empty string if
/// `c_str` is NULL.
fn c_str_to_string(c_str: *const c_char) -> String {
    if c_str.is_null() {
        String::new()
    } else {
        unsafe {
            CStr::from_ptr(c_str as *const _)
                .to_str()
                .unwrap()
                .to_owned()
        }
    }
}
