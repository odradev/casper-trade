use casper_trade_contracts::factory::{Factory, FactoryHostRef, FactoryUpgradeArgs};
use casper_trade_contracts::pair::{PairFactory, PairFactoryHostRef, PairHostRef};
use casper_trade_contracts::router::{Router, RouterHostRef, RouterUpgradeArgs};
use casper_trade_contracts::sample_tokens::{SampleToken, SampleTokenInitArgs};
use odra::casper_types::U256;
use odra::host::{Deployer, HostEnv, HostRef, InstallConfig, NoArgs};
use odra::prelude::{Address, Addressable};
use odra::schema::casper_contract_schema::NamedCLType;
use odra_cli::{
    cspr,
    scenario::{Args, Error, Scenario, ScenarioMetadata},
    CommandArg, ContractProvider, DeployedContractsContainer,
};
use odra_modules::dapp::{DappMetadata, OwnedDappRegistry, OwnedDappRegistryHostRef};

/// Returns the dapp registry from the container, deploying it if it is not there yet.
pub fn load_or_deploy_registry(
    env: &HostEnv,
    container: &DeployedContractsContainer,
) -> Result<OwnedDappRegistryHostRef, Error> {
    if let Ok(registry) = container.contract_ref::<OwnedDappRegistry>(env) {
        return Ok(registry);
    }
    odra_cli::log("Deploying OwnedDappRegistry...");
    env.set_gas(cspr!(400));
    let mut registry = OwnedDappRegistry::deploy_with_cfg(
        env,
        NoArgs,
        InstallConfig::upgradable::<OwnedDappRegistry>(),
    );
    container.add_contract(&registry)?;
    env.set_gas(cspr!(10));
    registry.set_dapp_metadata(DappMetadata {
        name: "CSPR.trade".to_string(),
        description: "Automated Market Maker on Casper Network".to_string(),
        website_url: "https://cspr.trade".to_string(),
        icon_url: "https://cspr.trade/icon.png".to_string(),
    });
    odra_cli::log(format!(
        "✓ OwnedDappRegistry deployed: {:?}",
        registry.address()
    ));
    Ok(registry)
}

/// Adds the contract to the registry unless it is already there.
pub fn register(
    env: &HostEnv,
    registry: &mut OwnedDappRegistryHostRef,
    name: &str,
    address: Address,
    is_factory: bool,
) {
    if registry.is_dapp_contract(&address) {
        odra_cli::log(format!("  {} already registered", name));
        return;
    }
    env.set_gas(cspr!(10));
    registry.add_dapp_contract(&address, is_factory);
    odra_cli::log(format!("✓ {} registered (factory: {})", name, is_factory));
}

/// Points the PairFactory to the registry and adds it to the dapp, unless it is already there.
///
/// The factory only deploys the pairs; the Factory registers them, so the PairFactory joins as a
/// regular contract.
pub fn join_pair_factory(
    env: &HostEnv,
    registry: &mut OwnedDappRegistryHostRef,
    pair_factory: &mut PairFactoryHostRef,
) {
    if registry.is_dapp_contract(&pair_factory.address()) {
        odra_cli::log("  PairFactory already registered");
        return;
    }
    env.set_gas(cspr!(10));
    pair_factory.set_factory_dapp_registry(registry.address());
    register(env, registry, "PairFactory", pair_factory.address(), false);
}

/// Pairs recorded in the container, by package name.
fn pairs(env: &HostEnv, container: &DeployedContractsContainer) -> Vec<(String, PairHostRef)> {
    container
        .all_contracts()
        .into_iter()
        .filter(|contract| contract.name() == "Pair")
        .map(|contract| {
            (
                contract.key_name(),
                PairHostRef::new(contract.address(), env.clone()),
            )
        })
        .collect()
}

pub struct MigrateToDappRegistry;

impl ScenarioMetadata for MigrateToDappRegistry {
    const NAME: &'static str = "MigrateToDappRegistry";
    const DESCRIPTION: &'static str = "Deploys the dapp registry, upgrades Factory, Router, PairFactory and the pairs from the contracts file, and registers them in the dapp";
}

impl Scenario for MigrateToDappRegistry {
    fn args(&self) -> Vec<CommandArg> {
        vec![]
    }

