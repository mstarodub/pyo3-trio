use pyo3::prelude::*;
mod coroutine;
mod pyfuture;
mod trio;

#[pymodule]
fn pyrstest(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(async_sleep, m)?)?;
    Ok(())
}

fn tokio() -> &'static tokio::runtime::Runtime {
    use std::sync::OnceLock;
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| tokio::runtime::Runtime::new().unwrap())
}

async fn sleep(seconds: u64) -> Result<(), PyErr> {
    let sleep = async move { tokio::time::sleep(std::time::Duration::from_secs(seconds)).await };
    tokio().spawn(sleep).await.unwrap();
    Ok(())
}

#[pyfunction]
#[pyo3(name = "sleep")]
fn async_sleep(seconds: u64) -> trio::Coroutine {
    trio::Coroutine::from_future(sleep(seconds))
}
