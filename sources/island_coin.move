/// This module defines a minimal and generic Coin and Balance.
module justa::island_coin {
    struct IslandCoin {}

    fun init_module(sender: &signer) {
        aptos_framework::managed_coin::initialize<IslandCoin>(
            sender,
            b"Island Coin",
            b"ISL",
            6,
            false,
        );
    }
}