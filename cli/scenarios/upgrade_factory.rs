use super::utils::{create_token_ref, parse_token_input};
use casper_trade_contracts::pair::PairFactory;
use casper_trade_contracts::utils::contract_symbol;
use odra::host::{Deployer, HostEnv, NoArgs};
use odra::prelude::Addressable;
use odra::schema::casper_contract_schema::NamedCLType;
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
        args: Args,
    ) -> Result<(), Error> {
        let mut pair_factory = container.contract_ref::<PairFactory>(env)?;
        odra_cli::log(format!(
            "Upgrading pair_factory at address: {:?}",
            pair_factory.address()
        ));
        env.set_gas(cspr!(1000));
        let _ = PairFactory::try_upgrade(env, pair_factory.address(), NoArgs)?;

        Ok(())
    }
}
