use odra::prelude::*;

/// Describes one hop of a swap route executed by the Router.
///
/// `token_in_sender` and `token_out_recipient` describe the token flow
/// orchestrated by the Router. They are not ownership claims over the Pair's
/// complete balance delta.
#[odra::event]
pub struct RouterSwapHop {
    pub hop_index: u32,
    pub is_last: bool,
    pub router_caller: Address,
    pub route_recipient: Address,
    pub pair: Address,
    pub input_token: Address,
    pub token_in_sender: Address,
    pub output_token: Address,
    pub token_out_recipient: Address,
}
