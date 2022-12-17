//! The [`Estimator`] struct can be used to store multiple [`DataSet`]s for convenient parameter
//! optimization.
use super::{DataSet, EstimatorError, Loss};
use feos_core::EosUnit;
use feos_core::EquationOfState;
use ndarray::{arr1, concatenate, Array1, ArrayView1, Axis};
use quantity::QuantityArray1;
use quantity::QuantityScalar;
#[cfg(feature = "rayon")]
use rayon::{prelude::*, ThreadPool};
use std::fmt;
use std::fmt::Display;
use std::fmt::Write;
use std::sync::Arc;

/// A collection of [`DataSet`]s and weights that can be used to
/// evaluate an equation of state versus experimental data.
pub struct Estimator<U: EosUnit, E: EquationOfState> {
    data: Vec<Arc<dyn DataSet<U, E>>>,
    weights: Vec<f64>,
    losses: Vec<Loss>,
}

impl<U: EosUnit, E: EquationOfState> Estimator<U, E>
where
    QuantityScalar<U>: std::fmt::Display + std::fmt::LowerExp,
{
    /// Create a new `Estimator` given `DataSet`s and weights.
    ///
    /// The weights are normalized and used as multiplicator when the
    /// cost function across all `DataSet`s is evaluated.
    pub fn new(data: Vec<Arc<dyn DataSet<U, E>>>, weights: Vec<f64>, losses: Vec<Loss>) -> Self {
        Self {
            data,
            weights,
            losses,
        }
    }

    /// Add a `DataSet` and its weight.
    pub fn add_data(&mut self, data: &Arc<dyn DataSet<U, E>>, weight: f64, loss: Loss) {
        self.data.push(data.clone());
        self.weights.push(weight);
        self.losses.push(loss);
    }

    /// Returns the cost of each `DataSet`.
    pub fn cost(&self, eos: &Arc<E>) -> Result<Array1<f64>, EstimatorError> {
        let w = arr1(&self.weights) / self.weights.iter().sum::<f64>();
        let costs = self
            .data
            .iter()
            .enumerate()
            .map(|(i, d)| Ok(d.cost(eos, self.losses[i])? * w[i]))
            .collect::<Result<Vec<_>, EstimatorError>>()?;
        let aview: Vec<ArrayView1<f64>> = costs.iter().map(|pi| pi.view()).collect();
        Ok(concatenate(Axis(0), &aview)?)
    }

    /// Returns the cost of each `DataSet` evaluated in parallel.
    #[cfg(feature = "rayon")]
    pub fn par_cost(
        &self,
        eos: &Arc<E>,
        thread_pool: ThreadPool,
    ) -> Result<Array1<f64>, EstimatorError> {
        let ws_inv = 1.0 / self.weights.iter().sum::<f64>();
        let w: Vec<_> = self.weights.iter().map(|wi| wi * ws_inv).collect();

        let costs = thread_pool.install(|| {
            (&self.data, &w, &self.losses)
                .into_par_iter()
                .map(|(d, &w, l)| Ok(d.cost(eos, l.clone())? * w))
                .collect::<Result<Vec<_>, EstimatorError>>()
        })?;
        let aview: Vec<ArrayView1<f64>> = costs.iter().map(|pi| pi.view()).collect();
        Ok(concatenate(Axis(0), &aview)?)
    }

    /// Returns the properties as computed by the equation of state for each `DataSet`.
    pub fn predict(&self, eos: &Arc<E>) -> Result<Vec<QuantityArray1<U>>, EstimatorError> {
        self.data.iter().map(|d| d.predict(eos)).collect()
    }

    /// Returns the relative difference for each `DataSet`.
    pub fn relative_difference(&self, eos: &Arc<E>) -> Result<Vec<Array1<f64>>, EstimatorError> {
        self.data
            .iter()
            .map(|d| d.relative_difference(eos))
            .collect()
    }

    /// Returns the mean absolute relative difference for each `DataSet`.
    pub fn mean_absolute_relative_difference(
        &self,
        eos: &Arc<E>,
    ) -> Result<Array1<f64>, EstimatorError> {
        self.data
            .iter()
            .map(|d| d.mean_absolute_relative_difference(eos))
            .collect()
    }

    /// Returns the stored `DataSet`s.
    pub fn datasets(&self) -> Vec<Arc<dyn DataSet<U, E>>> {
        self.data.to_vec()
    }

    /// Representation as markdown string.
    pub fn _repr_markdownn_(&self) -> String {
        let mut f = String::new();
        write!(f, "| target | input | datapoints |\n|:-|:-|:-|").unwrap();
        for d in self.data.iter() {
            write!(
                f,
                "\n|{}|{}|{}|",
                d.target_str(),
                d.input_str().join(", "),
                d.datapoints()
            )
            .unwrap();
        }
        f
    }
}

impl<U: EosUnit, E: EquationOfState> Display for Estimator<U, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for d in self.data.iter() {
            writeln!(f, "{}", d)?;
        }
        Ok(())
    }
}
