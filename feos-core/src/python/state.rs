#[macro_export]
macro_rules! impl_state {
    ($eos:ty, $py_eos:ty) => {
        /// A thermodynamic state at given conditions.
        ///
        /// Parameters
        /// ----------
        /// eos : Eos
        ///     The equation of state to use.
        /// temperature : SINumber, optional
        ///     Temperature.
        /// volume : SINumber, optional
        ///     Volume.
        /// density : SINumber, optional
        ///     Molar density.
        /// partial_density : SIArray1, optional
        ///     Partial molar densities.
        /// total_moles : SINumber, optional
        ///     Total amount of substance (of a mixture).
        /// moles : SIArray1, optional
        ///     Amount of substance for each component.
        /// molefracs : numpy.ndarray[float]
        ///     Molar fraction of each component.
        /// pressure : SINumber, optional
        ///     Pressure.
        /// molar_enthalpy : SINumber, optional
        ///     Molar enthalpy.
        /// molar_entropy : SINumber, optional
        ///     Molar entropy.
        /// molar_internal_energy: SINumber, optional
        ///     Molar internal energy
        /// density_initialization : {'vapor', 'liquid', SINumber, None}, optional
        ///     Method used to initialize density for density iteration.
        ///     'vapor' and 'liquid' are inferred from the maximum density of the equation of state.
        ///     If no density or keyword is provided, the vapor and liquid phase is tested and, if
        ///     different, the result with the lower free energy is returned.
        /// initial_temperature : SINumber, optional
        ///     Initial temperature for temperature iteration. Can improve convergence
        ///     when the state is specified with pressure and molar entropy or enthalpy.
        ///
        /// Returns
        /// -------
        /// State : state at given conditions
        ///
        /// Raises
        /// ------
        /// Error
        ///     When the state cannot be created using the combination of input.
        #[pyclass(name = "State")]
        #[derive(Clone)]
        pub struct PyState(pub State<$eos>);

        #[pymethods]
        impl PyState {
            #[new]
            #[pyo3(text_signature = "(eos, temperature=None, volume=None, density=None, partial_density=None, total_moles=None, moles=None, molefracs=None, pressure=None, molar_enthalpy=None, molar_entropy=None, molar_internal_energy=None, density_initialization=None, initial_temperature=None)")]
            #[pyo3(signature = (eos, temperature=None, volume=None, density=None, partial_density=None, total_moles=None, moles=None, molefracs=None, pressure=None, molar_enthalpy=None, molar_entropy=None, molar_internal_energy=None, density_initialization=None, initial_temperature=None))]
            #[expect(clippy::too_many_arguments)]
            pub fn new<'py>(
                eos: $py_eos,
                temperature: Option<Temperature>,
                volume: Option<Volume>,
                density: Option<Density>,
                partial_density: Option<Density<Array1<f64>>>,
                total_moles: Option<Moles>,
                moles: Option<Moles<Array1<f64>>>,
                molefracs: Option<&Bound<'py, PyArray1<f64>>>,
                pressure: Option<Pressure>,
                molar_enthalpy: Option<MolarEnergy>,
                molar_entropy: Option<MolarEntropy>,
                molar_internal_energy: Option<MolarEnergy>,
                density_initialization: Option<&Bound<'py, PyAny>>,
                initial_temperature: Option<Temperature>,
            ) -> PyResult<Self> {
                let x = molefracs.and_then(|m| Some(m.to_owned_array()));
                let density_init = if let Some(di) = density_initialization {
                    if let Ok(d) = di.extract::<String>().as_deref() {
                        match d {
                            "vapor" => Ok(DensityInitialization::Vapor),
                            "liquid" => Ok(DensityInitialization::Liquid),
                            _ => Err(PyErr::new::<PyValueError, _>(format!(
                                "`density_initialization` must be 'vapor' or 'liquid'."
                            ))),
                        }
                    } else if let Ok(d) = di.extract::<Density>() {
                        Ok(DensityInitialization::InitialDensity(d.try_into()?))
                    } else {
                        Err(PyErr::new::<PyValueError, _>(format!(
                            "`density_initialization` must be 'vapor' or 'liquid' or a molar density as `SINumber` has to be provided."
                        )))
                    }
                } else {
                    Ok(DensityInitialization::None)
                };
                let s = State::new_full(
                    &eos.0,
                    temperature.map(|t| t.try_into()).transpose()?,
                    volume.map(|v| v.try_into()).transpose()?,
                    density.map(|s| s.try_into()).transpose()?,
                    partial_density.map(|s| s.try_into()).transpose()?.as_ref(),
                    total_moles.map(|s| s.try_into()).transpose()?,
                    moles.map(|m| m.try_into()).transpose()?.as_ref(),
                    x.as_ref(),
                    pressure.map(|s| s.try_into()).transpose()?,
                    molar_enthalpy.map(|s| s.try_into()).transpose()?,
                    molar_entropy.map(|s| s.try_into()).transpose()?,
                    molar_internal_energy.map(|s| s.try_into()).transpose()?,
                    density_init?,
                    initial_temperature.map(|s| s.try_into()).transpose()?,
                )?;
                Ok(Self(s))
            }

            /// Return a list of thermodynamic state at critical conditions
            /// for each pure substance in the system.
            ///
            /// Parameters
            /// ----------
            /// eos: EquationOfState
            ///     The equation of state to use.
            /// initial_temperature: SINumber, optional
            ///     The initial temperature.
            /// max_iter : int, optional
            ///     The maximum number of iterations.
            /// tol: float, optional
            ///     The solution tolerance.
            /// verbosity : Verbosity, optional
            ///     The verbosity.
            ///
            /// Returns
            /// -------
            /// State : tate at critical conditions
            #[staticmethod]
            #[pyo3(text_signature = "(eos, initial_temperature=None, max_iter=None, tol=None, verbosity=None)")]
            #[pyo3(signature = (eos, initial_temperature=None, max_iter=None, tol=None, verbosity=None))]
            fn critical_point_pure(
                eos: $py_eos,
                initial_temperature: Option<Temperature>,
                max_iter: Option<usize>,
                tol: Option<f64>,
                verbosity: Option<Verbosity>,
            ) -> PyResult<Vec<Self>> {
                let t = initial_temperature.map(|t0| t0.try_into()).transpose()?;
                let cp = State::critical_point_pure(&eos.0, t, (max_iter, tol, verbosity).into())?;
                Ok(cp.into_iter().map(Self).collect())
            }

            /// Create a thermodynamic state at critical conditions.
            ///
            /// Parameters
            /// ----------
            /// eos: EquationOfState
            ///     The equation of state to use.
            /// moles: SIArray1, optional
            ///     Amount of substance of each component.
            ///     Only optional for a pure component.
            /// initial_temperature: SINumber, optional
            ///     The initial temperature.
            /// max_iter : int, optional
            ///     The maximum number of iterations.
            /// tol: float, optional
            ///     The solution tolerance.
            /// verbosity : Verbosity, optional
            ///     The verbosity.
            ///
            /// Returns
            /// -------
            /// State : State at critical conditions.
            #[staticmethod]
            #[pyo3(text_signature = "(eos, moles=None, initial_temperature=None, max_iter=None, tol=None, verbosity=None)")]
            #[pyo3(signature = (eos, moles=None, initial_temperature=None, max_iter=None, tol=None, verbosity=None))]
            fn critical_point(
                eos: $py_eos,
                moles: Option<Moles<Array1<f64>>>,
                initial_temperature: Option<Temperature>,
                max_iter: Option<usize>,
                tol: Option<f64>,
                verbosity: Option<Verbosity>,
            ) -> PyResult<Self> {
                Ok(PyState(State::critical_point(
                    &eos.0,
                    moles.map(|m| m.try_into()).transpose()?.as_ref(),
                    initial_temperature.map(|t| t.try_into()).transpose()?,
                    (max_iter, tol, verbosity).into(),
                )?))
            }

            /// Create a thermodynamic state at critical conditions for a binary system.
            ///
            /// Parameters
            /// ----------
            /// eos: EquationOfState
            ///     The equation of state to use.
            /// temperature_or_pressure: SINumber
            ///     temperature_or_pressure.
            /// initial_temperature: SINumber, optional
            ///     An initial guess for the temperature.
            /// initial_molefracs: [float], optional
            ///     An initial guess for the composition.
            /// max_iter : int, optional
            ///     The maximum number of iterations.
            /// tol: float, optional
            ///     The solution tolerance.
            /// verbosity : Verbosity, optional
            ///     The verbosity.
            ///
            /// Returns
            /// -------
            /// State : State at critical conditions.
            #[staticmethod]
            #[pyo3(text_signature = "(eos, temperature_or_pressure, initial_temperature=None, initial_molefracs=None, max_iter=None, tol=None, verbosity=None)")]
            #[pyo3(signature = (eos, temperature_or_pressure, initial_temperature=None, initial_molefracs=None, max_iter=None, tol=None, verbosity=None))]
            fn critical_point_binary(
                eos: $py_eos,
                temperature_or_pressure: Bound<'_, PyAny>,
                initial_temperature: Option<Temperature>,
                initial_molefracs: Option<[f64; 2]>,
                max_iter: Option<usize>,
                tol: Option<f64>,
                verbosity: Option<Verbosity>,
            ) -> PyResult<Self> {
                if let Ok(t) = temperature_or_pressure.extract::<Temperature>() {
                    Ok(PyState(State::critical_point_binary(
                        &eos.0,
                        t,
                        initial_temperature.map(|t| t.try_into()).transpose()?,
                        initial_molefracs,
                        (max_iter, tol, verbosity).into(),
                    )?))
                } else if let Ok(p) = temperature_or_pressure.extract::<Pressure>() {
                    Ok(PyState(State::critical_point_binary(
                        &eos.0,
                        p,
                        initial_temperature.map(|t| t.try_into()).transpose()?,
                        initial_molefracs,
                        (max_iter, tol, verbosity).into(),
                    )?))
                } else {
                    Err(PyErr::new::<PyValueError, _>(format!(
                        "Wrong units! Expected K or Pa, got {}.",
                        temperature_or_pressure.call_method0("__repr__")?
                    )))
                }
            }

            /// Calculate spinodal states for a given temperature and composition.
            ///
            /// Parameters
            /// ----------
            /// eos: EquationOfState
            ///     The equation of state to use.
            /// temperature: SINumber
            ///     The temperature.
            /// moles: SIArray1, optional
            ///     Amount of substance of each component.
            ///     Only optional for a pure component.
            /// max_iter : int, optional
            ///     The maximum number of iterations.
            /// tol: float, optional
            ///     The solution tolerance.
            /// verbosity : Verbosity, optional
            ///     The verbosity.
            ///
            /// Returns
            /// -------
            /// State : State at critical conditions.
            #[staticmethod]
            #[pyo3(text_signature = "(eos, temperature, moles=None, max_iter=None, tol=None, verbosity=None)")]
            #[pyo3(signature = (eos, temperature, moles=None, max_iter=None, tol=None, verbosity=None))]
            fn spinodal(
                eos: $py_eos,
                temperature: Temperature,
                moles: Option<Moles<Array1<f64>>>,
                max_iter: Option<usize>,
                tol: Option<f64>,
                verbosity: Option<Verbosity>,
            ) -> PyResult<(Self, Self)> {
                let [state1, state2] = State::spinodal(
                    &eos.0,
                    temperature,
                    moles.map(|m| m.try_into()).transpose()?.as_ref(),
                    (max_iter, tol, verbosity).into(),
                )?;
                Ok((PyState(state1), PyState(state2)))
            }

            /// Performs a stability analysis and returns a list of stable
            /// candidate states.
            ///
            /// Parameters
            /// ----------
            /// max_iter : int, optional
            ///     The maximum number of iterations.
            /// tol: float, optional
            ///     The solution tolerance.
            /// verbosity : Verbosity, optional
            ///     The verbosity.
            ///
            /// Returns
            /// -------
            /// State
            #[pyo3(text_signature = "(max_iter=None, tol=None, verbosity=None)")]
            #[pyo3(signature = (max_iter=None, tol=None, verbosity=None))]
            fn stability_analysis(&self,
                max_iter: Option<usize>,
                tol: Option<f64>,
                verbosity: Option<Verbosity>,
            ) -> PyResult<Vec<Self>> {
                Ok(self
                    .0
                    .stability_analysis((max_iter, tol, verbosity).into())?
                    .into_iter()
                    .map(Self)
                    .collect())
            }

            /// Performs a stability analysis and returns whether the state
            /// is stable
            ///
            /// Parameters
            /// ----------
            /// max_iter : int, optional
            ///     The maximum number of iterations.
            /// tol: float, optional
            ///     The solution tolerance.
            /// verbosity : Verbosity, optional
            ///     The verbosity.
            ///
            /// Returns
            /// -------
            /// bool
            #[pyo3(text_signature = "(max_iter=None, tol=None, verbosity=None)")]
            #[pyo3(signature = (max_iter=None, tol=None, verbosity=None))]
            fn is_stable(&self,
                max_iter: Option<usize>,
                tol: Option<f64>,
                verbosity: Option<Verbosity>,
            ) -> PyResult<bool> {
                Ok(self.0.is_stable((max_iter, tol, verbosity).into())?)
            }

            /// Return pressure.
            ///
            /// Parameters
            /// ----------
            /// contributions: Contributions, optional
            ///     the contributions of the Helmholtz energy.
            ///     Defaults to Contributions.Total.
            ///
            /// Returns
            /// -------
            /// SINumber
            #[pyo3(signature = (contributions=Contributions::Total), text_signature = "($self, contributions)")]
            fn pressure(&self, contributions: Contributions) -> Pressure {
                self.0.pressure(contributions)
            }

            /// Return pressure contributions.
            ///
            /// Returns
            /// -------
            /// List[Tuple[str, SINumber]]
            fn pressure_contributions(&self) -> Vec<(String, Pressure)> {
                self.0.pressure_contributions()
            }

            /// Return compressibility.
            ///
            /// Parameters
            /// ----------
            /// contributions: Contributions, optional
            ///     the contributions of the Helmholtz energy.
            ///     Defaults to Contributions.Total.
            ///
            /// Returns
            /// -------
            /// float
            #[pyo3(signature = (contributions=Contributions::Total), text_signature = "($self, contributions)")]
            fn compressibility(&self, contributions: Contributions) -> f64 {
                self.0.compressibility(contributions)
            }

            /// Return partial derivative of pressure w.r.t. volume.
            ///
            /// Parameters
            /// ----------
            /// contributions: Contributions, optional
            ///     the contributions of the Helmholtz energy.
            ///     Defaults to Contributions.Total.
            ///
            /// Returns
            /// -------
            /// SINumber
            #[pyo3(signature = (contributions=Contributions::Total), text_signature = "($self, contributions)")]
            fn dp_dv(&self, contributions: Contributions) -> Quot<Pressure, Volume> {
                self.0.dp_dv(contributions)
            }

            /// Return partial derivative of pressure w.r.t. density.
            ///
            /// Parameters
            /// ----------
            /// contributions: Contributions, optional
            ///     the contributions of the Helmholtz energy.
            ///     Defaults to Contributions.Total.
            ///
            /// Returns
            /// -------
            /// SINumber
            #[pyo3(signature = (contributions=Contributions::Total), text_signature = "($self, contributions)")]
            fn dp_drho(&self, contributions: Contributions) -> Quot<Pressure, Density> {
                self.0.dp_drho(contributions)
            }

            /// Return partial derivative of pressure w.r.t. temperature.
            ///
            /// Parameters
            /// ----------
            /// contributions: Contributions, optional
            ///     the contributions of the Helmholtz energy.
            ///     Defaults to Contributions.Total.
            ///
            /// Returns
            /// -------
            /// SINumber
            #[pyo3(signature = (contributions=Contributions::Total), text_signature = "($self, contributions)")]
            fn dp_dt(&self, contributions: Contributions) -> Quot<Pressure, Temperature> {
                self.0.dp_dt(contributions)
            }

            /// Return partial derivative of pressure w.r.t. amount of substance.
            ///
            /// Parameters
            /// ----------
            /// contributions: Contributions, optional
            ///     the contributions of the Helmholtz energy.
            ///     Defaults to Contributions.Total.
            ///
            /// Returns
            /// -------
            /// SIArray1
            #[pyo3(signature = (contributions=Contributions::Total), text_signature = "($self, contributions)")]
            fn dp_dni(&self, contributions: Contributions) -> Quot<Pressure, Moles<Array1<f64>>> {
                self.0.dp_dni(contributions)
            }

            /// Return second partial derivative of pressure w.r.t. volume.
            ///
            /// Parameters
            /// ----------
            /// contributions: Contributions, optional
            ///     the contributions of the Helmholtz energy.
            ///     Defaults to Contributions.Total.
            ///
            /// Returns
            /// -------
            /// SINumber
            #[pyo3(signature = (contributions=Contributions::Total), text_signature = "($self, contributions)")]
            fn d2p_dv2(&self, contributions: Contributions) -> Quot<Quot<Pressure, Volume>, Volume> {
                self.0.d2p_dv2(contributions)
            }

            /// Return second partial derivative of pressure w.r.t. density.
            ///
            /// Parameters
            /// ----------
            /// contributions: Contributions, optional
            ///     the contributions of the Helmholtz energy.
            ///     Defaults to Contributions.Total.
            ///
            /// Returns
            /// -------
            /// SINumber
            #[pyo3(signature = (contributions=Contributions::Total), text_signature = "($self, contributions)")]
            fn d2p_drho2(&self, contributions: Contributions) -> Quot<Quot<Pressure, Density>, Density> {
                self.0.d2p_drho2(contributions)
            }

            /// Return partial molar volume of each component.
            ///
            /// Returns
            /// -------
            /// SIArray1
            fn partial_molar_volume(&self) -> MolarVolume<Array1<f64>> {
                self.0.partial_molar_volume()
            }

            /// Return chemical potential of each component.
            ///
            /// Parameters
            /// ----------
            /// contributions: Contributions, optional
            ///     the contributions of the Helmholtz energy.
            ///     Defaults to Contributions.Total.
            ///
            /// Returns
            /// -------
            /// SIArray1
            #[pyo3(signature = (contributions=Contributions::Total), text_signature = "($self, contributions)")]
            fn chemical_potential(&self, contributions: Contributions) -> MolarEnergy<Array1<f64>> {
                self.0.chemical_potential(contributions)
            }

            /// Return chemical potential contributions.
            ///
            /// Parameters
            /// ----------
            /// component: int
            ///     the component for which the contributions
            ///     are calculated
            /// contributions: Contributions, optional
            ///     the contributions of the Helmholtz energy.
            ///     Defaults to Contributions.Total.
            ///
            /// Returns
            /// -------
            /// List[Tuple[str, SINumber]]
            #[pyo3(signature = (component, contributions=Contributions::Total), text_signature = "($self, component, contributions)")]
            fn chemical_potential_contributions(&self, component: usize, contributions: Contributions) -> Vec<(String, MolarEnergy)> {
                self.0.chemical_potential_contributions(component, contributions)
            }

            /// Return derivative of chemical potential w.r.t temperature.
            ///
            /// Parameters
            /// ----------
            /// contributions: Contributions, optional
            ///     the contributions of the Helmholtz energy.
            ///     Defaults to Contributions.Total.
            ///
            /// Returns
            /// -------
            /// SIArray1
            #[pyo3(signature = (contributions=Contributions::Total), text_signature = "($self, contributions)")]
            fn dmu_dt(&self, contributions: Contributions) -> Quot<MolarEnergy<Array1<f64>>, Temperature> {
                self.0.dmu_dt(contributions)
            }

            /// Return derivative of chemical potential w.r.t amount of substance.
            ///
            /// Parameters
            /// ----------
            /// contributions: Contributions, optional
            ///     the contributions of the Helmholtz energy.
            ///     Defaults to Contributions.Total.
            ///
            /// Returns
            /// -------
            /// SIArray2
            #[pyo3(signature = (contributions=Contributions::Total), text_signature = "($self, contributions)")]
            fn dmu_dni(&self, contributions: Contributions) -> Quot<MolarEnergy<Array2<f64>>, Moles> {
                self.0.dmu_dni(contributions)
            }

            /// Return logarithmic fugacity coefficient.
            ///
            /// Returns
            /// -------
            /// numpy.ndarray
            fn ln_phi<'py>(&self, py: Python<'py>) -> Bound<'py, PyArray1<f64>> {
                self.0.ln_phi().into_pyarray_bound(py)
            }

            /// Return logarithmic fugacity coefficient of all components treated as
            /// pure substance at mixture temperature and pressure.
            ///
            /// Returns
            /// -------
            /// numpy.ndarray
            fn ln_phi_pure_liquid<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyArray1<f64>>> {
                Ok(self.0.ln_phi_pure_liquid()?.into_pyarray_bound(py))
            }

            /// Return logarithmic symmetric activity coefficient.
            ///
            /// Returns
            /// -------
            /// numpy.ndarray
            fn ln_symmetric_activity_coefficient<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyArray1<f64>>> {
                Ok(self.0.ln_symmetric_activity_coefficient()?.into_pyarray_bound(py))
            }

            /// Return Henry's law constant of every solute (x_i=0) for a given solvent (x_i>0).
            ///
            /// Parameters
            /// ----------
            /// eos : Eos
            ///     The equation of state to use.
            /// temperature : SINumber
            ///     Temperature.
            /// molefracs : np.ndarray[float]
            ///     Composition of the solvent including x_i=0 for solutes.
            ///
            /// Returns
            /// -------
            /// SIArray1
            #[staticmethod]
            fn henrys_law_constant(eos: $py_eos, temperature: Temperature, molefracs: &Bound<'_, PyArray1<f64>>) -> PyResult<Pressure<Array1<f64>>> {
                Ok(State::henrys_law_constant(&eos.0, temperature, &molefracs.to_owned_array())?)
            }

            /// Return Henry's law constant of a binary system, assuming the first
            /// component is the solute and the second component is the solvent.
            ///
            /// Parameters
            /// ----------
            /// eos : Eos
            ///     The equation of state to use.
            /// temperature : SINumber
            ///     Temperature.
            ///
            /// Returns
            /// -------
            /// SIArray1
            #[staticmethod]
            fn henrys_law_constant_binary(eos: $py_eos, temperature: Temperature) -> PyResult<Pressure> {
                Ok(State::henrys_law_constant_binary(&eos.0, temperature)?)
            }

            /// Return derivative of logarithmic fugacity coefficient w.r.t. temperature.
            ///
            /// Returns
            /// -------
            /// SIArray1
            fn dln_phi_dt(&self) -> Quot<f64, Temperature<Array1<f64>>> {
                self.0.dln_phi_dt()
            }

            /// Return derivative of logarithmic fugacity coefficient w.r.t. pressure.
            ///
            /// Returns
            /// -------
            /// SIArray1
            fn dln_phi_dp(&self) -> Quot<f64, Pressure<Array1<f64>>> {
                self.0.dln_phi_dp()
            }

            /// Return derivative of logarithmic fugacity coefficient w.r.t. amount of substance.
            ///
            /// Returns
            /// -------
            /// SIArray2
            fn dln_phi_dnj(&self) -> Quot<f64, Moles<Array2<f64>>> {
                self.0.dln_phi_dnj()
            }

            /// Return thermodynamic factor.
            ///
            /// Returns
            /// -------
            /// numpy.ndarray
            fn thermodynamic_factor<'py>(&self, py: Python<'py>) -> Bound<'py, PyArray2<f64>> {
                self.0.thermodynamic_factor().into_pyarray_bound(py)
            }

            /// Return molar isochoric heat capacity.
            ///
            /// Parameters
            /// ----------
            /// contributions: Contributions, optional
            ///     the contributions of the Helmholtz energy.
            ///     Defaults to Contributions.Total.
            ///
            /// Returns
            /// -------
            /// SINumber
            #[pyo3(signature = (contributions=Contributions::Total), text_signature = "($self, contributions)")]
            fn molar_isochoric_heat_capacity(&self, contributions: Contributions) -> MolarEntropy {
                self.0.molar_isochoric_heat_capacity(contributions)
            }

            /// Return derivative of isochoric heat capacity w.r.t. temperature.
            ///
            /// Parameters
            /// ----------
            /// contributions: Contributions, optional
            ///     the contributions of the Helmholtz energy.
            ///     Defaults to Contributions.Total.
            ///
            /// Returns
            /// -------
            /// SINumber
            #[pyo3(signature = (contributions=Contributions::Total), text_signature = "($self, contributions)")]
            fn dc_v_dt(&self, contributions: Contributions) -> Quot<MolarEntropy, Temperature> {
                self.0.dc_v_dt(contributions)
            }

            /// Return molar isobaric heat capacity.
            ///
            /// Parameters
            /// ----------
            /// contributions: Contributions, optional
            ///     the contributions of the Helmholtz energy.
            ///     Defaults to Contributions.Total.
            ///
            /// Returns
            /// -------
            /// SINumber
            #[pyo3(signature = (contributions=Contributions::Total), text_signature = "($self, contributions)")]
            fn molar_isobaric_heat_capacity(&self, contributions: Contributions) -> MolarEntropy {
                self.0.molar_isobaric_heat_capacity(contributions)
            }

	        /// Return entropy.
            ///
            /// Parameters
            /// ----------
            /// contributions: Contributions, optional
            ///     the contributions of the Helmholtz energy.
            ///     Defaults to Contributions.Total.
            ///
            /// Returns
            /// -------
            /// SINumber
            #[pyo3(signature = (contributions=Contributions::Total), text_signature = "($self, contributions)")]
            fn entropy(&self, contributions: Contributions) -> Entropy {
                self.0.entropy(contributions)
            }

            /// Return derivative of entropy with respect to temperature.
            ///
            /// Parameters
            /// ----------
            /// contributions: Contributions, optional
            ///     the contributions of the Helmholtz energy.
            ///     Defaults to Contributions.Total.
            ///
            /// Returns
            /// -------
            /// SINumber
            #[pyo3(signature = (contributions=Contributions::Total), text_signature = "($self, contributions)")]
            fn ds_dt(&self, contributions: Contributions) -> Quot<Entropy, Temperature> {
                self.0.ds_dt(contributions)
            }

            /// Return molar entropy.
            ///
            /// Parameters
            /// ----------
            /// contributions: Contributions, optional
            ///     the contributions of the Helmholtz energy.
            ///     Defaults to Contributions.Total.
            ///
            /// Returns
            /// -------
            /// SINumber
            #[pyo3(signature = (contributions=Contributions::Total), text_signature = "($self, contributions)")]
            fn molar_entropy(&self, contributions: Contributions) -> MolarEntropy {
                self.0.molar_entropy(contributions)
            }


            /// Return partial molar entropy of each component.
            ///
            /// Returns
            /// -------
            /// SIArray1
            fn partial_molar_entropy(&self) -> MolarEntropy<Array1<f64>> {
                self.0.partial_molar_entropy()
            }

            /// Return enthalpy.
            ///
            /// Parameters
            /// ----------
            /// contributions: Contributions, optional
            ///     the contributions of the Helmholtz energy.
            ///     Defaults to Contributions.Total.
            ///
            /// Returns
            /// -------
            /// SINumber
            #[pyo3(signature = (contributions=Contributions::Total), text_signature = "($self, contributions)")]
            fn enthalpy(&self, contributions: Contributions) -> Energy {
                self.0.enthalpy(contributions)
            }

            /// Return molar enthalpy.
            ///
            /// Parameters
            /// ----------
            /// contributions: Contributions, optional
            ///     the contributions of the Helmholtz energy.
            ///     Defaults to Contributions.Total.
            ///
            /// Returns
            /// -------
            /// SINumber
            #[pyo3(signature = (contributions=Contributions::Total), text_signature = "($self, contributions)")]
            fn molar_enthalpy(&self, contributions: Contributions) -> MolarEnergy {
                self.0.molar_enthalpy(contributions)
            }


            /// Return partial molar enthalpy of each component.
            ///
            /// Returns
            /// -------
            /// SIArray1
            fn partial_molar_enthalpy(&self) -> MolarEnergy<Array1<f64>> {
                self.0.partial_molar_enthalpy()
            }

            /// Return Helmholtz energy.
            ///
            /// Parameters
            /// ----------
            /// contributions: Contributions, optional
            ///     the contributions of the Helmholtz energy.
            ///     Defaults to Contributions.Total.
            ///
            /// Returns
            /// -------
            /// SINumber
            #[pyo3(signature = (contributions=Contributions::Total), text_signature = "($self, contributions)")]
            fn helmholtz_energy(&self, contributions: Contributions) -> Energy {
                self.0.helmholtz_energy(contributions)
            }

            /// Return molar Helmholtz energy.
            ///
            /// Parameters
            /// ----------
            /// contributions: Contributions, optional
            ///     the contributions of the Helmholtz energy.
            ///     Defaults to Contributions.Total.
            ///
            /// Returns
            /// -------
            /// SINumber
            #[pyo3(signature = (contributions=Contributions::Total), text_signature = "($self, contributions)")]
            fn molar_helmholtz_energy(&self, contributions: Contributions) -> MolarEnergy {
                self.0.molar_helmholtz_energy(contributions)
            }

            /// Return residual Helmholtz energy contributions.
            ///
            /// Returns
            /// -------
            /// List[Tuple[str, SINumber]]
            fn residual_helmholtz_energy_contributions(&self) -> Vec<(String, Energy)> {
                self.0.residual_helmholtz_energy_contributions()
            }

            /// Return Gibbs energy.
            ///
            /// Parameters
            /// ----------
            /// contributions: Contributions, optional
            ///     the contributions of the Helmholtz energy.
            ///     Defaults to Contributions.Total.
            ///
            /// Returns
            /// -------
            /// SINumber
            #[pyo3(signature = (contributions=Contributions::Total), text_signature = "($self, contributions)")]
            fn gibbs_energy(&self, contributions: Contributions) -> Energy {
                self.0.gibbs_energy(contributions)
            }

            /// Return molar Gibbs energy.
            ///
            /// Parameters
            /// ----------
            /// contributions: Contributions, optional
            ///     the contributions of the Helmholtz energy.
            ///     Defaults to Contributions.Total.
            ///
            /// Returns
            /// -------
            /// SINumber
            #[pyo3(signature = (contributions=Contributions::Total), text_signature = "($self, contributions)")]
            fn molar_gibbs_energy(&self, contributions: Contributions) -> MolarEnergy {
                self.0.molar_gibbs_energy(contributions)
            }


            /// Return internal energy.
            ///
            /// Parameters
            /// ----------
            /// contributions: Contributions, optional
            ///     the contributions of the Helmholtz energy.
            ///     Defaults to Contributions.Total.
            ///
            /// Returns
            /// -------
            /// SINumber
            #[pyo3(signature = (contributions=Contributions::Total), text_signature = "($self, contributions)")]
            fn internal_energy(&self, contributions: Contributions) -> Energy {
                self.0.internal_energy(contributions)
            }

            /// Return molar internal energy.
            ///
            /// Parameters
            /// ----------
            /// contributions: Contributions, optional
            ///     the contributions of the Helmholtz energy.
            ///     Defaults to Contributions.Total.
            ///
            /// Returns
            /// -------
            /// SINumber
            #[pyo3(signature = (contributions=Contributions::Total), text_signature = "($self, contributions)")]
            fn molar_internal_energy(&self, contributions: Contributions) -> MolarEnergy {
                self.0.molar_internal_energy(contributions)
            }

            /// Return Joule Thomson coefficient.
            ///
            /// Returns
            /// -------
            /// SINumber
            fn joule_thomson(&self) -> Quot<Temperature, Pressure> {
                self.0.joule_thomson()
            }

            /// Return isentropy compressibility coefficient.
            ///
            /// Returns
            /// -------
            /// SINumber
            fn isentropic_compressibility(&self) -> Quot<f64, Pressure> {
                self.0.isentropic_compressibility()
            }

            /// Return isothermal compressibility coefficient.
            ///
            /// Returns
            /// -------
            /// SINumber
            fn isothermal_compressibility(&self) -> Quot<f64, Pressure> {
                self.0.isothermal_compressibility()
            }

            /// Return isenthalpic compressibility coefficient.
            ///
            /// Returns
            /// -------
            /// SINumber
            fn isenthalpic_compressibility(&self) -> Quot<f64, Pressure> {
                self.0.isenthalpic_compressibility()
            }

            /// Return thermal expansivity coefficient.
            ///
            /// Returns
            /// -------
            /// SINumber
            fn thermal_expansivity(&self) -> Quot<f64, Temperature> {
                self.0.thermal_expansivity()
            }

            /// Return Grueneisen parameter.
            ///
            /// Returns
            /// -------
            /// float
            fn grueneisen_parameter(&self) -> f64 {
                self.0.grueneisen_parameter()
            }

            /// Return structure factor.
            ///
            /// Returns
            /// -------
            /// float
            fn structure_factor(&self) -> f64 {
                self.0.structure_factor()
            }

            /// Return total molar weight.
            ///
            /// Returns
            /// -------
            /// SINumber
            fn total_molar_weight(&self) -> MolarWeight {
                self.0.total_molar_weight()
            }

            /// Return speed of sound.
            ///
            /// Returns
            /// -------
            /// SINumber
            fn speed_of_sound(&self) -> Velocity {
                self.0.speed_of_sound()
            }

            /// Returns mass of each component in the system.
            ///
            /// Returns
            /// -------
            /// SIArray1
            fn mass(&self) -> Mass<Array1<f64>> {
                self.0.mass()
            }

            /// Returns system's total mass.
            ///
            /// Returns
            /// -------
            /// SINumber
            fn total_mass(&self) -> Mass {
                self.0.total_mass()
            }

            /// Returns system's mass density.
            ///
            /// Returns
            /// -------
            /// SINumber
            fn mass_density(&self) -> MassDensity {
                self.0.mass_density()
            }

            /// Returns mass fractions for each component.
            ///
            /// Returns
            /// -------
            /// numpy.ndarray[Float64]
            fn massfracs<'py>(&self, py: Python<'py>) -> Bound<'py, PyArray1<f64>> {
                self.0.massfracs().into_pyarray_bound(py)
            }

            /// Return mass specific Helmholtz energy.
            ///
            /// Parameters
            /// ----------
            /// contributions: Contributions, optional
            ///     the contributions of the Helmholtz energy.
            ///     Defaults to Contributions.Total.
            ///
            /// Returns
            /// -------
            /// SINumber
            #[pyo3(signature = (contributions=Contributions::Total), text_signature = "($self, contributions)")]
            fn specific_helmholtz_energy(&self, contributions: Contributions) -> SpecificEnergy {
                self.0.specific_helmholtz_energy(contributions)
            }

            /// Return mass specific entropy.
            ///
            /// Parameters
            /// ----------
            /// contributions: Contributions, optional
            ///     the contributions of the Helmholtz energy.
            ///     Defaults to Contributions.Total.
            ///
            /// Returns
            /// -------
            /// SINumber
            #[pyo3(signature = (contributions=Contributions::Total), text_signature = "($self, contributions)")]
            fn specific_entropy(&self, contributions: Contributions) -> SpecificEntropy {
                self.0.specific_entropy(contributions)
            }

            /// Return mass specific internal_energy.
            ///
            /// Parameters
            /// ----------
            /// contributions: Contributions, optional
            ///     the contributions of the Helmholtz energy.
            ///     Defaults to Contributions.Total.
            ///
            /// Returns
            /// -------
            /// SINumber
            #[pyo3(signature = (contributions=Contributions::Total), text_signature = "($self, contributions)")]
            fn specific_internal_energy(&self, contributions: Contributions) -> SpecificEnergy {
                self.0.specific_internal_energy(contributions)
            }

            /// Return mass specific gibbs_energy.
            ///
            /// Parameters
            /// ----------
            /// contributions: Contributions, optional
            ///     the contributions of the Helmholtz energy.
            ///     Defaults to Contributions.Total.
            ///
            /// Returns
            /// -------
            /// SINumber
            #[pyo3(signature = (contributions=Contributions::Total), text_signature = "($self, contributions)")]
            fn specific_gibbs_energy(&self, contributions: Contributions) -> SpecificEnergy {
                self.0.specific_gibbs_energy(contributions)
            }

            /// Return mass specific enthalpy.
            ///
            /// Parameters
            /// ----------
            /// contributions: Contributions, optional
            ///     the contributions of the Helmholtz energy.
            ///     Defaults to Contributions.Total.
            ///
            /// Returns
            /// -------
            /// SINumber
            #[pyo3(signature = (contributions=Contributions::Total), text_signature = "($self, contributions)")]
            fn specific_enthalpy(&self, contributions: Contributions) -> SpecificEnergy {
                self.0.specific_enthalpy(contributions)
            }

            /// Return mass specific isochoric heat capacity.
            ///
            /// Parameters
            /// ----------
            /// contributions: Contributions, optional
            ///     the contributions of the Helmholtz energy.
            ///     Defaults to Contributions.Total.
            ///
            /// Returns
            /// -------
            /// SINumber
            #[pyo3(signature = (contributions=Contributions::Total), text_signature = "($self, contributions)")]
            fn specific_isochoric_heat_capacity(&self, contributions: Contributions) -> SpecificEntropy {
                self.0.specific_isochoric_heat_capacity(contributions)
            }

            /// Return mass specific isobaric heat capacity.
            ///
            /// Parameters
            /// ----------
            /// contributions: Contributions, optional
            ///     the contributions of the Helmholtz energy.
            ///     Defaults to Contributions.Total.
            ///
            /// Returns
            /// -------
            /// SINumber
            #[pyo3(signature = (contributions=Contributions::Total), text_signature = "($self, contributions)")]
            fn specific_isobaric_heat_capacity(&self, contributions: Contributions) -> SpecificEntropy {
                self.0.specific_isobaric_heat_capacity(contributions)
            }

            #[getter]
            fn get_total_moles(&self) -> Moles {
                self.0.total_moles
            }

            #[getter]
            fn get_temperature(&self) -> Temperature {
                self.0.temperature
            }

            #[getter]
            fn get_volume(&self) -> Volume {
                self.0.volume
            }

            #[getter]
            fn get_density(&self) -> Density {
                self.0.density
            }

            #[getter]
            fn get_moles(&self) -> Moles<Array1<f64>> {
                self.0.moles.clone()
            }

            #[getter]
            fn get_partial_density(&self) -> Density<Array1<f64>> {
                self.0.partial_density.clone()
            }

            #[getter]
            fn get_molefracs<'py>(&self, py: Python<'py>) -> Bound<'py, PyArray1<f64>> {
                self.0.molefracs.to_pyarray_bound(py)
            }

            fn _repr_markdown_(&self) -> String {
                if self.0.eos.components() == 1 {
                    format!(
                        "|temperature|density|\n|-|-|\n|{:.5}|{:.5}|",
                        self.0.temperature, self.0.density
                    )
                } else {
                    format!(
                        "|temperature|density|molefracs\n|-|-|-|\n|{:.5}|{:.5}|{:.5}|",
                        self.0.temperature, self.0.density, self.0.molefracs
                    )
                }
            }

            fn __repr__(&self) -> PyResult<String> {
                Ok(self.0.to_string())
            }
        }


        /// A list of states that provides convenient getters
        /// for properties of all the individual states.
        ///
        /// Parameters
        /// ----------
        /// states : [State]
        ///     A list of individual states.
        ///
        /// Returns
        /// -------
        /// StateVec
        #[pyclass(name = "StateVec")]
        pub struct PyStateVec(Vec<State<$eos>>);

        impl From<StateVec<'_, $eos>> for PyStateVec {
            fn from(vec: StateVec<$eos>) -> Self {
                Self(vec.into_iter().map(|s| s.clone()).collect())
            }
        }

        impl<'a> From<&'a PyStateVec> for StateVec<'a, $eos> {
            fn from(vec: &'a PyStateVec) -> Self {
                Self(vec.0.iter().collect())
            }
        }

        #[pymethods]
        impl PyStateVec {
            #[new]
            fn new(states: Vec<PyState>) -> Self {
                Self(states.into_iter().map(|s| s.0).collect())
            }

            fn __len__(&self) -> PyResult<usize> {
                Ok(self.0.len())
            }

            fn __getitem__(&self, idx: isize) -> PyResult<PyState> {
                let i = if idx < 0 {
                    self.0.len() as isize + idx
                } else {
                    idx
                };
                if (0..self.0.len()).contains(&(i as usize)) {
                    Ok(PyState(self.0[i as usize].clone()))
                } else {
                    Err(PyIndexError::new_err(format!("StateVec index out of range")))
                }
            }

            /// Return molar entropy.
            ///
            /// Parameters
            /// ----------
            /// contributions: Contributions, optional
            ///     the contributions of the Helmholtz energy.
            ///     Defaults to Contributions.Total.
            ///
            /// Returns
            /// -------
            /// SIArray1
            #[pyo3(signature = (contributions=Contributions::Total), text_signature = "($self, contributions)")]
            fn molar_entropy(&self, contributions: Contributions) -> MolarEntropy<Array1<f64>> {
                StateVec::from(self).molar_entropy(contributions)
            }

            /// Return mass specific entropy.
            ///
            /// Parameters
            /// ----------
            /// contributions: Contributions, optional
            ///     the contributions of the Helmholtz energy.
            ///     Defaults to Contributions.Total.
            ///
            /// Returns
            /// -------
            /// SIArray1
            #[pyo3(signature = (contributions=Contributions::Total), text_signature = "($self, contributions)")]
            fn specific_entropy(&self, contributions: Contributions) -> SpecificEntropy<Array1<f64>> {
                StateVec::from(self).specific_entropy(contributions)
            }

            /// Return molar enthalpy.
            ///
            /// Parameters
            /// ----------
            /// contributions: Contributions, optional
            ///     the contributions of the Helmholtz energy.
            ///     Defaults to Contributions.Total.
            ///
            /// Returns
            /// -------
            /// SIArray1
            #[pyo3(signature = (contributions=Contributions::Total), text_signature = "($self, contributions)")]
            fn molar_enthalpy(&self, contributions: Contributions) -> MolarEnergy<Array1<f64>> {
                StateVec::from(self).molar_enthalpy(contributions)
            }

            /// Return mass specific enthalpy.
            ///
            /// Parameters
            /// ----------
            /// contributions: Contributions, optional
            ///     the contributions of the Helmholtz energy.
            ///     Defaults to Contributions.Total.
            ///
            /// Returns
            /// -------
            /// SIArray1
            #[pyo3(signature = (contributions=Contributions::Total), text_signature = "($self, contributions)")]
            fn specific_enthalpy(&self, contributions: Contributions) -> SpecificEnergy<Array1<f64>> {
                StateVec::from(self).specific_enthalpy(contributions)
            }


            #[getter]
            fn get_temperature(&self) -> Temperature<Array1<f64>> {
                StateVec::from(self).temperature()
            }

            #[getter]
            fn get_pressure(&self) -> Pressure<Array1<f64>> {
                StateVec::from(self).pressure()
            }

            #[getter]
            fn get_compressibility<'py>(&self, py: Python<'py>) -> Bound<'py, PyArray1<f64>> {
                StateVec::from(self).compressibility().into_pyarray_bound(py)
            }

            #[getter]
            fn get_density(&self) -> Density<Array1<f64>> {
                StateVec::from(self).density()
            }

            #[getter]
            fn get_moles<'py>(&self, py: Python<'py>) -> Moles<Array2<f64>> {
                StateVec::from(self).moles()
            }

            #[getter]
            fn get_molefracs<'py>(&self, py: Python<'py>) -> Bound<'py, PyArray2<f64>> {
                StateVec::from(self).molefracs().into_pyarray_bound(py)
            }

            #[getter]
            fn get_mass_density(&self) -> Option<MassDensity<Array1<f64>>> {
                self.0[0].eos.residual.has_molar_weight().then(|| StateVec::from(self).mass_density())
            }

            #[getter]
            fn get_massfracs<'py>(&self, py: Python<'py>) -> Option<Bound<'py, PyArray2<f64>>> {
                self.0[0].eos.residual.has_molar_weight().then(|| StateVec::from(self).massfracs().into_pyarray_bound(py))
            }

            /// Returns selected properties of a StateVec as dictionary.
            ///
            /// Parameters
            /// ----------
            /// contributions : Contributions, optional
            ///     The contributions to consider when calculating properties.
            ///     Defaults to Contributions.Total.
            ///
            /// Returns
            /// -------
            /// Dict[str, List[float]]
            ///     Keys: property names. Values: property for each state.
            ///
            /// Notes
            /// -----
            /// - temperature : K
            /// - pressure : Pa
            /// - densities : mol / m³
            /// - mass densities : kg / m³
            /// - molar enthalpies : kJ / mol
            /// - molar entropies : kJ / mol / K
            /// - specific enthalpies : kJ / kg
            /// - specific entropies : kJ / kg / K
            /// - xi: molefraction of component i
            /// - component index `i` matches to order of components in parameters.
            #[pyo3(signature = (contributions=Contributions::Total), text_signature = "($self, contributions)")]
            pub fn to_dict(&self, contributions: Contributions) -> HashMap<String, Vec<f64>> {
                let states = StateVec::from(self);
                let n = states.0[0].eos.components();
                let mut dict = HashMap::with_capacity(8 + n);
                if n != 1 {
                    let xs = states.molefracs();
                    for i in 0..n {
                        dict.insert(String::from(format!("x{}", i)), xs.column(i).to_vec());
                    }
                }
                dict.insert(String::from("temperature"), states.temperature().convert_to(KELVIN).into_raw_vec_and_offset().0);
                dict.insert(String::from("pressure"), states.pressure().convert_to(PASCAL).into_raw_vec_and_offset().0);
                dict.insert(String::from("density"), states.density().convert_to(MOL / METER.powi::<P3>()).into_raw_vec_and_offset().0);
                dict.insert(String::from("molar enthalpy"), states.molar_enthalpy(contributions).convert_to(KILO * JOULE / MOL).into_raw_vec_and_offset().0);
                dict.insert(String::from("molar entropy"), states.molar_entropy(contributions).convert_to(KILO * JOULE / KELVIN / MOL).into_raw_vec_and_offset().0);
                if states.0[0].eos.residual.has_molar_weight() {
                    dict.insert(String::from("mass density"), states.mass_density().convert_to(KILOGRAM / METER.powi::<P3>()).into_raw_vec_and_offset().0);
                    dict.insert(String::from("specific enthalpy"), states.specific_enthalpy(contributions).convert_to(KILO * JOULE / KILOGRAM).into_raw_vec_and_offset().0);
                    dict.insert(String::from("specific entropy"), states.specific_entropy(contributions).convert_to(KILO * JOULE / KELVIN / KILOGRAM).into_raw_vec_and_offset().0);
                }
                dict
            }
        }
    };
}

