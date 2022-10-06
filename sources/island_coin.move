/// This module defines a minimal and generic Coin and Balance.
module injoy_labs::island_coin {
    struct IslandCoin {}

    fun init_module(sender: &signer) {
        aptos_framework::managed_coin::initialize<IslandCoin>(
            sender,
            b"Island Coin",
            b"ISL",
            8,
            false,
        );
    }
}