module deployer::coin_pool {
    use aptos_framework::coin::{Self, Coin};
    use aptos_framework::signer::address_of;

    use std::error;

    const ECOIN_POOL_NOT_FOUND: u64 = 0;
    const ECOIN_POOL_ALREADY_EXISTS: u64 = 1;

    struct CoinPool<phantom CoinType> has key {
        coin: Coin<CoinType>,
    }

    /// -----------------
    /// Getter functions
    /// -----------------
    
    public fun has_pool<CoinType>(owner: address): bool {
        exists<CoinPool<CoinType>>(owner)
    }

    public fun pool_liquidity<CoinType>(addr: address): u64 acquires CoinPool {
        assert!(
            has_pool<CoinType>(addr),
            error::not_found(ECOIN_POOL_NOT_FOUND),
        );
        let coin_pool = borrow_global<CoinPool<CoinType>>(addr);
        coin::value<CoinType>(&coin_pool.coin)
    }

    /// -----------------
    /// Public functions
    /// -----------------

    public entry fun create_pool<CoinType>(account: &signer) {
        assert!(
            !has_pool<CoinType>(address_of(account)),
            error::already_exists(ECOIN_POOL_ALREADY_EXISTS),
        );
        let coin_pool = CoinPool { coin: coin::zero<CoinType>() };
        move_to(account, coin_pool);
    }

    public fun destroy_empty_pool<CoinType>(account: &signer) acquires CoinPool {
        let addr = address_of(account);
        assert!(
            has_pool<CoinType>(addr),
            error::not_found(ECOIN_POOL_NOT_FOUND),
        );
        let CoinPool { coin } = move_from<CoinPool<CoinType>>(addr);
        coin::destroy_zero(coin);
    }

    public fun extract<CoinType>(
        pool: &mut CoinPool<CoinType>,
        amount: u64
    ): Coin<CoinType> {
        let coin = coin::extract<CoinType>(&mut pool.coin, amount);
        coin
    }

    public fun merge<CoinType>(
        pool: &mut CoinPool<CoinType>,
        coin: Coin<CoinType>,
    ) {
        coin::merge<CoinType>(&mut pool.coin, coin);
    }

    public fun deposit<CoinType>(
        from_account: &signer,
        owner: address,
        amount: u64,
    ) acquires CoinPool {
        assert!(
            has_pool<CoinType>(owner),
            error::not_found(ECOIN_POOL_NOT_FOUND),
        );
        let coin_in = coin::withdraw<CoinType>(from_account, amount);
        let pool = borrow_global_mut<CoinPool<CoinType>>(owner);
        merge<CoinType>(pool, coin_in);
    }

    public fun withdraw<CoinType>(
        owner: address,
        to_account: address,
        amount: u64,
    ) acquires CoinPool {
        assert!(
            has_pool<CoinType>(owner),
            error::not_found(ECOIN_POOL_NOT_FOUND),
        );
        let pool = borrow_global_mut<CoinPool<CoinType>>(owner);
        let claim_coin = coin::extract<CoinType>(&mut pool.coin, amount);
        coin::deposit<CoinType>(to_account, claim_coin);
    }
}