use crate::config::timestamp::{TimeStampMethod, TimeZone};
use crate::config::Configuration;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use crate::handler::gps;
use std::sync::Mutex;
use std::result::Result;

lazy_static! {
    static ref TIMEZONE: Mutex<TimeZone> = Mutex::new(TimeZone::ZERO);
    static ref METHOD: Mutex<TimeStampMethod> = Mutex::new(TimeStampMethod::GPS);
    static ref START_TIME: Mutex<SystemTime> = Mutex::new(SystemTime::UNIX_EPOCH);
    static ref PREV_COUNTER: Mutex<u32> = Mutex::new(0);
}

pub fn start(config : &Configuration) {
    *METHOD.lock().unwrap() = config.gateway.timestamp_method;
    match config.gateway.timestamp_method {
        TimeStampMethod::GPS => (),
        TimeStampMethod::Systemtime => { *START_TIME.lock().unwrap() = SystemTime::now() },
    }

    *TIMEZONE.lock().unwrap() = config.gateway.timezone;
}

pub fn update_counter(current_count_us : u32)
{
    match *METHOD.lock().unwrap()
    {
        TimeStampMethod::Systemtime => {
            let mut prev_count = PREV_COUNTER.lock().unwrap();
            if current_count_us < *prev_count
            {
                let duration = Duration::from_micros(u32::MAX as u64);
                *START_TIME.lock().unwrap() += duration;
                info!("Timestamp counter wrap around");
            }

            *prev_count = current_count_us;
        },
        _ => (),
    }
}

pub fn calculate_timestamp(current_count_us : u32) -> Result<prost_types::Timestamp, String> {
    match *METHOD.lock().unwrap()
    {
        TimeStampMethod::GPS => {
            match gps::cnt2time(current_count_us) {
                Ok(v) => {
                    let v = v.duration_since(UNIX_EPOCH).unwrap();
        
                    return Ok(prost_types::Timestamp {
                        seconds: v.as_secs() as i64,
                        nanos: v.subsec_nanos() as i32,
                    });
                }
                Err(err) => {
                    return Err(err);
                }
            };
        },
        TimeStampMethod::Systemtime => {
            let time = START_TIME.lock().unwrap();
            let time_since_epoch = 
                (*time + Duration::from_micros(current_count_us as u64) + *TIMEZONE.lock().unwrap())
                .duration_since(UNIX_EPOCH).unwrap();

            update_counter(current_count_us);

            return Ok(prost_types::Timestamp {
                seconds: time_since_epoch.as_secs() as i64,
                nanos: time_since_epoch.subsec_nanos() as i32,
            });
        },
    }
}

pub fn calculate_epochtime(current_count_us : u32) -> Result<prost_types::Duration, String> {
    match *METHOD.lock().unwrap()
    {
        TimeStampMethod::GPS => {
            match gps::cnt2epoch(current_count_us) {
                Ok(v) => {
                    return Ok(prost_types::Duration {
                        seconds: v.as_secs() as i64,
                        nanos: v.subsec_nanos() as i32,
                    });
                }
                Err(err) => {
                    return Err(err);
                }
            }
        },
        TimeStampMethod::Systemtime => {
            let time = START_TIME.lock().unwrap();
            let time_since_epoch = 
                (*time + Duration::from_micros(current_count_us as u64) + *TIMEZONE.lock().unwrap())
                .duration_since(UNIX_EPOCH).unwrap();

            update_counter(current_count_us);

            return Ok(prost_types::Duration {
                seconds: time_since_epoch.as_secs() as i64,
                nanos: time_since_epoch.subsec_nanos() as i32,
            });
        },
    }
}