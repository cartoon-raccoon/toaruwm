macro_rules! fatal {
    ($fmt:expr) => {
        (panic!(concat!("[FATAL] ", $fmt)))
    };
    ($fmt:expr, $($arg:tt)*) => {
        (panic!(concat!("[FATAL] ", $fmt), $($arg)*))
    };
}

extern crate tracing;

macro_rules! trace {
    ($fmt:expr) => {
        #[cfg(debug_assertions)]
        tracing::trace!($fmt)
    };
    ($fmt:expr, $($arg:tt)*) => {
        #[cfg(debug_assertions)]
        tracing::trace!($fmt, $($arg)*)
    }
}

use crate::manager::{RuntimeConfig, WmState};
use crate::{ErrorHandler, ToaruError, XConn};
use tracing::error;

pub(crate) struct DefaultErrorHandler;

impl<X, C> ErrorHandler<X, C> for DefaultErrorHandler
where
    X: XConn,
    C: RuntimeConfig,
{
    fn call(&self, _: WmState<'_, X, C>, err: ToaruError) {
        error!("{}", err)
    }
}
