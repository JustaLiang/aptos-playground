module deployer::coin_pool {
    use aptos_framework::coin::{Self, Coin};
    use aptos_framework::signer::address_of;

    struct CoinPool<phantom CoinType> has key {
        coin: Coin<CoinType>,
    }

    public entry fun create_pool<CoinType>(account: &signer) {
        let coin_pool = CoinPool { coin: coin::zero<CoinType>() };
        move_to(account, coin_pool);
    }

    public entry fun destroy_empty_pool<CoinType>(account: &signer) acquires CoinPool {
        let addr = address_of(account);
        let CoinPool { coin } = move_from<CoinPool<CoinType>>(addr);
        coin::destroy_zero(coin);
    }

    public entry fun put_in<CoinType>(
        from_account: &signer,
        pool_owner: address,
        amount: u64,
    ) acquires CoinPool {
        let coin_in = coin::withdraw<CoinType>(from_account, amount);
        let pool = borrow_global_mut<CoinPool<CoinType>>(pool_owner);
        coin::merge(&mut pool.coin, coin_in);
    }

    public entry fun take_out<CoinType>(
        pool_owner: &signer,
        to_account: address,
        amount: u64,
    ) acquires CoinPool {
        let owner_addr = address_of(pool_owner);
        let pool = borrow_global_mut<CoinPool<CoinType>>(owner_addr);
        let claim_coin = coin::extract<CoinType>(&mut pool.coin, amount);
        coin::deposit<CoinType>(to_account, claim_coin);
    }

    /// -----------------
    /// Getter functions
    /// -----------------
    public fun has_pool<CoinType>(owner: address): bool {
        exists<CoinPool<CoinType>>(owner)
    }

    public fun pool_liquidity<CoinType>(owner: address): u64 acquires CoinPool {
        let coin_pool = borrow_global<CoinPool<CoinType>>(owner);
        coin::value<CoinType>(&coin_pool.coin)
    }
}