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

use crate::manager::ToaruState;
use crate::config::RuntimeConfig;
use crate::{ErrorHandler, ToaruError, Platform};
use tracing::error;

pub(crate) struct DefaultErrorHandler;

impl<P, C> ErrorHandler<P, C> for DefaultErrorHandler
where
    P: Platform,
    C: RuntimeConfig,
{
    fn call(&self, _: ToaruState<'_, P, C>, err: ToaruError<P>) {
        error!("{}", err)
    }
}