    fn run(
        &self,
        env: &HostEnv,
        container: &DeployedContractsContainer,
        _args: Args,
    ) -> Result<(), Error> {
        let mut registry = load_or_deploy_registry(env, container)?;
        let dapp_registry = registry.address();

        let factory = container.contract_ref::<Factory>(env)?;
        // Livenet runs getters locally with the new code, so `get_dapp_registry` of a contract that
        // has not been upgraded reverts (and panics) instead of failing; ask the registry instead.
        if registry.is_dapp_contract(&factory.address()) {
            odra_cli::log("  Factory already upgraded");
        } else {
            odra_cli::log(format!("Upgrading Factory at {:?}...", factory.address()));
            env.set_gas(cspr!(500));
            Factory::try_upgrade(
                env,
                factory.address(),
                FactoryUpgradeArgs {
                    dapp_registry: Some(dapp_registry),
                },
            )?;
            odra_cli::log("✓ Factory upgraded");
        }

        let router = container.contract_ref::<Router>(env)?;
        if registry.is_dapp_contract(&router.address()) {
            odra_cli::log("  Router already upgraded");
        } else {
            odra_cli::log(format!("Upgrading Router at {:?}...", router.address()));
            env.set_gas(cspr!(500));
            Router::try_upgrade(
                env,
                router.address(),
                RouterUpgradeArgs {
                    dapp_registry: Some(dapp_registry),
                },
            )?;
            odra_cli::log("✓ Router upgraded");
        }

        register(env, &mut registry, "Factory", factory.address(), true);
        register(env, &mut registry, "Router", router.address(), false);

        // The pairs are upgraded by the PairFactory, which has to carry the new Pair code first.
        let mut pair_factory = container.contract_ref::<PairFactory>(env)?;
        let pairs = pairs(env, container);
        let pair_factory_joined = registry.is_dapp_contract(&pair_factory.address());
        if pair_factory_joined
            && pairs
                .iter()
                .all(|(_, pair)| registry.is_dapp_contract(&pair.address()))
        {
            odra_cli::log("  PairFactory and pairs already upgraded");
        } else {
            odra_cli::log(format!(
                "Upgrading PairFactory at {:?}...",
                pair_factory.address()
            ));
            env.set_gas(cspr!(800));
            PairFactory::try_upgrade(env, pair_factory.address(), NoArgs)?;
            odra_cli::log("✓ PairFactory upgraded");
        }
        join_pair_factory(env, &mut registry, &mut pair_factory);

        for (name, pair) in &pairs {
            if !registry.is_dapp_contract(&pair.address()) {
                // The PairFactory keys its children by the pair symbol.
                let symbol = pair.symbol();
                odra_cli::log(format!("Upgrading pair {} ({})...", name, symbol));
                env.set_gas(cspr!(800));
                pair_factory.try_upgrade_child_contract(symbol)?;
                odra_cli::log(format!("✓ Pair {} upgraded", name));
            }
            register(env, &mut registry, name, pair.address(), false);
        }

        odra_cli::log("✓ Migration to the dapp registry completed");
        Ok(())
    }
}

pub struct CreateDappPair;

impl ScenarioMetadata for CreateDappPair {
    const NAME: &'static str = "CreateDappPair";
    const DESCRIPTION: &'static str =
        "Deploys two new sample tokens (DappTokenA, DappTokenB) and creates their pair (DappPair) through the Factory; with index N > 1 the names get the N suffix";
}

impl Scenario for CreateDappPair {
    fn args(&self) -> Vec<CommandArg> {
        vec![
            CommandArg::new(
                "symbol_a",
                "Symbol of the first new token",
                NamedCLType::String,
            )
            .required(),
            CommandArg::new(
                "symbol_b",
                "Symbol of the second new token",
                NamedCLType::String,
            )
            .required(),
            CommandArg::new(
                "index",
                "Index of the pair, 1 by default; the CLI knows the names of indices 1 to 3",
                NamedCLType::U32,
            ),
        ]
    }

