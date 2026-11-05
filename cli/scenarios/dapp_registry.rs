use casper_trade_contracts::factory::{Factory, FactoryUpgradeArgs, PairCreated};
use casper_trade_contracts::pair::{PairFactory, PairHostRef};
use casper_trade_contracts::router::{Router, RouterUpgradeArgs};
use casper_trade_contracts::sample_tokens::{SampleToken, SampleTokenInitArgs};
use odra::casper_types::bytesrepr::{Bytes, FromBytes};
use odra::casper_types::U256;
use odra::host::{Deployer, HostEnv, HostRef, InstallConfig, NoArgs};
use odra::prelude::{Address, Addressable, BTreeMap, OdraResult};
use odra::schema::casper_contract_schema::NamedCLType;
use odra_cli::{
    scenario::{Args, Error, Scenario, ScenarioMetadata},
    CommandArg, ContractProvider, DeployedContractsContainer,
};
use odra_modules::dapp::storage::{DAPP_KEY, REGISTRY_KEY};
use odra_modules::dapp::{DappMetadata, OwnedDappRegistry, OwnedDappRegistryHostRef};

/// Gas limits in motes: the use measured on nctl (protocol 2.0, with the 1_117_587 motes per stored
/// byte of casper-test) plus about 20%. On a `payment_limited` chain a quarter of the unused limit is
/// burned, while a limit below the use fails and costs the whole limit.
mod gas {
    const fn cspr(amount: u64) -> u64 {
        amount * 1_000_000_000
    }
    /// Installing or upgrading the registry: 273 CSPR used.
    pub const REGISTRY: u64 = cspr(330);
    /// Upgrading the Factory: 337 used.
    pub const FACTORY: u64 = cspr(405);
    /// Upgrading the Router: 408 used.
    pub const ROUTER: u64 = cspr(490);
    /// Upgrading the PairFactory: 676 used; a casper-test block takes at most 812.
    pub const PAIR_FACTORY: u64 = cspr(800);
    /// Upgrading a pair through the PairFactory: 450 used.
    pub const PAIR: u64 = cspr(540);
    /// A call of the registry (metadata, membership): under 1 CSPR used.
    pub const REGISTRY_CALL: u64 = cspr(3);
    /// Installing a sample token: 294 used.
    pub const TOKEN: u64 = cspr(360);
    /// Creating a pair through the Factory: 405 used.
    pub const CREATE_PAIR: u64 = cspr(490);
}

// Factory, Router and the pairs point to the dapp registry (DappContractBase). The Factory is a registrar:
// it passes the registry to every pair it creates and adds the pair to the dapp. The PairFactory is a
// plain deployer and not part of the dapp.

