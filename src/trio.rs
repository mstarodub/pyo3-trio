//! `trio` compatible coroutine and async generator implementation.
use pyo3::{exceptions::PyStopIteration, intern, prelude::*};

use crate::coroutine;

macro_rules! module {
    ($name:ident ,$path:literal, $($field:ident),* $(,)?) => {
        #[allow(non_upper_case_globals)]
        static $name: ::pyo3::sync::PyOnceLock<$name> = ::pyo3::sync::PyOnceLock::new();

        #[allow(non_snake_case)]
        struct $name {
            $($field: Py<PyAny>),*
        }

        impl $name {
            fn get(py: Python) -> PyResult<&Self> {
                $name.get_or_try_init(py, || {
                    let module = py.import($path)?;
                    Ok(Self {
                        $($field: module.getattr(stringify!($field))?.into(),)*
                    })
                })
            }
        }
    };
}

module!(
    Trio,
    "trio.lowlevel",
    Abort,
    current_task,
    current_trio_token,
    reschedule,
    wait_task_rescheduled
);

pub(crate) struct Waker {
    task: Py<PyAny>,
    token: Py<PyAny>,
}

impl coroutine::CoroutineWaker for Waker {
    fn new(py: Python) -> PyResult<Self> {
        let trio = Trio::get(py)?;
        Ok(Waker {
            task: trio.current_task.call0(py)?,
            token: trio.current_trio_token.call0(py)?,
        })
    }

    fn yield_(&self, py: Python) -> PyResult<Py<PyAny>> {
        Trio::get(py)?
            .wait_task_rescheduled
            .call1(py, (wrap_pyfunction!(abort_func, py)?,))?
            .call_method0(py, intern!(py, "__await__"))?
            .call_method0(py, intern!(py, "__next__"))
    }

    fn wake(&self, py: Python) {
        let reschedule = &Trio::get(py).unwrap().reschedule;
        reschedule
            .call1(py, (&self.task,))
            .expect("unexpected error while calling trio.lowlevel.reschedule");
    }

    fn wake_threadsafe(&self, py: Python) {
        let reschedule = &Trio::get(py).unwrap().reschedule;
        self.token
            .call_method1(py, intern!(py, "run_sync_soon"), (reschedule, &self.task))
            .expect("unexpected error while scheduling TrioToken.run_sync_soon");
    }
}

#[pyfunction]
fn abort_func(py: Python, _arg: Py<PyAny>) -> PyResult<Py<PyAny>> {
    Trio::get(py)?.Abort.getattr(py, intern!(py, "SUCCEEDED"))
}

/// Python coroutine wrapping a [`PyFuture`](crate::PyFuture).
#[pyclass]
pub struct Coroutine(crate::coroutine::Coroutine<Waker>);

impl Coroutine {
    /// Wrap a boxed future in to a Python coroutine.
    ///
    /// If `throw` callback is provided:
    /// - coroutine `throw` method will call it with the passed exception before polling;
    /// - coroutine `close` method will call it with `None` before polling and dropping
    ///   the future.
    /// If `throw` callback is not provided, the future will dropped without additional
    /// poll.
    pub fn new(
        future: ::std::pin::Pin<Box<dyn crate::pyfuture::PyFuture>>,
        throw: Option<crate::pyfuture::ThrowCallback>,
    ) -> Self {
        Self(crate::coroutine::Coroutine::new(future, throw))
    }

    /// Wrap a generic future into a Python coroutine.
    pub fn from_future(future: impl crate::pyfuture::PyFuture + 'static) -> Self {
        Self::new(Box::pin(future), None)
    }
}

#[pymethods]
impl Coroutine {
    fn send(&mut self, py: Python, _value: Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        match self.0.poll(py, None)? {
            Some(value) => Ok(value),
            None => Err(PyStopIteration::new_err(())),
        }
    }

    fn throw(&mut self, py: Python, exc: Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        let py_err: PyErr = PyErr::from_value(exc);
        match self.0.poll(py, Some(py_err))? {
            Some(value) => Ok(value),
            None => Err(PyStopIteration::new_err(())),
        }
    }

    fn close(&mut self, py: Python) -> PyResult<()> {
        self.0.close(py)
    }

    fn __await__(slf: Bound<'_, Self>) -> PyResult<Bound<'_, PyAny>> {
        Ok(slf.into_any())
    }

    fn __iter__(slf: Bound<'_, Self>) -> PyResult<Bound<'_, PyAny>> {
        Ok(slf.into_any())
    }

    fn __next__(&mut self, py: Python) -> PyResult<Option<Py<PyAny>>> {
        self.0.poll(py, None)
    }
}
