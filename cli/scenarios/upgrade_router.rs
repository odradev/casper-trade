use casper_trade_contracts::router::Router;
use odra::host::{Deployer, HostEnv, NoArgs};
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
        // A Router from before the dapp registry becomes it; MigrateToDappRegistry then moves the
        // Factory and the pairs to it. A Router that is the registry already keeps its dapp state.
        let _ = Router::try_upgrade(env, router.address(), NoArgs)?;

        odra_cli::log("Router upgraded successfully");

        Ok(())
    }
}
