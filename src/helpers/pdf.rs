use pyo3::ffi::c_str;
use pyo3::prelude::*;

use crate::models::pdf::TimesheetData;

pub fn create_sasi_timesheet(data: TimesheetData) -> Result<Vec<u8>, PyErr> {
    let template_path = concat!(env!("CARGO_MANIFEST_DIR"), "/templates/sasi.pdf");
    let output_path = concat!(env!("CARGO_MANIFEST_DIR"), "/temp/output.pdf");

    let py_app = c_str!(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/pdf_generator/main.py"
    )));
    let from_python = Python::attach(|py| -> PyResult<Py<PyAny>> {
        PyModule::import(py, "PyPDFForm")?;

        let app: Py<PyAny> = PyModule::from_code(py, py_app, c_str!("main.py"), c_str!(""))?
            .getattr("create_sasi_timesheet")?
            .into();

        let res = app.call1(py, (data, template_path, output_path));

        res
    });

    println!("py: {}", from_python?);

    let res_buff = std::fs::read(output_path)?;

    Ok(res_buff)
}
