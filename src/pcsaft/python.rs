use super::parameters::{
    PcSaftAssociationRecord, PcSaftBinaryRecord, PcSaftParameters, PcSaftRecord,
};
use super::DQVariants;
use crate::association::AssociationRecord;
// use crate::association::PyAssociationRecord;
use feos_core::parameter::{
    BinaryRecord, Identifier, IdentifierOption, Parameter, ParameterError, PureRecord,
    SegmentRecord,
};
use feos_core::python::parameter::*;
use feos_core::*;
use numpy::{PyArray2, PyReadonlyArray2, ToPyArray};
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use std::convert::{TryFrom, TryInto};
use std::sync::Arc;

/// Pure component association parameters
#[pyclass(name = "AssociationRecord")]
#[derive(Clone)]
pub struct PyAssociationRecord(pub AssociationRecord<PcSaftAssociationRecord>);

#[pymethods]
impl PyAssociationRecord {
    #[new]
    #[pyo3(signature = (kappa_ab, epsilon_k_ab, na=0.0, nb=0.0, nc=0.0))]
    fn new(kappa_ab: f64, epsilon_k_ab: f64, na: f64, nb: f64, nc: f64) -> Self {
        Self(AssociationRecord::new(
            PcSaftAssociationRecord::new(kappa_ab, epsilon_k_ab),
            na,
            nb,
            nc,
        ))
    }

    #[getter]
    fn get_kappa_ab(&self) -> f64 {
        self.0.parameters.kappa_ab
    }

    #[getter]
    fn get_epsilon_k_ab(&self) -> f64 {
        self.0.parameters.epsilon_k_ab
    }

    #[getter]
    fn get_na(&self) -> f64 {
        self.0.na
    }

    #[getter]
    fn get_nb(&self) -> f64 {
        self.0.nb
    }

    #[getter]
    fn get_nc(&self) -> f64 {
        self.0.nc
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(self.0.to_string())
    }
}

impl_json_handling!(PyAssociationRecord);

/// Pure-substance parameters for the PC-Saft equation of state.
///
/// Parameters
/// ----------
/// m : float
///     Segment number
/// sigma : float
///     Segment diameter in units of Angstrom.
/// epsilon_k : float
///     Energetic parameter in units of Kelvin.
/// mu : float, optional
///     Dipole moment in units of Debye.
/// q : float, optional
///     Quadrupole moment in units of Debye * Angstrom.
/// kappa_ab : float, optional
///     Association volume parameter.
/// epsilon_k_ab : float, optional
///     Association energy parameter in units of Kelvin.
/// na : float, optional
///     Number of association sites of type A.
/// nb : float, optional
///     Number of association sites of type B.
/// nc : float, optional
///     Number of association sites of type C.
/// association_records : List[AssociationRecord], optional
///     A list of association records, if the molecule has more than one association site.
/// viscosity : List[float], optional
///     Entropy-scaling parameters for viscosity. Defaults to `None`.
/// diffusion : List[float], optional
///     Entropy-scaling parameters for diffusion. Defaults to `None`.
/// thermal_conductivity : List[float], optional
///     Entropy-scaling parameters for thermal_conductivity. Defaults to `None`.
#[pyclass(name = "PcSaftRecord")]
#[derive(Clone)]
pub struct PyPcSaftRecord(PcSaftRecord);

