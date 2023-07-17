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

use crate::manager::{WmState, RuntimeConfig};
use crate::{ErrorHandler, ToaruError, XConn};
use tracing::error;

pub(crate) struct DefaultErrorHandler;

impl<X, C> ErrorHandler<X, C> for DefaultErrorHandler
where
    X: XConn,
    C: RuntimeConfig
{
    fn call(&self, _: WmState<'_, X, C>, err: ToaruError) {
        error!("{}", err)
    }
}
