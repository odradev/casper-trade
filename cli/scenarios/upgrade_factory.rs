use casper_trade_contracts::pair::PairFactory;
use odra::host::{Deployer, HostEnv, NoArgs};
use odra::prelude::Addressable;
use odra_cli::{
    cspr,
    scenario::{Args, Error, Scenario, ScenarioMetadata},
    CommandArg, ContractProvider, DeployedContractsContainer,
};

pub struct UpgradeFactory;

impl ScenarioMetadata for UpgradeFactory {
    const NAME: &'static str = "UpgradeFactory";
    const DESCRIPTION: &'static str = "Upgrade PairFactory to the newest version";
}

impl Scenario for UpgradeFactory {
    fn args(&self) -> Vec<CommandArg> {
        vec![]
    }

    fn run(
        &self,
        env: &HostEnv,
        container: &DeployedContractsContainer,
        _args: Args,
    ) -> Result<(), Error> {
        let pair_factory = container.contract_ref::<PairFactory>(env)?;
        odra_cli::log(format!(
            "Upgrading pair_factory at address: {:?}",
            pair_factory.address()
        ));
        env.set_gas(cspr!(800));
        let _ = PairFactory::try_upgrade(env, pair_factory.address(), NoArgs)?;

        Ok(())
    }
}
