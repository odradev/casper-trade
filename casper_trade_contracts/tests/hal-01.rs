mod common;

use casper_trade_contracts::factory::errors::FactoryError;
use casper_trade_contracts::factory::{Factory, FactoryInitArgs};
use casper_trade_contracts::pair::PairFactory;
use odra::host::{Deployer, HostEnv, NoArgs};
use odra::prelude::*;

// (HAL-01) MISSING ACCESS CONTROL ON FEE RECEIVER
// ALLOWS PROTOCOL FEE THEFT
#[test]
fn test_set_fee_to_unrestricted_access_poc() {
    // Arrange: fresh env and deploy Factory with no fee_to configured
    let env: HostEnv = odra_test::env();
    let _deployer = env.get_account(0);
    let attacker = env.get_account(1);
    let treasury = env.get_account(2);

    let pair_factory = PairFactory::deploy(&env, NoArgs);
    let mut factory = Factory::deploy(
        &env,
        FactoryInitArgs {
            fee_to: None,
            pair_factory: pair_factory.address(),
        },
    );
    assert_eq!(factory.fee_to(), None);

    // Act: attacker (non-privileged) changes fee_to
    // This should fail because attacker is not an admin/owner
    env.set_caller(attacker);
    assert!(!factory.is_admin(attacker));
    let result = factory.try_set_fee_to(Some(treasury));
    assert_eq!(result, Err(FactoryError::PermissionDenied.into()));

    // Below now will fail because attacker is not an admin/owner
    // Assert: fee_to updated despite caller not being an admin/owner (demonstrates lack of access control)
    // assert_eq!(factory.fee_to(), Some(treasury));

    // New assert: fee_to is still None because attacker is not an admin/owner
    assert_eq!(factory.fee_to(), None);
}

