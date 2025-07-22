use std::{
    sync::Mutex,
    task::{Context, Poll},
};

use futures_channel::oneshot::{self, Canceled};
use futures_lite::FutureExt;
use java_spaghetti::{sys::jobject, Env, Local, Ref};
use log::error;
use rust_android_utilities::java_wrapped_object::{get_ref, to_java};

use crate::bindings::com::maticrobots::rust_android_utilities::RustArcBoxDynAny;

pub struct CallBackFuture<T: Send> {
    recv: oneshot::Receiver<T>,
}

pub struct CallBackFutureData<T: Send> {
    sender: Mutex<Option<oneshot::Sender<T>>>,
}

impl<T: Send + Sync + 'static> CallBackFuture<T> {
    pub fn new(env: Env<'_>) -> (Local<'_, RustArcBoxDynAny>, Self) {
        let (data_tx, recv) = oneshot::channel();
        let in_java = to_java(env, CallBackFutureData { sender: Mutex::new(Some(data_tx)) });
        (in_java.unwrap().cast().unwrap(), Self { recv })
    }
}

impl<T: Send + 'static> Future for CallBackFuture<T> {
    type Output = T;

    fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.recv.poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(output)) => Poll::Ready(output),
            Poll::Ready(Err(Canceled)) => {
                unreachable!("Java has dropped the sender before it sent data");
            }
        }
    }
}

impl<T: Send + 'static> CallBackFutureData<T> {
    /// SAFETY: `rust_obj` must be a `jobject` that represents a java RustArcBoxDynAny
    pub unsafe fn wake(env: Env<'_>, rust_obj: jobject, value: T) {
        //spurious callback with no associated rust future
        if rust_obj.is_null() {
            error!("Attempting to wake up a null CallBackFutureData");
            return;
        }
        let rust_obj: Ref<RustArcBoxDynAny> = unsafe { Ref::from_raw(env, rust_obj) };
        let rust_ptr = rust_obj.getRust_ptr().unwrap();
        let rust_obj = unsafe { get_ref(rust_ptr) };
        let this = rust_obj.downcast_ref::<Self>().unwrap();
        let sender = this.sender.lock().unwrap().take();
        let Some(sender) = sender else {
            error!(
                "Java has tried to wake up a callback future. `wake` should only be called on a callback future once"
            );
            return;
        };
        // We dont care if the receiver is dropped
        let _ = sender.send(value);
    }
}