#[macro_export]
macro_rules! impl_state_entropy_scaling {
    ($eos:ty, $py_eos:ty) => {
        #[pymethods]
        impl PyState {
            /// Return viscosity via entropy scaling.
            ///
            /// Returns
            /// -------
            /// SINumber
            fn viscosity(&self) -> PyResult<quantity::Viscosity> {
                Ok(self.0.viscosity()?)
            }

            /// Return reference viscosity for entropy scaling.
            ///
            /// Returns
            /// -------
            /// SINumber
            fn viscosity_reference(&self) -> PyResult<quantity::Viscosity> {
                Ok(self.0.viscosity_reference()?)
            }

            /// Return logarithmic reduced viscosity.
            ///
            /// This equals the viscosity correlation function
            /// as used by entropy scaling.
            ///
            /// Returns
            /// -------
            /// float
            fn ln_viscosity_reduced(&self) -> PyResult<f64> {
                Ok(self.0.ln_viscosity_reduced()?)
            }

            /// Return diffusion coefficient via entropy scaling.
            ///
            /// Returns
            /// -------
            /// SINumber
            fn diffusion(&self) -> PyResult<Diffusivity> {
                Ok(self.0.diffusion()?)
            }

            /// Return reference diffusion for entropy scaling.
            ///
            /// Returns
            /// -------
            /// SINumber
            fn diffusion_reference(&self) -> PyResult<Diffusivity> {
                Ok(self.0.diffusion_reference()?)
            }

            /// Return logarithmic reduced diffusion.
            ///
            /// This equals the diffusion correlation function
            /// as used by entropy scaling.
            ///
            /// Returns
            /// -------
            /// float
            fn ln_diffusion_reduced(&self) -> PyResult<f64> {
                Ok(self.0.ln_diffusion_reduced()?)
            }

            /// Return thermal conductivity via entropy scaling.
            ///
            /// Returns
            /// -------
            /// SINumber
            fn thermal_conductivity(&self) -> PyResult<quantity::ThermalConductivity> {
                Ok(self.0.thermal_conductivity()?)
            }

            /// Return reference thermal conductivity for entropy scaling.
            ///
            /// Returns
            /// -------
            /// SINumber
            fn thermal_conductivity_reference(&self) -> PyResult<quantity::ThermalConductivity> {
                Ok(self.0.thermal_conductivity_reference()?)
            }

            /// Return logarithmic reduced thermal conductivity.
            ///
            /// This equals the thermal conductivity correlation function
            /// as used by entropy scaling.
            ///
            /// Returns
            /// -------
            /// float
            fn ln_thermal_conductivity_reduced(&self) -> PyResult<f64> {
                Ok(self.0.ln_thermal_conductivity_reduced()?)
            }
        }
    };
}
