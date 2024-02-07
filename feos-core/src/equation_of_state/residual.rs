use super::{Components, HelmholtzEnergy, HelmholtzEnergyDual};
use crate::si::*;
use crate::StateHD;
use crate::{EosError, EosResult};
use ndarray::prelude::*;
use num_dual::*;
use num_traits::{One, Zero};
use std::marker::PhantomData;
use std::ops::Div;

pub trait Properties {
    type Values<D>;
}

/// A reisdual Helmholtz energy model.
pub trait Residual: Components + Send + Sync {
    type Properties<D>;

    fn properties<D: DualNum<f64>>(&self, temperature: D) -> Self::Properties<D>;

    /// Return the maximum density in Angstrom^-3.
    ///
    /// This value is used as an estimate for a liquid phase for phase
    /// equilibria and other iterations. It is not explicitly meant to
    /// be a mathematical limit for the density (if those exist in the
    /// equation of state anyways).
    fn compute_max_density(&self, moles: &Array1<f64>) -> f64;

    /// Return a slice of the individual contributions (excluding the ideal gas)
    /// of the equation of state.
    fn contributions(&self) -> &[Box<dyn HelmholtzEnergy<Self>>];

    /// Molar weight of all components.
    ///
    /// Enables calculation of (mass) specific properties.
    fn molar_weight(&self) -> MolarWeight<Array1<f64>>;

    /// Evaluate the residual reduced Helmholtz energy $\beta A^\mathrm{res}$.
    fn evaluate_residual<D: DualNum<f64> + Copy>(&self, state: &StateHD<D>) -> D
    where
        dyn HelmholtzEnergy<Self>: HelmholtzEnergyDual<Self::Properties<D>, D>,
    {
        let properties = self.properties(state.temperature);
        self.contributions()
            .iter()
            .map(|c| c.helmholtz_energy(state, &properties))
            .sum()
    }

    /// Evaluate the reduced Helmholtz energy of each individual contribution
    /// and return them together with a string representation of the contribution.
    fn evaluate_residual_contributions<D: DualNum<f64> + Copy>(
        &self,
        state: &StateHD<D>,
    ) -> Vec<(String, D)>
    where
        dyn HelmholtzEnergy<Self>: HelmholtzEnergyDual<Self::Properties<D>, D>,
    {
        let properties = self.properties(state.temperature);
        self.contributions()
            .iter()
            .map(|c| (c.to_string(), c.helmholtz_energy(state, &properties)))
            .collect()
    }

    /// Check if the provided optional mole number is consistent with the
    /// equation of state.
    ///
    /// In general, the number of elements in `moles` needs to match the number
    /// of components of the equation of state. For a pure component, however,
    /// no moles need to be provided. In that case, it is set to the constant
    /// reference value.
    fn validate_moles(&self, moles: Option<&Moles<Array1<f64>>>) -> EosResult<Moles<Array1<f64>>> {
        let l = moles.map_or(1, |m| m.len());
        if self.components() == l {
            match moles {
                Some(m) => Ok(m.to_owned()),
                None => Ok(Moles::from_reduced(Array::ones(1))),
            }
        } else {
            Err(EosError::IncompatibleComponents(self.components(), l))
        }
    }

    /// Calculate the maximum density.
    ///
    /// This value is used as an estimate for a liquid phase for phase
    /// equilibria and other iterations. It is not explicitly meant to
    /// be a mathematical limit for the density (if those exist in the
    /// equation of state anyways).
    fn max_density(&self, moles: Option<&Moles<Array1<f64>>>) -> EosResult<Density> {
        let mr = self.validate_moles(moles)?.to_reduced();
        Ok(Density::from_reduced(self.compute_max_density(&mr)))
    }

    // /// Calculate the second virial coefficient $B(T)$
    // fn second_virial_coefficient(
    //     &self,
    //     temperature: Temperature,
    //     moles: Option<&Moles<Array1<f64>>>,
    // ) -> EosResult<<f64 as Div<Density>>::Output> {
    //     let mr = self.validate_moles(moles)?;
    //     let x = (&mr / mr.sum()).into_value();
    //     let mut rho = HyperDual64::zero();
    //     rho.eps1 = 1.0;
    //     rho.eps2 = 1.0;
    //     let t = HyperDual64::from(temperature.to_reduced());
    //     let s = StateHD::new_virial(t, rho, x);
    //     Ok(Quantity::from_reduced(
    //         self.evaluate_residual(&s).eps1eps2 * 0.5,
    //     ))
    // }

