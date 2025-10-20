use odra::prelude::*;

#[odra::module]
pub struct Factory {
    fee_to: Var<Option<Address>>,
    mock_pair: Var<Option<Address>>,
}

#[odra::module]
impl Factory {
    /// Initializes the factory with the given fee to address.
    ///
    /// If fee to is None, the factory will not charge any fees.
    pub fn init(&mut self, fee_to: Option<Address>) {
        self.fee_to.set(fee_to);
        self.mock_pair.set(None);
    }

    pub fn fee_to(&self) -> Option<Address> {
        self.fee_to.get().unwrap_or_revert(self)
    }

    pub fn set_fee_to(&mut self, fee_to: Option<Address>) {
        self.fee_to.set(fee_to);
    }

    pub fn will_return_pair(&mut self, pair: Option<Address>) {
        self.mock_pair.set(pair);
    }

    pub fn create_pair(&self, _token_a: Address, _token_b: Address) -> Address {
        self.mock_pair
            .get()
            .unwrap_or_revert(self)
            .unwrap_or_revert_with(self, errors::FactoryError::CreatingAPairWithoutMockingIt)
    }

    pub fn get_pair(&self, _token_a: Address, _token_b: Address) -> Option<Address> {
        self.mock_pair.get().unwrap_or_revert(self)
    }
}

pub mod errors {
    use odra::prelude::*;

    #[odra::odra_error]
    pub enum FactoryError {
        CreatingAPairWithoutMockingIt = 1,
    }
}
