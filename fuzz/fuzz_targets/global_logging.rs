#![no_main]
use libfuzzer_sys::fuzz_target;

use std::sync::{Arc, Mutex};

use log::{debug, error, info, trace, warn};

/// Custom logger built to verify our exact test case.
struct LogVerify {
    info: bool,
    warn: bool,
    error: bool,
}

impl LogVerify {
    fn new() -> Self {
        LogVerify {
            info: false,
            warn: false,
            error: false,
        }
    }
    fn log(&mut self, record: &log::Record) {
        let formatted_message = format!("{}", record.args());
        if formatted_message.starts_with("[INFO] ") {
            assert_eq!(self.info, false, "expected only one info message");
            self.info = true;
        } else if formatted_message.starts_with("[WARN] ") {
            assert_eq!(self.warn, false, "expected only one warn message");
            self.warn = true;
        } else if formatted_message.starts_with("[ERROR] ") {
            assert_eq!(self.error, false, "expected only one error message");
            self.error = true;
        } else {
            panic!();
        }
    }
}
/// Wrapper for our verification which acts as the actual logger.
#[derive(Clone)]
struct LogVerifyWrapper(Arc<Mutex<LogVerify>>);

impl LogVerifyWrapper {
    fn new() -> Self {
        LogVerifyWrapper(Arc::new(Mutex::new(LogVerify::new())))
    }

    fn cloned_boxed_logger(&self) -> Box<dyn log::Log> {
        Box::new(self.clone())
    }
}

impl log::Log for LogVerifyWrapper {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }
    fn log(&self, record: &log::Record) {
        self.0.lock().unwrap().log(record);
    }
    fn flush(&self) {}
}

fuzz_target!(|data: &[u8]| {
    match std::str::from_utf8(data) {
        Ok(s) => {
            let verify = LogVerifyWrapper::new();

            // Create a basic logger configuration
            fern::Dispatch::new()
                .format(|out, msg, record| out.finish(
                    format_args!("[{}] {}", record.level(), msg)
                ))
                // Only log messages Info and above
                .level(log::LevelFilter::Info)
                // Output to our verification logger for verification
                .chain(verify.cloned_boxed_logger())
                .apply()
                .expect("Failed to initialize logger: global logger already set!");

            trace!("{}", s);
            debug!("{}", s);
            info!("{}", s);
            warn!("{}", s);
            error!("{}", s);

            // ensure all buffers are flushed.
            log::logger().flush();

            let verify_acquired = verify.0.lock().unwrap();
            assert_eq!(
                verify_acquired.info, true,
                "expected info message to be received"
            );
            assert_eq!(
                verify_acquired.warn, true,
                "expected warn message to be received"
            );
            assert_eq!(
                verify_acquired.error, true,
                "expected error message to be received"
            );
        },
        _ => {},
    }
});
