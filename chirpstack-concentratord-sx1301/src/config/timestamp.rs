use std::time::{Duration, SystemTime};
use std::ops;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{self, Visitor};
use std::fmt;
use std::result::Result;

#[derive(Copy, Clone)]
pub enum TimeStampMethod {
    Systemtime,
    GPS,
}

impl std::default::Default for TimeStampMethod {
    fn default() -> TimeStampMethod {TimeStampMethod::GPS}
}

impl<'de> Deserialize<'de> for TimeStampMethod {
    fn deserialize<D>(deserializer: D) -> Result<TimeStampMethod, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TimeStampMethodVisitor;

        impl <'de> Visitor<'de> for TimeStampMethodVisitor {
            type Value = TimeStampMethod;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("\"gps\" or \"systemtime\"")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                where
                    E: de::Error,
            {
                match value {
                    "gps" => Ok(TimeStampMethod::GPS),
                    "systemtime" => Ok(TimeStampMethod::Systemtime),
                    _ => {
                        warn!("Invalid timestamp method, falling back to GPS method");
                        return Ok(TimeStampMethod::GPS);
                    },
                }
            }

        }

        return deserializer.deserialize_identifier(TimeStampMethodVisitor);
    }
}

impl Serialize for TimeStampMethod {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            TimeStampMethod::Systemtime => serializer.serialize_str("systemtime"),
            TimeStampMethod::GPS => serializer.serialize_str("gps"),
        }
        
    }
}

#[derive(Copy, Clone)]
enum Operation {
    Add,
    Subtract
}

#[derive(Copy, Clone)]
pub struct TimeZone {
    offset : Duration,
    operation : Operation,
}

impl std::default::Default for TimeZone {
    fn default() -> TimeZone {TimeZone::ZERO}
}

impl ops::Add<TimeZone> for SystemTime {
    type Output = SystemTime;

    fn add(self, rhs : TimeZone) -> SystemTime {
        match rhs.operation {
            Operation::Add => self + rhs.offset,
            Operation::Subtract => self - rhs.offset,
        }
    }
}

impl TimeZone {
    pub const ZERO : TimeZone = TimeZone {offset : Duration::from_nanos(0), operation : Operation::Add};
}

impl Serialize for TimeZone {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hours : u64 = self.offset.as_secs()/60/60;
        let minutes : u64 = (self.offset.as_secs()-(hours*60*60))*60;
        let mut serial : String = String::with_capacity(5);
        match &self.operation {
            Operation::Add => serial.push('+'),
            Operation::Subtract => serial.push('-'),
        }

        let zero = "0".to_string();

        match hours.to_string().len()
        {
            0 => serial.push_str("00"),
            1 => serial.push_str((zero.clone() + &hours.to_string()).as_str()),
            2 => serial.push_str(hours.to_string().as_str()),
            _ => return serializer.serialize_str("+0000"),
        }

        match minutes.to_string().len()
        {
            0 => serial.push_str("00"),
            1 => serial.push_str((zero.clone() + &minutes.to_string()).as_str()),
            2 => serial.push_str(minutes.to_string().as_str()),
            _ => return serializer.serialize_str("+0000"),
        }

        return serializer.serialize_str(serial.as_str());
    }
}

impl<'de> Deserialize<'de> for TimeZone {
    fn deserialize<D>(deserializer: D) -> Result<TimeZone, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TimeZoneVisitor;

        impl <'de> Visitor<'de> for TimeZoneVisitor {
            type Value = TimeZone;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                where
                    E: de::Error,
            {
                let mut timezone = String::from(value);
                if timezone.len() == 5
                {
                    let operator : Operation;
                    let operator_char = timezone.remove(0);
                    match operator_char {
                        '+' => operator = Operation::Add,
                        '-' => operator = Operation::Subtract,
                        _ => {
                            warn!("Invalid operation before timezone values can only be + or - found: {}", operator_char);
                            return Ok(TimeZone::ZERO);
                        },
                    }
        
                    let mut seconds : u64 = 0;
        
                    match timezone.split_off(1).parse::<u64>()
                    {
                        Ok(v) => seconds += v*60,
                        Err(_e) => {
                            warn!("Parsing minutes in timezone failed, are all the values numbers?");
                            return Ok(TimeZone::ZERO);
                        },
                    }
        
                    match timezone.parse::<u64>()
                    {
                        Ok(v) => seconds += v*60*60,
                        Err(_e) => {
                            warn!("Parsing hours in timezone failed, are all the values numbers?");
                            return Ok(TimeZone::ZERO);
                        },
                    }
        
                    return Ok(TimeZone {offset: Duration::from_secs(seconds), operation : operator});
                }
                warn!("Invalid timezone string length, expected 5 got {}", timezone.len());
                return Ok(TimeZone::ZERO);
            }

        }
        const FIELDS: &'static [&'static str] = &["offset", "operation"];

        return deserializer.deserialize_struct("TimeZone", FIELDS, TimeZoneVisitor);
    }
}

