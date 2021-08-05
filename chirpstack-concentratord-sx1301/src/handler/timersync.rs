use std::sync::mpsc::Receiver;
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

use libconcentratord::signals::Signal;
use libloragw_sx1301::{hal, reg, wrapper};

lazy_static! {
    static ref PREV_CONCENTRATOR_COUNT: Mutex<u32> = Mutex::new(hal::get_trigcnt().unwrap());
    static ref PREV_UNIX_TIME: Mutex<Duration> = Mutex::new(
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
    );
    pub static ref USE_GPS_TIME: Mutex<bool> = Mutex::new(true);
    pub static ref START_TIME: Mutex<SystemTime> = Mutex::new(SystemTime::now());
    pub static ref LAST_COUNTER: Mutex<u32> = Mutex::new(0);
}

pub fn timesync_loop(stop_receive: Receiver<Signal>) {
    debug!("Starting timesync loop");

    loop {
        // The timesync is in a separate function to make sure that the
        // mutex guard is dereferenced as soon as the function returns.
        timesync();

        // Instead of a 60s sleep, we receive from the stop channel with a
        // timeout of 60 seconds.
        match stop_receive.recv_timeout(Duration::from_secs(60)) {
            Ok(v) => {
                debug!("Received stop signal, signal: {}", v);
                break;
            }
            _ => {}
        };
    }

    debug!("Timesync loop ended");
}

pub fn get_concentrator_count() -> u32 {
    let prev_concentrator_count = PREV_CONCENTRATOR_COUNT.lock().unwrap();
    let prev_unix_time = PREV_UNIX_TIME.lock().unwrap();

    let unix_diff = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        - *prev_unix_time;

    return prev_concentrator_count.wrapping_add(unix_diff.as_micros() as u32);
}

fn timesync() {
    debug!("Disabling GPS mode for concentrator counter");
    reg::reg_w(wrapper::LGW_GPS_EN, 0).unwrap();

    let mut prev_concentrator_count = PREV_CONCENTRATOR_COUNT.lock().unwrap();
    let mut prev_unix_time = PREV_UNIX_TIME.lock().unwrap();

    let concentrator_count = hal::get_trigcnt().unwrap();
    let unix_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    let unix_time_diff = unix_time - *prev_unix_time;
    let concentrator_diff = {
        let diff: i64;

        if concentrator_count > *prev_concentrator_count {
            diff = (concentrator_count - *prev_concentrator_count) as i64;
        } else {
            diff =
                (concentrator_count as i64) + ((1 << 32) - 1) - (*prev_concentrator_count as i64);
        }

        diff
    };

    let drift = (unix_time_diff.as_micros() as i64) - concentrator_diff;

    *prev_unix_time = unix_time;
    *prev_concentrator_count = concentrator_count;

    let mut counter = LAST_COUNTER.lock().unwrap();
    let mut time = START_TIME.lock().unwrap();

    if *counter > concentrator_count
    {
        let time_passed = Duration::from_micros(u32::MAX as u64 + concentrator_count as u64);
        *time += time_passed;
    }
    *counter = concentrator_count;

    debug!("Current concentrator count_us: {}", concentrator_count);
    debug!("Concentrator drift, drift_us: {}", drift);

    debug!("Enabling GPS mode for concentrator counter");
    reg::reg_w(wrapper::LGW_GPS_EN, 1).unwrap();
}
