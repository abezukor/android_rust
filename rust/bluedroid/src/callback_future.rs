use std::{
    sync::Mutex,
    task::{Context, Poll},
};

use async_lock::MutexGuardArc;
use futures_channel::oneshot::{self, Canceled};
use futures_lite::FutureExt;
use java_spaghetti::{sys::jobject, Env, Local, Ref};
use log::error;
use java_owned_rust_object::{get_ref, to_java};

use crate::bindings::com::maticrobots::rust_android_utilities::RustArcBoxDynAny;

pub struct CallBackFuture<T: Send> {
    recv: oneshot::Receiver<T>,
}

pub struct CallBackFutureData<T: Send>(Mutex<CallBackFutureDataInner<T>>);

struct CallBackFutureDataInner<T: Send> {
    sender: Option<oneshot::Sender<T>>,
    gatt_lock: Option<MutexGuardArc<()>>,
}

impl<T: Send + Sync + 'static> CallBackFuture<T> {
    #[inline]
    pub fn new(env: Env<'_>) -> (Local<'_, RustArcBoxDynAny>, Self) {
        Self::new_inner(env, None)
    }

    #[inline]
    /// The gatt lock needs to be held until the callback happens, so it needs to be owned by the future data in case the parent future is cancelled.
    pub fn new_locked(env: Env<'_>, gatt_lock: MutexGuardArc<()>) -> (Local<'_, RustArcBoxDynAny>, Self) {
        Self::new_inner(env, Some(gatt_lock))
    }

    fn new_inner(env: Env<'_>, gatt_lock: Option<MutexGuardArc<()>>) -> (Local<'_, RustArcBoxDynAny>, Self) {
        let (data_tx, recv) = oneshot::channel();
        let in_java = to_java(env, CallBackFutureData::new(data_tx, gatt_lock));
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
    fn new(data_tx: oneshot::Sender<T>, gatt_lock: Option<MutexGuardArc<()>>) -> Self {
        Self(Mutex::new(CallBackFutureDataInner { sender: Some(data_tx), gatt_lock }))
    }

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
        let mut inner = this.0.lock().unwrap();

        let sender = inner.sender.take();
        if let Some(sender) = sender {
            // We dont care if the receiver is dropped
            let _ = sender.send(value);
        } else {
            error!(
                "Java has tried to wake up a callback future. `wake` should only be called on a callback future once"
            );
        }

        // Release the gatt lock since the future has been called, if there was no lock, this is a no-op.
        let _ = inner.gatt_lock.take();
    }
}
