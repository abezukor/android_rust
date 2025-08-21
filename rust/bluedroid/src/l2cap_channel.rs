// Originally from https://github.com/alexmoon/bluest/blob/ea0386d8f21bbb4ff3a9676d522e209b824b345f/src/android/l2cap_channel.rs, written by
// https://github.com/Dirbaio

use std::{
    collections::VecDeque,
    hash::{DefaultHasher, Hasher},
    io,
    marker::PhantomData,
    pin::{pin, Pin},
    sync::{mpsc, Arc},
    task::{Context, Poll},
    thread,
};

use futures_channel::oneshot::{self, Canceled};
use futures_io::{AsyncRead, AsyncWrite};
use futures_lite::FutureExt;
use java_spaghetti::{ByteArray, Global, Local, PrimitiveArray};
use log::{debug, trace};
use java_spaghetti_result::{JavaError, JavaResult};

use crate::{
    bindings::{
        android::bluetooth::BluetoothDevice as JavaBluetoothDevice,
        java::io::{InputStream, OutputStream},
    },
    rust_slice_to_java_byte_array,
};

pub struct Channel<H: Hasher + Default = DefaultHasher> {
    reader: Reader,
    writer: Writer<H>,
}

#[cfg(not(test))]
mod closer;
#[cfg(not(test))]
use closer::L2capCloser;
#[cfg(test)]
use tests::L2capCloser;

type ReaderResponse = io::Result<Box<[u8]>>;
type ReaderRequest = (usize, oneshot::Sender<ReaderResponse>);
pub struct Reader {
    request: mpsc::Sender<ReaderRequest>,
    data_recv: Option<oneshot::Receiver<ReaderResponse>>,
    /// This is an overflow buffer that should only be used in a very specific case
    /// It should only be used when there is a read-request, that gets canclled, and then another
    /// smaller read request comes through. When that happens we may get a packet that is bigger
    /// then the new read, and thus we need to buffer the rest of the result until another read comes and can finish reading it.
    cancel_buffer: VecDeque<u8>,
    _closer: Arc<L2capCloser>,
}

type Hash = u64;
type WriterResponse = (usize, Hash, io::Result<()>);
type WriterRequest = (Box<[u8]>, Hash, oneshot::Sender<WriterResponse>);
pub struct Writer<H: Hasher + Default = DefaultHasher> {
    request: mpsc::Sender<WriterRequest>,
    done: Option<oneshot::Receiver<WriterResponse>>,
    _hasher: PhantomMarker<H>,
    closer: Arc<L2capCloser>,
}

/// Marker Type that is always Unpin, No matter T
#[derive(Debug, Default)]
struct PhantomMarker<T>(PhantomData<T>);
impl<T> Unpin for PhantomMarker<T> {}

impl<H: Hasher + Default> Channel<H> {
    pub(crate) fn open_l2cap_channel(device: Global<JavaBluetoothDevice>, psm: u16, secure: bool) -> JavaResult<Self> {
        device.vm().with_env(|env| {
            let device = device.as_local(env);

            let channel = if secure {
                device.createL2capChannel(psm as _)?.unwrap()
            } else {
                device.createInsecureL2capChannel(psm as _)?.unwrap()
            };

            channel.connect()?;

            // The L2capCloser closes the l2cap channel when dropped.
            // We put it in an Arc held by both the reader and writer, so it gets dropped
            // when both the reader and write are dropped.
            #[cfg(not(test))]
            let closer = Arc::new(L2capCloser { channel: channel.as_global() });

            let input_stream = channel.getInputStream()?.unwrap().as_global();
            let output_stream = channel.getOutputStream()?.unwrap().as_global();

            let (reader_request_tx, reader_request_rx) = mpsc::channel();
            let reader = Reader {
                request: reader_request_tx,
                data_recv: None,
                cancel_buffer: VecDeque::new(),
                #[cfg(not(test))]
                _closer: closer.clone(),
                #[cfg(test)]
                _closer: Default::default(),
            };

            let (writer_request_tx, writer_request_rx) = mpsc::channel();
            let writer = Writer::<H> {
                request: writer_request_tx,
                done: None,
                _hasher: Default::default(),
                #[cfg(not(test))]
                closer,
                #[cfg(test)]
                closer: Default::default(),
            };

            // Unfortunately, Android's API for L2CAP channels is only blocking. Only way to deal with it
            // is to launch two background threads with blocking loops for reading and writing, which communicate
            // with the async Rust world via async channels.
            //
            // The loops stop when either Android returns an error (for example if the channel is closed), or the
            // async channel gets closed because the user dropped the reader or writer structs.
            {
                thread::spawn(move || read_thread(reader_request_rx, input_stream));

                thread::spawn(move || write_thread(writer_request_rx, output_stream));
            }

            Ok::<_, JavaError>(Self { reader, writer })
        })
    }

    pub fn split(self) -> (Reader, Writer<H>) {
        let Self { reader, writer } = self;
        (reader, writer)
    }
}

