use casper_trade_contracts::factory::{Factory, FactoryUpgradeArgs, PairCreated};
use casper_trade_contracts::pair::{PairFactory, PairFactoryHostRef, PairHostRef};
use casper_trade_contracts::router::{Router, RouterUpgradeArgs};
use casper_trade_contracts::sample_tokens::{SampleToken, SampleTokenInitArgs};
use odra::casper_types::bytesrepr::{Bytes, FromBytes};
use odra::casper_types::U256;
use odra::factory_dapp::{CREATOR_KEY, DAPP_KEY, REGISTRY_KEY};
use odra::host::{Deployer, HostEnv, HostRef, InstallConfig, NoArgs};
use odra::prelude::{Address, Addressable, BTreeMap};
use odra::schema::casper_contract_schema::NamedCLType;
use odra_cli::{
    cspr,
    scenario::{Args, Error, Scenario, ScenarioMetadata},
    CommandArg, ContractProvider, DeployedContractsContainer,
};
use odra_modules::dapp::{DappMetadata, OwnedDappRegistry, OwnedDappRegistryHostRef};

// Factory and Router point to the dapp registry themselves (DappContractBase). The pairs carry no dapp
// code: the PairFactory, which only the Factory may call, points every pair it creates or upgrades to the
// registry and adds the pairs it creates.

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
    is_registrar: bool,
) {
    if registry.is_dapp_contract(&address) {
        odra_cli::log(format!("  {} already registered", name));
        return;
    }
    env.set_gas(cspr!(10));
    registry.add_dapp_contract(&address, is_registrar);
    odra_cli::log(format!(
        "✓ {} registered (registrar: {})",
        name, is_registrar
    ));
}

fn to_address(bytes: Option<Bytes>) -> Option<Address> {
    Address::from_bytes(&bytes?)
        .ok()
        .map(|(address, _)| address)
}

/// The address stored under a named key of the contract, if any.
fn named_address(env: &HostEnv, contract: &Address, key: &str) -> Option<Address> {
    to_address(env.get_named_value(contract, key))
}

/// The registry a contract points to: the `registry` item of its `__dapp` dictionary, read from the
/// chain. Every contract of the dapp keeps it there, the pairs included.
fn dapp_registry_of(env: &HostEnv, contract: &Address) -> Option<Address> {
    to_address(env.get_dictionary_value(contract, DAPP_KEY, REGISTRY_KEY.as_bytes()))
}

/// Lets only the Factory create pairs, points the PairFactory to the registry (it points every pair it
/// creates or upgrades there too) and registers it as a registrar, so it adds the pairs it creates.
/// Skips the steps already done.
pub fn connect_pair_factory(
    env: &HostEnv,
    registry: &mut OwnedDappRegistryHostRef,
    pair_factory: &mut PairFactoryHostRef,
    factory: Address,
) {
    let address = pair_factory.address();
    if named_address(env, &address, CREATOR_KEY) == Some(factory) {
        odra_cli::log("  PairFactory creator already set");
    } else {
        env.set_gas(cspr!(10));
        pair_factory.set_factory_creator(factory);
        odra_cli::log("✓ PairFactory creates pairs only for the Factory");
    }
    if dapp_registry_of(env, &address) == Some(registry.address()) {
        odra_cli::log("  PairFactory registry already set");
    } else {
        env.set_gas(cspr!(10));
        pair_factory.set_factory_dapp_registry(registry.address());
        odra_cli::log("✓ PairFactory points to the registry");
    }
    // The registry verifies the PairFactory points to it.
    register(env, registry, "PairFactory", address, true);
}

/// Every pair of the dapp: the pairs the Factory announced with `PairCreated`, read from its events on
/// the chain, so the contracts file need not list them all (e.g. pairs created through the Router). A
/// pair the contracts file lists keeps its name from there.
fn pairs(
    env: &HostEnv,
    container: &DeployedContractsContainer,
) -> Result<Vec<(String, PairHostRef)>, Error> {
    let factory = container.contract_ref::<Factory>(env)?.address();
    let names: BTreeMap<Address, String> = container
        .all_contracts()
        .into_iter()
        .filter(|contract| contract.name() == "Pair")
        .map(|contract| (contract.address(), contract.key_name()))
        .collect();
    let mut pairs = vec![];
    for index in 0..env.events_count(&factory) {
        // The Factory emits other events too; only `PairCreated` parses.
        let Ok(bytes) = env.get_event_bytes(&factory, index) else {
            continue;
        };
        let Ok((event, _)) = PairCreated::from_bytes(&bytes) else {
            continue;
        };
        let name = names
            .get(&event.pair)
            .cloned()
            .unwrap_or(event.contract_name);
        pairs.push((name, PairHostRef::new(event.pair, env.clone())));
    }
    Ok(pairs)
}

pub struct MigrateToDappRegistry;

impl ScenarioMetadata for MigrateToDappRegistry {
    const NAME: &'static str = "MigrateToDappRegistry";
    const DESCRIPTION: &'static str = "Deploys the dapp registry, upgrades Factory, Router, PairFactory and every pair of the Factory so they point to it, and registers them; the PairFactory adds new pairs to the dapp";
}

impl Scenario for MigrateToDappRegistry {
    fn args(&self) -> Vec<CommandArg> {
        vec![CommandArg::new(
            "force_upgrade",
            "Upgrades every contract, the registry included, even if it already points to the registry; brings a deployment of an earlier version of the dapp registry up to date",
            NamedCLType::Bool,
        )]
    }

