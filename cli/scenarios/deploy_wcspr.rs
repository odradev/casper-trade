use odra::host::{Deployer, HostEnv, InstallConfig, NoArgs};
use odra::prelude::Addressable;
use odra_cli::{
    scenario::{Args, Error, Scenario, ScenarioMetadata},
    CommandArg, ContractProvider, DeployedContractsContainer,
};
use odra_modules::wrapped_native::WrappedNativeToken;

pub struct DeployWcspr;

impl ScenarioMetadata for DeployWcspr {
    const NAME: &'static str = "DeployWcspr";
    const DESCRIPTION: &'static str = "Deploys a new WCSPR (Wrapped CSPR) contract";
}

impl Scenario for DeployWcspr {
    fn args(&self) -> Vec<CommandArg> {
        vec![]
    }

    fn run(
        &self,
        env: &HostEnv,
        container: &DeployedContractsContainer,
        _args: Args,
    ) -> Result<(), Error> {
        let wcspr = container.address_by_name("WrappedNativeToken");
        if wcspr.is_some() {
            odra_cli::log("WCSPR already exists.");
            return Ok(());
        }

        odra_cli::log("Deploying Wrapped Native Token (WCSPR)...");

        env.set_gas(400_000_000_000);

        let wcspr = WrappedNativeToken::deploy_with_cfg(
            env,
            NoArgs,
            InstallConfig::upgradable::<WrappedNativeToken>(),
        );

        container.add_contract(&wcspr)?;

        odra_cli::log(format!(
            "✓ WCSPR deployed successfully at {:?}",
            wcspr.address()
        ));
        Ok(())
    }
}