/// Returns the dapp registry from the container, deploying it if it is not there yet.
pub fn load_or_deploy_registry(
    env: &HostEnv,
    container: &DeployedContractsContainer,
) -> Result<OwnedDappRegistryHostRef, Error> {
    if let Ok(registry) = container.contract_ref::<OwnedDappRegistry>(env) {
        return Ok(registry);
    }
    odra_cli::log("Deploying OwnedDappRegistry...");
    env.set_gas(gas::REGISTRY);
    let mut registry = OwnedDappRegistry::deploy_with_cfg(
        env,
        NoArgs,
        InstallConfig::upgradable::<OwnedDappRegistry>(),
    );
    container.add_contract(&registry)?;
    env.set_gas(gas::REGISTRY_CALL);
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

/// Adds the contract to the registry unless it is already there. A contract registered with the other
/// registrar role (by an earlier version of the dapp registry) is removed and added again.
pub fn register(
    env: &HostEnv,
    registry: &mut OwnedDappRegistryHostRef,
    name: &str,
    address: Address,
    is_registrar: bool,
) {
    if registry.is_dapp_contract(&address) {
        if registry.is_dapp_registrar(&address) == is_registrar {
            odra_cli::log(format!("  {} already registered", name));
            return;
        }
        env.set_gas(gas::REGISTRY_CALL);
        registry.remove_dapp_contract(&address);
        odra_cli::log(format!("✓ {} removed to change its registrar role", name));
    }
    env.set_gas(gas::REGISTRY_CALL);
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

/// The registry a contract points to: the `registry` item of its `__dapp` dictionary, read from the
/// chain. Every contract of the dapp keeps it there, the pairs included.
fn dapp_registry_of(env: &HostEnv, contract: &Address) -> Option<Address> {
    to_address(env.get_dictionary_value(contract, DAPP_KEY, REGISTRY_KEY.as_bytes()))
}

/// Every pair of the dapp: the pairs the Factory announced with `PairCreated`, read from its events on
/// the chain, so the contracts file need not list them all (e.g. pairs created through the Router). A
/// pair the contracts file lists keeps its name from there.
///
/// A read of the events that fails is an error, not an empty list: the host reports no events for a
/// Factory it cannot read, and a migration or a check that saw no pairs would wrongly succeed.
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
    let events_count = env.events_count(&factory);
    // A Factory that points to a registry has emitted at least `DappRegistryChanged`.
    if events_count == 0 {
        return Err(pairs_read_error("the Factory reports no events"));
    }
    let mut pairs = vec![];
    for index in 0..events_count {
        let bytes = env
            .get_event_bytes(&factory, index)
            .map_err(|e| pairs_read_error(&format!("event {} of the Factory: {:?}", index, e)))?;
        // The Factory emits other events too; only `PairCreated` parses.
        let Ok((event, _)) = PairCreated::from_bytes(&bytes) else {
            continue;
        };
        let name = names
            .get(&event.pair)
            .cloned()
            .unwrap_or(event.contract_name);
        pairs.push((name, PairHostRef::new(event.pair, env.clone())));
    }
    // Every pair of the contracts file was created by this Factory, so each one has its event.
    let missing: Vec<&str> = names
        .iter()
        .filter(|(address, _)| !pairs.iter().any(|(_, pair)| pair.address() == **address))
        .map(|(_, name)| name.as_str())
        .collect();
    if !missing.is_empty() {
        return Err(pairs_read_error(&format!(
            "no PairCreated event for the pairs {} of the contracts file",
            missing.join(", ")
        )));
    }
    Ok(pairs)
}

fn pairs_read_error(what: &str) -> Error {
    Error::OdraError {
        message: format!("Cannot read the pairs of the Factory: {}", what),
    }
}

pub struct MigrateToDappRegistry;

impl ScenarioMetadata for MigrateToDappRegistry {
    const NAME: &'static str = "MigrateToDappRegistry";
    const DESCRIPTION: &'static str = "Deploys the dapp registry, upgrades Factory, Router, PairFactory and every pair of the Factory so they point to it, and registers them; the Factory registers the pairs it creates";
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
        if force && registry_existed {
            upgrade("OwnedDappRegistry", registry_address, || {
                env.set_gas(gas::REGISTRY);
                OwnedDappRegistry::try_upgrade(env, registry_address, NoArgs)
            })?;
        }

        // A contract is up to date once it points to the registry, read from its `__dapp->registry`
        // on the chain; a forced migration upgrades it anyway.
        let dapp_registry = Some(registry_address);
        let up_to_date =
            |address: &Address| !force && dapp_registry_of(env, address) == dapp_registry;

        let factory = container.contract_ref::<Factory>(env)?.address();
        let factory_was_up_to_date = up_to_date(&factory);
        upgrade_unless(factory_was_up_to_date, "Factory", factory, || {
            env.set_gas(gas::FACTORY);
            Factory::try_upgrade(env, factory, FactoryUpgradeArgs { dapp_registry })
        })?;
        let router = container.contract_ref::<Router>(env)?.address();
        upgrade_unless(up_to_date(&router), "Router", router, || {
            env.set_gas(gas::ROUTER);
            Router::try_upgrade(env, router, RouterUpgradeArgs { dapp_registry })
        })?;
        // The Factory registers the pairs it creates.
        register(env, &mut registry, "Factory", factory, true);
        register(env, &mut registry, "Router", router, false);

        let mut pair_factory = container.contract_ref::<PairFactory>(env)?;
        // An earlier version of the dapp registry registered the PairFactory as a registrar.
        if registry.is_dapp_contract(&pair_factory.address()) {
            env.set_gas(gas::REGISTRY_CALL);
            registry.remove_dapp_contract(&pair_factory.address());
            odra_cli::log("✓ PairFactory removed from the dapp");
        }
        // The PairFactory carries the Pair code the pairs are deployed and upgraded with, and has no
        // dapp state of its own to tell its version. It counts as up to date once an earlier run
        // upgraded the Factory and the pairs; with no pairs yet, it is upgraded on every run.
        let pairs = pairs(env, container)?;
        let pair_factory_up_to_date = factory_was_up_to_date
            && !pairs.is_empty()
            && pairs.iter().all(|(_, pair)| up_to_date(&pair.address()));
        upgrade_unless(
            pair_factory_up_to_date,
            "PairFactory",
            pair_factory.address(),
            || {
                env.set_gas(gas::PAIR_FACTORY);
                PairFactory::try_upgrade(env, pair_factory.address(), NoArgs)
            },
        )?;

        // An upgraded pair takes the registry from the Factory.
        for (name, pair) in pairs {
            let pair_name = format!("Pair {}", name);
            upgrade_unless(
                up_to_date(&pair.address()),
                &pair_name,
                pair.address(),
                || {
                    // The PairFactory keys its children by the pair symbol.
                    env.set_gas(gas::PAIR);
                    pair_factory.try_upgrade_child_contract(pair.symbol())
                },
            )?;
            register(env, &mut registry, &name, pair.address(), false);
        }

        odra_cli::log("✓ Migration to the dapp registry completed");
        Ok(())
    }
}

/// Upgrades the contract unless it is up to date, logging either way.
fn upgrade_unless<T>(
    up_to_date: bool,
    name: &str,
    address: Address,
    upgrade_contract: impl FnOnce() -> OdraResult<T>,
) -> Result<(), Error> {
    if up_to_date {
        odra_cli::log(format!("  {} already upgraded", name));
        return Ok(());
    }
    upgrade(name, address, upgrade_contract)
}

/// Upgrades the contract, logging the start and the end.
fn upgrade<T>(
    name: &str,
    address: Address,
    upgrade_contract: impl FnOnce() -> OdraResult<T>,
) -> Result<(), Error> {
    odra_cli::log(format!("Upgrading {} at {:?}...", name, address));
    upgrade_contract()?;
    odra_cli::log(format!("✓ {} upgraded", name));
    Ok(())
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
            env.set_gas(gas::TOKEN);
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

        env.set_gas(gas::CREATE_PAIR);
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
    const DESCRIPTION: &'static str = "Checks that Factory, Router and every pair of the Factory belong to the dapp and point to it, that the Factory is a registrar and that the PairFactory is not registered";
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
        let mut checks: Vec<(String, bool)> = vec![(
            "Factory registered as a registrar".to_string(),
            registry.is_dapp_registrar(&factory),
        )];
        let pair_factory = container.contract_ref::<PairFactory>(env)?.address();
        checks.push((
            "PairFactory not registered".to_string(),
            !registry.is_dapp_contract(&pair_factory),
        ));
        let router = container.contract_ref::<Router>(env)?.address();
        let mut members = vec![
            ("Factory".to_string(), factory),
            ("Router".to_string(), router),
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