fn read_thread(request: mpsc::Receiver<ReaderRequest>, input_stream: Global<InputStream>) {
    debug!("l2cap read thread running!");

    input_stream.vm().with_env(|env| {
        let stream = input_stream.as_local(env);

        loop {
            let Ok((size, responder)) = request.recv() else {
                trace!("Read thread exiting due to request channel closed");
                break;
            };
            let arr: Local<ByteArray> = ByteArray::new(env, size);
            let response = match stream.read_byte_array(&arr) {
                Ok(err) if err < 0 => Err(io::Error::other(format!(
                    "Got an invalid number of bytes {err} from the channel"
                ))),
                Err(e) => Err(io::Error::other(format!("failed to read from l2cap channel: {e:?}"))),
                Ok(received_size) => {
                    let received_size = received_size as usize;
                    assert!(received_size <= size, "Read buffer must be less then data length");
                    let mut data = vec![0u8; received_size];
                    arr.get_region(0, u8toi8_mut(&mut data));
                    trace!("Read thread got {} bytes", data.len());
                    Ok(data.into_boxed_slice())
                }
            };
            if responder.send(response).is_err() {
                trace!("Read thread exiting due to response channel closed");
                break;
            }
        }
    });

    debug!("l2cap read thread exiting!");
}

fn write_thread(request: mpsc::Receiver<WriterRequest>, output_stream: Global<OutputStream>) {
    debug!("l2cap write thread running!");

    output_stream.vm().with_env(|env| {
        let stream = output_stream.as_local(env);

        loop {
            let Ok((data, hash, responder)) = request.recv() else {
                trace!("Write thread closing due to request channel");
                break;
            };

            let b = rust_slice_to_java_byte_array(env, &data);

            let response = (
                data.len(),
                hash,
                match stream.write_byte_array(b) {
                    Ok(()) => Ok(()),
                    Err(e) => Err(io::Error::other(format!("failed to read from l2cap channel: {e:?}"))),
                },
            );
            if responder.send(response).is_err() {
                trace!("Write thread closing due to response channel.");
                break;
            }
        }
    });

    debug!("l2cap write thread exiting!");
}

impl AsyncRead for Reader {
    /// This implementation should be cancel safe in that it should never loose data (i.e) calles will always eventually get all data that is read from the channel.
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<io::Result<usize>> {
        let cancel_buffer_has_data = !self.cancel_buffer.is_empty();
        match (self.data_recv.as_mut(), cancel_buffer_has_data) {
            (Some(recv), false) => match recv.poll(cx) {
                Poll::Ready(response) => {
                    self.data_recv = None;
                    let response = response.expect("Read thread unexpectdly closed when awaiting response");
                    match response {
                        Ok(data) => {
                            let return_len = std::cmp::min(data.len(), buf.len());
                            if return_len < data.len() {
                                self.cancel_buffer.extend(data[return_len..].iter());
                            }
                            buf[..return_len].copy_from_slice(&data[..return_len]);
                            Poll::Ready(Ok(return_len))
                        }
                        Err(e) => Poll::Ready(Err(e)),
                    }
                }
                Poll::Pending => Poll::Pending,
            },
            (Some(_), true) => {
                unreachable!("We should never have anything in the cancel buffer if we have a pending request. After a request fills the cancel buffer, it is drained before another request can be queued.");
            }
            (None, true) => {
                let data_len = std::cmp::min(self.cancel_buffer.len(), buf.len());

                // Take take the first data_len elements. Annoyingly `split_off` does the opposit of what I want
                // So we just swap it with the return value
                let cancel_buffer = self.cancel_buffer.split_off(data_len);
                let mut return_buffer = std::mem::replace(&mut self.cancel_buffer, cancel_buffer);

                let return_slice = return_buffer.make_contiguous();
                buf[..data_len].copy_from_slice(return_slice);
                Poll::Ready(Ok(data_len))
            }
            (None, false) => {
                // New request
                let (response_tx, response_rx) = oneshot::channel();
                let request = (buf.len(), response_tx);
                self.request
                    .send(request)
                    .expect("Read thread unexpectdly closed when trying to request");
                self.data_recv = Some(response_rx);
                self.poll_read(cx, buf)
            }
        }
    }
}

impl<H: Default + Hasher> Writer<H> {
    fn hash_data(data: &[u8]) -> Hash {
        let mut hasher = H::default();
        hasher.write(data);
        hasher.finish()
    }
}

