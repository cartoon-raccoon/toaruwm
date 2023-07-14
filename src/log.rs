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

use crate::{
    XConn, ToaruError, ErrorHandler
};
use crate::manager::WmState;
use tracing::error;

pub(crate) struct DefaultErrorHandler;

impl<X: XConn> ErrorHandler<X> for DefaultErrorHandler {
    fn call(&self, _: WmState<'_, X>, err: ToaruError) {
        error!("{}", err)
    }
}