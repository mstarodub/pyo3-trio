//! PyO3 bindings to various Python asynchronous frameworks.
use std::{
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicUsize, Ordering},
    task::{Context, Poll},
};

use pyo3::{prelude::*, BoundObject};

/// GIL-bound [`Future`].
///
/// Provided with a blanket implementation for [`Future`]. GIL is maintained during polling
/// operation. To release the GIL, see [`AllowThreads`].
pub trait PyFuture: Send + Sync {
    /// GIL-bound [`Future::poll`].
    fn poll_py(self: Pin<&mut Self>, py: Python, cx: &mut Context) -> Poll<PyResult<Py<PyAny>>>;
}

impl<F, T, E> PyFuture for F
where
    F: Future<Output = Result<T, E>> + Send + Sync,
    T: for<'py> IntoPyObject<'py> + Send,
    for<'py> <T as IntoPyObject<'py>>::Error: Into<PyErr>,
    for<'py> <T as IntoPyObject<'py>>::Output: IntoPyObject<'py>,
    E: Send,
    PyErr: From<E>,
{
    fn poll_py(self: Pin<&mut Self>, py: Python, cx: &mut Context) -> Poll<PyResult<Py<PyAny>>> {
        match self.poll(cx) {
            Poll::Ready(Ok(value)) => Poll::Ready(
                value
                    .into_pyobject(py)
                    .map_err(Into::into)
                    .map(|o| o.into_any().unbind()),
            ),
            Poll::Ready(Err(e)) => Poll::Ready(Err(PyErr::from(e))),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Callback for Python coroutine `throw` method (see [`asyncio::Coroutine::new`]) and
/// async generator `athrow` method (see [`asyncio::AsyncGenerator::new`]).
pub type ThrowCallback = Box<dyn FnMut(Python, Option<PyErr>) + Send + Sync>;

// -----

pub(crate) type ThreadId = usize;
pub(crate) fn current_thread_id() -> ThreadId {
    static THREAD_COUNTER: AtomicUsize = AtomicUsize::new(0);
    thread_local! {
        pub(crate) static THREAD_ID: ThreadId = THREAD_COUNTER.fetch_add(1, Ordering::Relaxed);
    }
    THREAD_ID.with(|id| *id)
}