    fn run(
        &self,
        env: &HostEnv,
        container: &DeployedContractsContainer,
        args: Args,
    ) -> Result<(), Error> {
        let mut factory = container.contract_ref::<Factory>(env)?;
        let suffix = match args.get_single::<u32>("index").unwrap_or(1) {
            1 => String::new(),
            index => index.to_string(),
        };
        let mut tokens = vec![];
        for (arg, container_name) in [("symbol_a", "DappTokenA"), ("symbol_b", "DappTokenB")] {
            let container_name = format!("{}{}", container_name, suffix);
            let symbol = args.get_single::<String>(arg)?;
            odra_cli::log(format!("Deploying token {}...", symbol));
            env.set_gas(cspr!(500));
            let token = SampleToken::deploy_with_cfg(
                env,
                SampleTokenInitArgs {
                    name: format!("Sample Token {}", symbol),
                    symbol: symbol.clone(),
                    decimals: 18,
                    initial_supply: U256::from(1_000_000_000u64) * U256::exp10(18),
                },
                InstallConfig {
                    package_named_key: format!("sample_token_{}", symbol.to_lowercase()),
                    is_upgradable: true,
                    allow_key_override: true,
                },
            );
            container.add_contract_named(&token, Some(container_name))?;
            odra_cli::log(format!("✓ {} deployed: {:?}", symbol, token.address()));
            tokens.push((symbol, token));
        }

        env.set_gas(cspr!(500));
        let pair = factory.create_pair(tokens[0].1.address(), tokens[1].1.address());
        let pair = PairHostRef::new(pair, env.clone());
        container.add_contract_named(&pair, Some(format!("DappPair{}", suffix)))?;
        odra_cli::log(format!(
            "✓ Pair {}-{} created: {:?}",
            tokens[0].0,
            tokens[1].0,
            pair.address()
        ));
        Ok(())
    }
}

/// Reads the registry a dapp contract points to.
type RegistryGetter = Box<dyn Fn() -> Address>;

pub struct CheckDappRegistry;

impl ScenarioMetadata for CheckDappRegistry {
    const NAME: &'static str = "CheckDappRegistry";
    const DESCRIPTION: &'static str =
        "Checks that Factory, Router, PairFactory and every pair from the contracts file belong to the dapp";
}

impl Scenario for CheckDappRegistry {
    fn args(&self) -> Vec<CommandArg> {
        vec![]
    }

    fn run(
        &self,
        env: &HostEnv,
        container: &DeployedContractsContainer,
        _args: Args,
    ) -> Result<(), Error> {
        let registry = container.contract_ref::<OwnedDappRegistry>(env)?;
        let metadata = registry.get_dapp_metadata();
        odra_cli::log(format!("Dapp registry: {:?}", registry.address()));
        odra_cli::log(format!("  Name: {}", metadata.name));
        odra_cli::log(format!("  Owner: {:?}", registry.get_owner()));
        let contracts = registry.get_dapp_contracts();
        odra_cli::log(format!("  Contracts ({}):", contracts.len()));
        for contract in &contracts {
            let name = container
                .all_contracts()
                .into_iter()
                .find(|c| c.address() == *contract)
                .map(|c| c.key_name())
                .unwrap_or_else(|| "<not in the contracts file>".to_string());
            odra_cli::log(format!(
                "    {:?} {}{}",
                contract,
                name,
                if registry.is_dapp_factory(contract) {
                    " (factory)"
                } else {
                    ""
                }
            ));
        }

        let factory: FactoryHostRef = container.contract_ref::<Factory>(env)?;
        let router: RouterHostRef = container.contract_ref::<Router>(env)?;
        let pair_factory = container.contract_ref::<PairFactory>(env)?;
        let mut members: Vec<(String, Address, RegistryGetter)> = vec![
            (
                "Factory".to_string(),
                factory.address(),
                Box::new(move || factory.get_dapp_registry()),
            ),
            (
                "Router".to_string(),
                router.address(),
                Box::new(move || router.get_dapp_registry()),
            ),
            (
                "PairFactory".to_string(),
                pair_factory.address(),
                Box::new(move || pair_factory.get_dapp_registry()),
            ),
        ];
        for (name, pair) in pairs(env, container) {
            members.push((
                name,
                pair.address(),
                Box::new(move || pair.get_dapp_registry()),
            ));
        }

        let mut failures = vec![];
        odra_cli::log("Membership:");
        for (name, address, get_dapp_registry) in members {
            let registered = registry.is_dapp_contract(&address);
            // A contract that has not been upgraded reverts on livenet, see MigrateToDappRegistry.
            let points_to = registered.then(get_dapp_registry);
            let ok = points_to == Some(registry.address());
            odra_cli::log(format!(
                "  {} {}: registered={}, get_dapp_registry={:?}",
                if ok { "✓" } else { "✗" },
                name,
                registered,
                points_to
            ));
            if !ok {
                failures.push(name);
            }
        }

        if failures.is_empty() {
            odra_cli::log("✓ All contracts belong to the dapp");
            Ok(())
        } else {
            Err(Error::OdraError {
                message: format!("Not part of the dapp: {}", failures.join(", ")),
            })
        }
    }
}
