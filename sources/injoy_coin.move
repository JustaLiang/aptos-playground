/// This module defines a minimal and generic Coin and Balance.
module injoy_labs::injoy_coin {
    struct InJoyCoin {}

    fun init_module(sender: &signer) {
        aptos_framework::managed_coin::initialize<InJoyCoin>(
            sender,
            b"InJoy Coin",
            b"IJY",
            8,
            false,
        );
    }
}