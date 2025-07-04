use crate::manager::ToaruState;
use crate::config::RuntimeConfig;
use crate::{ErrorHandler, ToaruError, Platform};
use tracing::error;

/// The default error handler.
pub struct DefaultErrorHandler;

impl<P, C> ErrorHandler<P, C> for DefaultErrorHandler
where
    P: Platform,
    C: RuntimeConfig,
{
    fn call(&self, _: ToaruState<'_, P, C>, err: ToaruError) {
        error!("{}", err)
    }
}