    fn run(
        &self,
        env: &HostEnv,
        container: &DeployedContractsContainer,
        args: Args,
    ) -> Result<(), Error> {
        // A contract's code version cannot be read from here, so a deployment of an earlier version
        // of the dapp registry needs an explicit forced upgrade.
        let force = args.get_single::<bool>("force_upgrade").unwrap_or(false);
        let registry_existed = container.contract_ref::<OwnedDappRegistry>(env).is_ok();
        let mut registry = load_or_deploy_registry(env, container)?;
        let registry_address = registry.address();
        let dapp_registry = Some(registry_address);
        if force && registry_existed {
            odra_cli::log(format!(
                "Upgrading OwnedDappRegistry at {:?}...",
                registry_address
            ));
            env.set_gas(cspr!(500));
            OwnedDappRegistry::try_upgrade(env, registry_address, NoArgs)?;
            odra_cli::log("✓ OwnedDappRegistry upgraded");
        }
        // Whether a contract points to the registry, read from its `__dapp->registry` on the chain.
        let points = |address: &Address| dapp_registry_of(env, address) == dapp_registry;

        // Factory and Router point to the registry after an upgrade.
        let factory = container.contract_ref::<Factory>(env)?.address();
        if !force && points(&factory) {
            odra_cli::log("  Factory already upgraded");
        } else {
            odra_cli::log(format!("Upgrading Factory at {:?}...", factory));
            env.set_gas(cspr!(500));
            Factory::try_upgrade(env, factory, FactoryUpgradeArgs { dapp_registry })?;
            odra_cli::log("✓ Factory upgraded");
        }
        let router = container.contract_ref::<Router>(env)?.address();
        if !force && points(&router) {
            odra_cli::log("  Router already upgraded");
        } else {
            odra_cli::log(format!("Upgrading Router at {:?}...", router));
            env.set_gas(cspr!(500));
            Router::try_upgrade(env, router, RouterUpgradeArgs { dapp_registry })?;
            odra_cli::log("✓ Router upgraded");
        }
        register(env, &mut registry, "Factory", factory, false);
        register(env, &mut registry, "Router", router, false);

        // The PairFactory needs the factory entry points of the creator and the registry. An
        // upgraded PairFactory has its creator set right after the upgrade.
        let mut pair_factory = container.contract_ref::<PairFactory>(env)?;
        if !force && named_address(env, &pair_factory.address(), CREATOR_KEY).is_some() {
            odra_cli::log("  PairFactory already upgraded");
        } else {
            odra_cli::log(format!(
                "Upgrading PairFactory at {:?}...",
                pair_factory.address()
            ));
            env.set_gas(cspr!(800));
            PairFactory::try_upgrade(env, pair_factory.address(), NoArgs)?;
            odra_cli::log("✓ PairFactory upgraded");
        }
        connect_pair_factory(env, &mut registry, &mut pair_factory, factory);

        // The existing pairs point to the registry after an upgrade through the PairFactory.
        for (name, pair) in pairs(env, container)? {
            if !force && points(&pair.address()) {
                odra_cli::log(format!("  Pair {} already upgraded", name));
            } else {
                // The PairFactory keys its children by the pair symbol.
                let symbol = pair.symbol();
                odra_cli::log(format!("Upgrading pair {} ({})...", name, symbol));
                env.set_gas(cspr!(800));
                pair_factory.try_upgrade_child_contract(symbol)?;
                odra_cli::log(format!("✓ Pair {} upgraded", name));
            }
            register(env, &mut registry, &name, pair.address(), false);
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

pub struct CheckDappRegistry;

impl ScenarioMetadata for CheckDappRegistry {
    const NAME: &'static str = "CheckDappRegistry";
    const DESCRIPTION: &'static str = "Checks that Factory, Router, PairFactory and every pair of the Factory belong to the dapp, and that the PairFactory adds new pairs to it";
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
                if registry.is_dapp_registrar(contract) {
                    " (registrar)"
                } else {
                    ""
                }
            ));
        }

        let factory = container.contract_ref::<Factory>(env)?.address();
        let pair_factory = container.contract_ref::<PairFactory>(env)?.address();
        let mut checks: Vec<(String, bool)> = vec![
            (
                "PairFactory registered as a registrar".to_string(),
                registry.is_dapp_registrar(&pair_factory),
            ),
            (
                "PairFactory creates pairs only for the Factory".to_string(),
                named_address(env, &pair_factory, CREATOR_KEY) == Some(factory),
            ),
            (
                "PairFactory adds new pairs to the registry".to_string(),
                dapp_registry_of(env, &pair_factory) == Some(registry.address()),
            ),
        ];
        let router = container.contract_ref::<Router>(env)?.address();
        let mut members = vec![
            ("Factory".to_string(), factory),
            ("Router".to_string(), router),
            ("PairFactory".to_string(), pair_factory),
        ];
        members.extend(
            pairs(env, container)?
                .into_iter()
                .map(|(name, pair)| (name, pair.address())),
        );
        for (name, address) in members {
            let points_back = registry.is_dapp_contract(&address)
                && dapp_registry_of(env, &address) == Some(registry.address());
            checks.push((
                format!("{} registered and points to the registry", name),
                points_back,
            ));
        }

        let mut failures = vec![];
        odra_cli::log("Checks:");
        for (name, ok) in checks {
            odra_cli::log(format!("  {} {}", if ok { "✓" } else { "✗" }, name));
            if !ok {
                failures.push(name);
            }
        }

        if failures.is_empty() {
            odra_cli::log("✓ All contracts belong to the dapp");
            Ok(())
        } else {
            Err(Error::OdraError {
                message: format!("Failed: {}", failures.join(", ")),
            })
        }
    }
}
