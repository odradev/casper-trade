use odra::{casper_types::U256, prelude::*};

/// Describes a complete swap route executed by the Router.
///
/// `token_in_senders` and `token_out_recipients` describe the token flow
/// orchestrated by the Router for each pair in the route. They are not
/// ownership claims over the Pairs' complete balance deltas.
#[odra::event]
pub struct RouterSwapRoute {
    pub router_caller: Address,
    pub route_recipient: Address,
    pub path: Vec<Address>,
    pub amounts: Vec<U256>,
    pub pairs: Vec<Address>,
    pub token_in_senders: Vec<Address>,
    pub token_out_recipients: Vec<Address>,
}
