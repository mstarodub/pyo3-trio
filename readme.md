## pyo3-trio

**goal**: make it convenient to call async rust functions from async python. only trio is supported (no asyncio).

this is derivative work of https://github.com/wyfo/pyo3-async. all credit for the original work goes to wyfo.

the apis have been updated for a new pyo3, and the relevant trio bits extracted.


### todo:
- fix ctrl c issue
- write the macros / ergonomics
- make it usable as a library
- tokio dep
