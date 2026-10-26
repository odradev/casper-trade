use crate::factory::errors::FactoryError;
use crate::pair::events::{FactoryInitialized, FeeToUpdated};
use crate::pair::PairFactoryContractRef;
use crate::router::errors::LibraryError::{IdenticalAddresses, ZeroAddress};
use crate::utils::{contract_name, contract_symbol, zero_address};
use odra::prelude::*;
use odra::ContractRef;
use odra_modules::access::{AccessControl, DEFAULT_ADMIN_ROLE};
use odra_modules::cep18_token::Cep18ContractRef;

#[odra::event]
pub struct PairCreated {
    pub token0: Address,
    pub token1: Address,
    pub pair: Address,
    pub contract_name: String,
}

#[odra::module(events = [PairCreated, FeeToUpdated, FactoryInitialized], errors = FactoryError)]
pub struct Factory {
    fee_to: Var<Option<Address>>,
    pairs: Mapping<(Address, Address), Address>,
    pair_factory: Var<Address>,
    access_control: SubModule<AccessControl>,
}

#[odra::module]
impl Factory {
    /// Initializes the factory with the given fee to address.
    ///
    /// If fee to is None, the factory will not charge any fees.
    pub fn init(&mut self, fee_to: Option<Address>, pair_factory: Address) {
        let caller = self.env().caller();
        self.fee_to.set(fee_to);
        self.pair_factory.set(pair_factory);
        self.access_control
            .unchecked_grant_role(&DEFAULT_ADMIN_ROLE, &caller);

        self.env().emit_event(FactoryInitialized {
            fee_to,
            pair_factory,
        });
    }

    /// Grants admin role to an address.
    pub fn register_admin(&mut self, admin: Address) {
        self.access_control.grant_role(&DEFAULT_ADMIN_ROLE, &admin);
    }

    /// Revokes admin role from an address.
    pub fn unregister_admin(&mut self, admin: Address) {
        self.access_control.revoke_role(&DEFAULT_ADMIN_ROLE, &admin);
    }

    /// Checks if an address has admin role.
    pub fn is_admin(&self, address: Address) -> bool {
        self.access_control.has_role(&DEFAULT_ADMIN_ROLE, &address)
    }

    /// Returns the `fee_to`
    pub fn fee_to(&self) -> Option<Address> {
        self.fee_to.get().unwrap_or_revert(self)
    }

    /// Sets `fee_to`
    pub fn set_fee_to(&mut self, fee_to: Option<Address>) {
        self.assert_admin();
        let old_fee_to = self.fee_to.get().unwrap_or_default();
        self.fee_to.set(fee_to);

        self.env().emit_event(FeeToUpdated {
            old: old_fee_to,
            new: fee_to,
        })
    }

    /// Creates a pair for the given tokens. If it exists, it will return existing one.
    pub fn create_pair(&mut self, token_a: Address, token_b: Address) -> Address {
        if token_a == token_b {
            self.revert(IdenticalAddresses)
        }

        let zero_address = zero_address();

        let (token0, token1) = self.sort_tokens(token_a, token_b);
        if token0 == zero_address {
            self.revert(ZeroAddress)
        }

        match self.pairs.get(&(token0, token1)) {
            None => {
                let token0_instance = Cep18ContractRef::new(self.env(), token0);
                let token1_instance = Cep18ContractRef::new(self.env(), token1);
                let contract_symbol =
                    contract_symbol(&token0_instance.symbol(), &token1_instance.symbol());

                let mut contract_factory = PairFactoryContractRef::new(
                    self.env(),
                    self.pair_factory
                        .get_or_revert_with(FactoryError::Misconfigured),
                );
                let pair = contract_factory
                    .new_contract(
                        contract_symbol.clone(),
                        self.env().self_address(),
                        token0,
                        token1,
                    )
                    .0;
                self.pairs.set(&(token0, token1), pair);
                self.env().emit_event(PairCreated {
                    token0,
                    token1,
                    pair,
                    contract_name: contract_symbol,
                });
                pair
            }
            Some(pair) => pair,
        }
    }

    /// Returns the pair address for the given tokens, if it exists.
    pub fn get_pair(&self, token_a: Address, token_b: Address) -> Option<Address> {
        let (token0, token1) = self.sort_tokens(token_a, token_b);
        self.pairs.get(&(token0, token1))
    }

    /// Returns the pair name under which it was deployed by factory.
    pub fn get_pair_name(&self, token_a: Address, token_b: Address) -> String {
        let (token0, token1) = self.sort_tokens(token_a, token_b);

        let token0_instance = Cep18ContractRef::new(self.env(), token0);
        let token1_instance = Cep18ContractRef::new(self.env(), token1);
        contract_name(&token0_instance.symbol(), &token1_instance.symbol())
    }

    /// Sorts two token addresses to ensure consistent ordering.
    fn sort_tokens(&self, token_a: Address, token_b: Address) -> (Address, Address) {
        if token_a < token_b {
            (token_a, token_b)
        } else {
            (token_b, token_a)
        }
    }
}

impl Factory {
    fn assert_admin(&self) {
        if !self.is_admin(self.env().caller()) {
            self.env().revert(FactoryError::PermissionDenied);
        }
    }
}

pub mod errors {
    use odra::prelude::*;

    #[odra::odra_error]
    pub enum FactoryError {
        CreatingAPairWithoutMockingIt = 1,
        Misconfigured = 2,
        PermissionDenied = 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pair::PairFactory;
    use odra::host::{Deployer, NoArgs};

    #[test]
    fn test_admin_add_and_remove() {
        // Arrange: deploy Factory
        let env = odra_test::env();
        let deployer = env.get_account(0);
        let new_admin = env.get_account(1);
        let non_admin = env.get_account(2);

        let pair_factory = PairFactory::deploy(&env, NoArgs);
        let mut factory = Factory::deploy(
            &env,
            FactoryInitArgs {
                fee_to: None,
                pair_factory: pair_factory.address(),
            },
        );

        // Assert: deployer is automatically an admin after init
        assert!(factory.is_admin(deployer));

        // Assert: new_admin is not an admin initially
        assert!(!factory.is_admin(new_admin));

        // Act: deployer adds new_admin as admin
        env.set_caller(deployer);
        factory.register_admin(new_admin);

        // Assert: new_admin is now an admin
        assert!(factory.is_admin(new_admin));
        assert!(factory.is_admin(deployer)); // deployer is still admin

        // Act: deployer removes new_admin from admins
        factory.unregister_admin(new_admin);

        // Assert: new_admin is no longer an admin
        assert!(!factory.is_admin(new_admin));
        assert!(factory.is_admin(deployer)); // deployer is still admin

        // Assert: non_admin was never an admin
        assert!(!factory.is_admin(non_admin));
    }
}
