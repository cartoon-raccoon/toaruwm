use crate::manager::ToaruState;
use crate::config::RuntimeConfig;
use crate::{ErrorHandler, ToaruError};
use tracing::error;

/// The default error handler.
pub struct DefaultErrorHandler;

impl<C> ErrorHandler<C> for DefaultErrorHandler
where
    C: RuntimeConfig,
{
    fn call(&self, _: ToaruState<'_, C>, err: ToaruError) {
        error!("{}", err)
    }
}
