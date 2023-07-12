// #[cfg(debug_assertions)]
// macro_rules! debug {
//     ($fmt:expr) => {
//         (println!(concat!("[debug] ", $fmt)));
//     };
//     ($fmt:expr, $($arg:tt)*) => {
//         (println!(concat!("[debug] ", $fmt), $($arg)*));
//     };
// }

#[cfg(not(debug_assertions))]
macro_rules! debug {
    ($fmt:expr) => {};
    ($fmt:expr, $($arg:tt)*) => {};
}

// #[cfg(debug_assertions)]
// macro_rules! fn_ends {
//     ($fmt:expr) => {
//         (println!(concat!("================ ", $fmt, " ================")));
//     };
//     ($fmt:expr, $($arg:tt)*) => {
//         (println!(concat!("================ ", $fmt, " ================"), $($arg)*));
//     };
// }

#[cfg(not(debug_assertions))]
macro_rules! fn_ends {
    ($fmt:expr) => {};
    ($fmt:expr, $($arg:tt)*) => {};
}

// macro_rules! info {
//     ($fmt:expr) => {
//         (println!(concat!("[*] ", $fmt)))
//     };
//     ($fmt:expr, $($arg:tt)*) => {
//         (println!(concat!("[*] ", $fmt), $($arg)*))
//     };
// }

// macro_rules! warn {
//     ($fmt:expr) => {
//         (println!(concat!("[!] ", $fmt)))
//     };
//     ($fmt:expr, $($arg:tt)*) => {
//         (println!(concat!("[!] ", $fmt), $($arg)*))
//     };
// }

macro_rules! fatal {
    ($fmt:expr) => {
        (panic!(concat!("[FATAL] ", $fmt)))
    };
    ($fmt:expr, $($arg:tt)*) => {
        (panic!(concat!("[FATAL] ", $fmt), $($arg)*))
    };
}

// macro_rules! error {
//     ($fmt:expr) => {
//         (eprintln!(concat!("[X] ", $fmt)))
//     };
//     ($fmt:expr, $($arg:tt)*) => {
//         (eprintln!(concat!("[X] ", $fmt), $($arg)*))
//     };
// }

use crate::ToaruError;
use tracing::error;

pub(crate) fn basic_error_handler(error: ToaruError) {
    error!("{}", error);
}
