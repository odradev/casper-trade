use casper_trade_contracts::router::{Router, RouterUpgradeArgs};
use odra::host::{Deployer, HostEnv};
use odra::prelude::Addressable;
use odra_cli::{
    cspr,
    scenario::{Args, Error, Scenario, ScenarioMetadata},
    CommandArg, ContractProvider, DeployedContractsContainer,
};

pub struct UpgradeRouter;

impl ScenarioMetadata for UpgradeRouter {
    const NAME: &'static str = "UpgradeRouter";
    const DESCRIPTION: &'static str = "Upgrade Router to the newest version";
}

impl Scenario for UpgradeRouter {
    fn args(&self) -> Vec<CommandArg> {
        vec![]
    }

    fn run(
        &self,
        env: &HostEnv,
        container: &DeployedContractsContainer,
        _args: Args,
    ) -> Result<(), Error> {
        let router = container.contract_ref::<Router>(env)?;
        odra_cli::log(format!(
            "Upgrading Router at address: {:?}",
            router.address()
        ));

        env.set_gas(cspr!(800));
        // Keeps the dapp registry the router points to; MigrateToDappRegistry sets it.
        let _ = Router::try_upgrade(
            env,
            router.address(),
            RouterUpgradeArgs {
                dapp_registry: None,
            },
        )?;

        odra_cli::log("Router upgraded successfully");

        Ok(())
    }
}
