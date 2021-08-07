use std::time::{Duration, SystemTime};
use std::ops;
use serde::{Deserialize, Serialize};
use serde::de::{self, Visitor};

pub enum TimeStampMethod {
    Systemtime,
    GPS,
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

            fn visit_str<E>(self, value: &str) -> Result<value, E>
                where
                    E: de::Error,
            {
                match value {
                    "gps" => Ok(TimeStampMethod::GPS),
                    "systemtime" => Ok(TimeStampMethod::Systemtime),
                    _ => Err(de::Error::unknown_field(value, FIELDS)),
                }
            }

        }
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

pub struct TimeZone {
    enum Operation {
        Add,
        Subtract
    }
    offset : Duration,
    operation : Operation,
}

impl ops::Add<TimeZone> for SystemTime {
    fn add(&self, rhs : TimeZone) -> SystemTime {
        match rhs.operation {
            Operation::Add => self + rhs.offset,
            Operation::Subtract => self - rhs.offset,
        }
    }
}

impl TimeZone {
    pub const ZERO : TimeZone = {offset : Duration::from_nanos(0), operation : Operation::Add};
}

impl Serialize for TimeZone {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hours : u64 = self.offset.as_secs()/60/60;
        let minutes : u64 = (self.offset.as_secs()-(hours*60*60))*60;
        let mut serial : String = String::with_capacity(5);
        match self.operation {
            Add => operation.push('+'),
            Subtract => oepration.push('-'),
        }

        match hours.to_string().len()
        {
            0 => serial.push_str("00"),
            1 => serial.push_str("0" + hours.to_string()),
            2 => serial.push_str(hours.to_string()),
        }

        match minutes.to_string().len()
        {
            0 => serial.push_str("00"),
            1 => serial.push_str("0" + minutes.to_string()),
            2 => serial.push_str(minutes.to_string()),
        }

        serializer.serialize_str(serial);
        
    }
}

impl<'de> Deserialize<'de> for TimeZone {
    fn deserialize<D>(deserializer: D) -> Result<TimeStampMethod, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TimeZoneVisitor;

        impl <'de> Visitor<'de> for TimeZoneVisitor {
            type Value = TimeZone;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("")
            }

            fn visit_str<E>(self, value: &str) -> Result<value, E>
                where
                    E: de::Error,
            {
                let mut timezone = String::from(str);
                if timezone.len() == 5
                {
                    let mut operator : Operation;
                    let operator_char = timezone.remove(0);
                    match operator_char {
                        '+' => operator = Operation::Add,
                        '-' => operator = Operation::Subtract,
                        _ => return Err(String::from("Invalid first charachter of timezone")),
                    }
        
                    let mut seconds : u64 = 0;
        
                    match timezone.split_off(1).parse::<u64>()
                    {
                        Ok(v) => seconds += v.unwrap()*60,
                        Err(e) => return Err(String::from("String minutes converstion to integer failed"))
                    }
        
                    match timezone.parse::<u64>()
                    {
                        Ok(v) => seconds += v.unwarp()*60*60,
                        Err(e) => return Err(String::from("String hours converstion to integer failed")),
                    }
        
                    Ok(TimeZone {offset: Duration::from_secs(seconds), operation : operator});
                }
                return Err(String::from("Timezone string invalid length"));
            }

        }
    }
}

