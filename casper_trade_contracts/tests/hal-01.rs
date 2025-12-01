mod common;

use casper_trade_contracts::factory::errors::FactoryError;
use common::setup;

// (HAL-01) MISSING ACCESS CONTROL ON FEE RECEIVER
// ALLOWS PROTOCOL FEE THEFT
#[test]
fn test_set_fee_to_unrestricted_access_poc() {
    // Arrange: fresh env and deploy Factory with no fee_to configured
    let mut context = setup();
    let attacker = context.env.get_account(1);
    let treasury = context.env.get_account(2);

    assert_eq!(context.factory.fee_to(), None);

    // Act: attacker (non-privileged) changes fee_to
    // This should fail because attacker is not an admin/owner
    context.env.set_caller(attacker);
    assert!(!context.factory.is_admin(attacker));
    let result = context.factory.try_set_fee_to(Some(treasury));
    assert_eq!(result, Err(FactoryError::PermissionDenied.into()));

    // Below now will fail because attacker is not an admin/owner
    // Assert: fee_to updated despite caller not being an admin/owner (demonstrates lack of access control)
    // assert_eq!(context.factory.fee_to(), Some(treasury));

    // New assert: fee_to is still None because attacker is not an admin/owner
    assert_eq!(context.factory.fee_to(), None);
}
