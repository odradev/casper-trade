use odra::host::{HostEnv, HostRef};
use odra::prelude::Address;
use odra::schema::casper_contract_schema::NamedCLType;
use odra_cli::{
    scenario::{Args, Error, Scenario, ScenarioMetadata},
    CommandArg, DeployedContractsContainer,
};
use odra_modules::wrapped_native::WrappedNativeTokenHostRef;

pub struct AddWCSPR;

impl ScenarioMetadata for AddWCSPR {
    const NAME: &'static str = "AddWCSPR";
    const DESCRIPTION: &'static str =
        "Add an existing WCSPR (Wrapped CSPR) contract to the container by package hash";
}

impl Scenario for AddWCSPR {
    fn args(&self) -> Vec<CommandArg> {
        vec![CommandArg::new(
            "package_hash",
            "Package hash of the existing WCSPR contract (e.g., hash-...)",
            NamedCLType::String,
        )
        .required()]
    }

    fn run(
        &self,
        env: &HostEnv,
        container: &DeployedContractsContainer,
        args: Args,
    ) -> Result<(), Error> {
        // Get the package hash from args
        let package_hash_str = args.get_single::<String>("package_hash")?;

        odra_cli::log(format!("Adding existing WCSPR contract to container"));
        odra_cli::log(format!("  Package Hash: {}", package_hash_str));

        // Parse the package hash as an address
        let address: Address = package_hash_str.parse().map_err(|_| Error::OdraError {
            message: format!("Invalid package hash format: {}", package_hash_str),
        })?;

        // Create a HostRef for the WCSPR contract at the given address
        let wcspr = WrappedNativeTokenHostRef::new(address, env.clone());

        // Add the contract to the container
        container.add_contract(&wcspr)?;

        odra_cli::log("\n✓ WCSPR contract added to container successfully!");
        odra_cli::log("  Contract is now available for use in scenarios");

        Ok(())
    }
}
