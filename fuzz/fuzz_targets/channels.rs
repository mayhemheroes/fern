#![no_main]
use libfuzzer_sys::fuzz_target;
use log::Level::*;

mod support;

use support::manual_log;

fuzz_target!(|data: &[u8]| {
    match std::str::from_utf8(data) {
        Ok(s) => {
            use std::sync::mpsc;
            // Create the channel
            let (send, recv) = mpsc::channel();

            let (_max_level, logger) = fern::Dispatch::new().chain(send).into_log();

            let l = &*logger;
            manual_log(l, Info, s);
            manual_log(l, Info, s);

            logger.flush();

            assert_eq!(recv.recv().unwrap(), format!("{}\n", s));
            assert_eq!(recv.recv().unwrap(), format!("{}\n", s));
        },
        _ => {},
    }
});
