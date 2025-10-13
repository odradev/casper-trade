use odra::prelude::*;

#[odra::module]
pub struct Factory {
    fee_to: Var<Option<Address>>,
}

#[odra::module]
impl Factory {
    /// Initializes the factory with the given fee to address.
    ///
    /// If fee to is None, the factory will not charge any fees.
    pub fn init(&mut self, fee_to: Option<Address>) {
        self.fee_to.set(fee_to);
    }

    pub fn fee_to(&self) -> Option<Address> {
        self.fee_to.get().unwrap_or_revert(self)
    }

    pub fn set_fee_to(&mut self, fee_to: Option<Address>) {
        self.fee_to.set(fee_to);
    }
}