#[pymethods]
impl PyPcSaftRecord {
    #[new]
    #[pyo3(
        text_signature = "(m, sigma, epsilon_k, mu=None, q=None, kappa_ab=None, epsilon_k_ab=None, na=None, nb=None, nc=None, association_records=None, viscosity=None, diffusion=None, thermal_conductivity=None)"
    )]
    fn new(
        m: f64,
        sigma: f64,
        epsilon_k: f64,
        mu: Option<f64>,
        q: Option<f64>,
        kappa_ab: Option<f64>,
        epsilon_k_ab: Option<f64>,
        na: Option<f64>,
        nb: Option<f64>,
        nc: Option<f64>,
        association_records: Option<Vec<PyAssociationRecord>>,
        viscosity: Option<[f64; 4]>,
        diffusion: Option<[f64; 5]>,
        thermal_conductivity: Option<[f64; 4]>,
    ) -> Self {
        Self(PcSaftRecord::new(
            m,
            sigma,
            epsilon_k,
            mu,
            q,
            kappa_ab,
            epsilon_k_ab,
            na,
            nb,
            nc,
            association_records
                .into_iter()
                .flat_map(|r| r.into_iter().map(|r| r.0))
                .collect(),
            viscosity,
            diffusion,
            thermal_conductivity,
        ))
    }

    #[getter]
    fn get_m(&self) -> f64 {
        self.0.m
    }

    #[getter]
    fn get_sigma(&self) -> f64 {
        self.0.sigma
    }

    #[getter]
    fn get_epsilon_k(&self) -> f64 {
        self.0.epsilon_k
    }

    #[getter]
    fn get_mu(&self) -> Option<f64> {
        self.0.mu
    }

    #[getter]
    fn get_q(&self) -> Option<f64> {
        self.0.q
    }

    // #[getter]
    // fn get_association_records(&self) -> Vec<PyAssociationRecord> {
    //     self.0
    //         .association_records
    //         .iter()
    //         .copied()
    //         .map(PyAssociationRecord)
    //         .collect()
    // }

    #[getter]
    fn get_viscosity(&self) -> Option<[f64; 4]> {
        self.0.viscosity
    }

    #[getter]
    fn get_diffusion(&self) -> Option<[f64; 5]> {
        self.0.diffusion
    }

    #[getter]
    fn get_thermal_conductivity(&self) -> Option<[f64; 4]> {
        self.0.thermal_conductivity
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(self.0.to_string())
    }
}

impl_json_handling!(PyPcSaftRecord);

impl_pure_record!(PcSaftRecord, PyPcSaftRecord);
impl_segment_record!(PcSaftRecord, PyPcSaftRecord);

#[pyclass(name = "PcSaftBinaryRecord")]
#[derive(Clone)]
pub struct PyPcSaftBinaryRecord(PcSaftBinaryRecord);

#[pymethods]
impl PyPcSaftBinaryRecord {
    #[new]
    fn new(k_ij: Option<f64>, kappa_ab: Option<f64>, epsilon_k_ab: Option<f64>) -> Self {
        Self(PcSaftBinaryRecord::new(k_ij, kappa_ab, epsilon_k_ab))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(self.0.to_string())
    }
}

impl_json_handling!(PyPcSaftBinaryRecord);

impl_binary_record!(PcSaftBinaryRecord, PyPcSaftBinaryRecord);

#[pyclass(name = "PcSaftParameters")]
#[derive(Clone)]
pub struct PyPcSaftParameters(pub Arc<PcSaftParameters>);

impl_parameter!(
    PcSaftParameters,
    PyPcSaftParameters,
    PyPcSaftRecord,
    PyPcSaftBinaryRecord
);
impl_parameter_from_segments!(PcSaftParameters, PyPcSaftParameters);

#[pymethods]
impl PyPcSaftParameters {
    #[getter]
    fn get_k_ij<'py>(&self, py: Python<'py>) -> Option<Bound<'py, PyArray2<f64>>> {
        self.0
            .binary_records
            .as_ref()
            .map(|br| br.map(|br| br.k_ij).view().to_pyarray_bound(py))
    }

    fn _repr_markdown_(&self) -> String {
        self.0.to_markdown()
    }
}

#[pymodule]
pub fn pcsaft(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyIdentifier>()?;
    m.add_class::<IdentifierOption>()?;
    m.add_class::<PyChemicalRecord>()?;
    m.add_class::<PySmartsRecord>()?;
    m.add_class::<PyAssociationRecord>()?;

    m.add_class::<DQVariants>()?;
    m.add_class::<PyPcSaftRecord>()?;
    m.add_class::<PyPcSaftBinaryRecord>()?;
    m.add_class::<PyPureRecord>()?;
    m.add_class::<PySegmentRecord>()?;
    m.add_class::<PyBinaryRecord>()?;
    m.add_class::<PyBinarySegmentRecord>()?;
    m.add_class::<PyPcSaftParameters>()?;
    Ok(())
}