impl<H: Hasher + Default + Unpin> AsyncWrite for Writer<H> {
    /// This implementation is semi-cancel safe. It makes the guarentee that any data that gets a `Poll::Ready` Response has been sent on the channel.
    /// Any data that has been polled may or may not be written to the channel. Note that this is minorly incompatible with tokio::AsyncWriteExt cancel safety guarentees.
    /// Thanks android for not providing any sort of async io apis on l2cap streams
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<io::Result<usize>> {
        match self.done.as_mut() {
            Some(recv) => match recv.poll(cx) {
                Poll::Ready(Ok((received_len, received_hash, result))) => {
                    self.done = None;
                    // This seems to be the same request that was completed
                    if received_len == buf.len() && received_hash == Self::hash_data(buf) {
                        Poll::Ready(result.map(|()| received_len))
                    } else {
                        // This seems to be a different write
                        self.poll_write(cx, buf)
                    }
                }
                Poll::Ready(Err(Canceled)) => {
                    panic!("Write thread unexpectdly closed when awaiting response");
                }
                Poll::Pending => Poll::Pending,
            },
            None => {
                // New request
                let (response_tx, response_rx) = oneshot::channel();
                let request = (buf.to_vec().into_boxed_slice(), Self::hash_data(buf), response_tx);
                self.request
                    .send(request)
                    .expect("Write thread unexpectdly closed when trying to request");
                self.done = Some(response_rx);
                self.poll_write(cx, buf)
            }
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.closer.close();
        Poll::Ready(Ok(()))
    }
}

impl AsyncRead for Channel {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<io::Result<usize>> {
        AsyncRead::poll_read(pin!(&mut self.reader), cx, buf)
    }
}

impl AsyncWrite for Channel {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<io::Result<usize>> {
        pin!(&mut self.writer).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        pin!(&mut self.writer).poll_flush(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        pin!(&mut self.writer).poll_close(cx)
    }
}

fn u8toi8_mut(slice: &mut [u8]) -> &mut [i8] {
    let len = slice.len();
    let data = slice.as_mut_ptr() as *mut i8;
    // safety: any bit pattern is valid for u8 and i8, so transmuting them is fine.
    unsafe { std::slice::from_raw_parts_mut(data, len) }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::VecDeque,
        hash::DefaultHasher,
        sync::mpsc,
        task::{Context, Poll, Waker},
    };

    use futures_io::{AsyncRead, AsyncWrite};

    #[derive(Debug, Default)]
    pub struct L2capCloser {}
    impl L2capCloser {
        pub const fn close(&self) {}
    }

    #[test]
    fn read_cancel_overflow_test() {
        let (request_tx, request_rx) = mpsc::channel();
        let reader = super::Reader {
            request: request_tx,
            data_recv: None,
            cancel_buffer: VecDeque::new(),
            _closer: Default::default(),
        };

        let mut reader = std::pin::pin!(reader);

        let mut context = Context::from_waker(Waker::noop());
        let mut big_read = [0; 10];
        assert!(matches!(
            reader.as_mut().poll_read(&mut context, &mut big_read),
            Poll::Pending
        ));

        let (_, response) = request_rx.try_recv().unwrap();
        response
            .send(Ok(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10].into_boxed_slice()))
            .unwrap();

        let mut small_read = [0u8; 5];
        assert!(matches!(
            reader.as_mut().poll_read(&mut context, &mut small_read),
            Poll::Ready(Ok(5))
        ));
        assert_eq!(small_read, [1, 2, 3, 4, 5]);

        let mut small_read = [0u8; 3];
        assert!(matches!(
            reader.as_mut().poll_read(&mut context, &mut small_read),
            Poll::Ready(Ok(3))
        ));
        assert_eq!(small_read, [6, 7, 8]);

        let mut small_read = [0u8; 2];
        assert!(matches!(
            reader.as_mut().poll_read(&mut context, &mut small_read),
            Poll::Ready(Ok(2))
        ));
        assert_eq!(small_read, [9, 10]);

        assert!(matches!(
            reader.as_mut().poll_read(&mut context, &mut [0u8; 10]),
            Poll::Pending
        ));
    }

    #[test]
    fn write_len_changed_test() {
        let (request_tx, request_rx) = mpsc::channel();
        let writer = super::Writer::<DefaultHasher> {
            request: request_tx,
            done: None,
            _hasher: Default::default(),
            closer: Default::default(),
        };

        let mut writer = std::pin::pin!(writer);
        let mut cx = Context::from_waker(Waker::noop());

        assert!(matches!(
            writer.as_mut().poll_write(&mut cx, &[1, 2, 3, 4, 5, 6, 7, 8]),
            Poll::Pending
        ));

        let (request_data, hash, responder) = request_rx.try_recv().unwrap();

        assert!(matches!(
            writer.as_mut().poll_write(&mut cx, &[3, 4, 5,]),
            Poll::Pending
        ));

        responder.send((request_data.len(), hash, Ok(()))).unwrap();

        // This is the importent check
        assert!(matches!(
            writer.as_mut().poll_write(&mut cx, &[3, 4, 5,]),
            Poll::Pending
        ));

        let (request_data, hash, responder) = request_rx.try_recv().unwrap();
        responder.send((request_data.len(), hash, Ok(()))).unwrap();

        assert!(matches!(
            writer.as_mut().poll_write(&mut cx, &[3, 4, 5,]),
            Poll::Ready(Ok(3))
        ));
    }
}
