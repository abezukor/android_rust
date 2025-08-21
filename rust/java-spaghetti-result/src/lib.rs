use std::fmt::{Debug, Display};

use java_spaghetti::{CastError, Global, Local, ReferenceType};
use thiserror::Error;

#[allow(mismatched_lifetime_syntaxes)]
#[rustfmt::skip]
mod bindings;
use crate::bindings::java::lang::Throwable;

pub type JavaResult<T> = Result<T, JavaError>;

/// Global Java Error that can be passed between threads.
#[derive(Error, Debug)]
pub enum JavaError {
    #[error(transparent)]
    Throwable(#[from] JavaException),
    #[error(transparent)]
    Cast(#[from] CastError),
}

impl<T: ReferenceType> From<Local<'_, T>> for JavaError {
    fn from(value: Local<'_, T>) -> Self {
        match value.cast::<Throwable>() {
            Ok(err) => Self::Throwable(JavaException(err.as_global())),
            Err(e) => Self::Cast(e),
        }
    }
}

#[derive(Error)]
pub struct JavaException(Global<Throwable>);

impl Debug for JavaException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.vm().with_env(|env| {
            let local_ref = self.0.as_ref(env);
            Debug::fmt(&local_ref, f)
        })
    }
}

impl Display for JavaException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}