    // /// Calculate the third virial coefficient $C(T)$
    // #[allow(clippy::type_complexity)]
    // fn third_virial_coefficient(
    //     &self,
    //     temperature: Temperature,
    //     moles: Option<&Moles<Array1<f64>>>,
    // ) -> EosResult<<<f64 as Div<Density>>::Output as Div<Density>>::Output> {
    //     let mr = self.validate_moles(moles)?;
    //     let x = (&mr / mr.sum()).into_value();
    //     let rho = Dual3_64::zero().derivative();
    //     let t = Dual3_64::from(temperature.to_reduced());
    //     let s = StateHD::new_virial(t, rho, x);
    //     Ok(Quantity::from_reduced(self.evaluate_residual(&s).v3 / 3.0))
    // }

    // /// Calculate the temperature derivative of the second virial coefficient $B'(T)$
    // #[allow(clippy::type_complexity)]
    // fn second_virial_coefficient_temperature_derivative(
    //     &self,
    //     temperature: Temperature,
    //     moles: Option<&Moles<Array1<f64>>>,
    // ) -> EosResult<<<f64 as Div<Density>>::Output as Div<Temperature>>::Output> {
    //     let mr = self.validate_moles(moles)?;
    //     let x = (&mr / mr.sum()).into_value();
    //     let mut rho = HyperDual::zero();
    //     rho.eps1 = Dual64::one();
    //     rho.eps2 = Dual64::one();
    //     let t = HyperDual::from_re(Dual64::from(temperature.to_reduced()).derivative());
    //     let s = StateHD::new_virial(t, rho, x);
    //     Ok(Quantity::from_reduced(
    //         self.evaluate_residual(&s).eps1eps2.eps * 0.5,
    //     ))
    // }

    // /// Calculate the temperature derivative of the third virial coefficient $C'(T)$
    // #[allow(clippy::type_complexity)]
    // fn third_virial_coefficient_temperature_derivative(
    //     &self,
    //     temperature: Temperature,
    //     moles: Option<&Moles<Array1<f64>>>,
    // ) -> EosResult<
    //     <<<f64 as Div<Density>>::Output as Div<Density>>::Output as Div<Temperature>>::Output,
    // > {
    //     let mr = self.validate_moles(moles)?;
    //     let x = (&mr / mr.sum()).into_value();
    //     let rho = Dual3::zero().derivative();
    //     let t = Dual3::from_re(Dual64::from(temperature.to_reduced()).derivative());
    //     let s = StateHD::new_virial(t, rho, x);
    //     Ok(Quantity::from_reduced(
    //         self.evaluate_residual(&s).v3.eps / 3.0,
    //     ))
    // }
}

/// Reference values and residual entropy correlations for entropy scaling.
pub trait EntropyScaling {
    fn viscosity_reference(
        &self,
        temperature: Temperature,
        volume: Volume,
        moles: &Moles<Array1<f64>>,
    ) -> EosResult<Viscosity>;
    fn viscosity_correlation(&self, s_res: f64, x: &Array1<f64>) -> EosResult<f64>;
    fn diffusion_reference(
        &self,
        temperature: Temperature,
        volume: Volume,
        moles: &Moles<Array1<f64>>,
    ) -> EosResult<Diffusivity>;
    fn diffusion_correlation(&self, s_res: f64, x: &Array1<f64>) -> EosResult<f64>;
    fn thermal_conductivity_reference(
        &self,
        temperature: Temperature,
        volume: Volume,
        moles: &Moles<Array1<f64>>,
    ) -> EosResult<ThermalConductivity>;
    fn thermal_conductivity_correlation(&self, s_res: f64, x: &Array1<f64>) -> EosResult<f64>;
}

/// Dummy implementation for [EquationOfState](super::EquationOfState)s that only contain an ideal gas contribution.
pub struct NoResidual(pub usize);

impl Components for NoResidual {
    fn components(&self) -> usize {
        self.0
    }

    fn subset(&self, component_list: &[usize]) -> Self {
        Self(component_list.len())
    }
}

impl Residual for NoResidual {
    type Properties<D> = PhantomData<D>;

    fn properties<D: DualNum<f64>>(&self, temperature: D) -> PhantomData<D> {
        PhantomData
    }

    fn compute_max_density(&self, _: &Array1<f64>) -> f64 {
        1.0
    }

    fn contributions(&self) -> &[Box<dyn HelmholtzEnergy<Self>>] {
        &[]
    }

    fn molar_weight(&self) -> MolarWeight<Array1<f64>> {
        panic!("No mass specific properties are available for this model!")
    }
}
