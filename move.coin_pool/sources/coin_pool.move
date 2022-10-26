module injoy_labs::coin_pool {
    use aptos_framework::coin::{Self, Coin};
    use aptos_framework::signer;

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
    spec has_pool {
        aborts_if false;
    }

    public fun pool_liquidity<CoinType>(addr: address): u64 acquires CoinPool {
        assert!(
            has_pool<CoinType>(addr),
            error::not_found(ECOIN_POOL_NOT_FOUND),
        );
        let coin_pool = borrow_global<CoinPool<CoinType>>(addr);
        coin::value<CoinType>(&coin_pool.coin)
    }
    spec pool_liquidity {
        aborts_if !exists<CoinPool<CoinType>>(addr);
    }

    /// -----------------
    /// Public functions
    /// -----------------

    public fun create_pool<CoinType>(account: &signer) {
        assert!(
            !has_pool<CoinType>(signer::address_of(account)),
            error::already_exists(ECOIN_POOL_ALREADY_EXISTS),
        );
        let coin_pool = CoinPool { coin: coin::zero<CoinType>() };
        move_to(account, coin_pool);
    }
    spec create_pool {
        let addr = signer::address_of(account);
        aborts_if exists<CoinPool<CoinType>>(addr);

        ensures exists<CoinPool<CoinType>>(addr);
    }

    public fun destroy_empty_pool<CoinType>(owner: address) acquires CoinPool {
        assert!(
            has_pool<CoinType>(owner),
            error::not_found(ECOIN_POOL_NOT_FOUND),
        );
        let CoinPool { coin } = move_from<CoinPool<CoinType>>(owner);
        coin::destroy_zero(coin);
    }
    spec destroy_empty_pool {
        let liquidity = global<CoinPool<CoinType>>(owner).coin.value;
        aborts_if !exists<CoinPool<CoinType>>(owner);
        aborts_if liquidity != 0;

        ensures !exists<CoinPool<CoinType>>(owner);
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
        pool_owner: address,
        amount: u64,
    ) acquires CoinPool {
        assert!(
            has_pool<CoinType>(pool_owner),
            error::not_found(ECOIN_POOL_NOT_FOUND),
        );
        let coin_in = coin::withdraw<CoinType>(from_account, amount);
        let pool = borrow_global_mut<CoinPool<CoinType>>(pool_owner);
        merge<CoinType>(pool, coin_in);
    }
    spec deposit {
        pragma aborts_if_is_partial;

        let from_addr = signer::address_of(from_account);
        let from_balance = coin::balance<CoinType>(from_addr);
        let liquidity = pool_liquidity<CoinType>(pool_owner);
        aborts_if !exists<CoinPool<CoinType>>(pool_owner);
        
        let post post_from_balance = coin::balance<CoinType>(from_addr);
        let post post_liquidity = pool_liquidity<CoinType>(pool_owner);
        ensures from_balance - post_from_balance == amount;
        ensures post_liquidity - liquidity == amount;
    }

    public fun withdraw<CoinType>(
        pool_owner: address,
        to_address: address,
        amount: u64,
    ) acquires CoinPool {
        assert!(
            has_pool<CoinType>(pool_owner),
            error::not_found(ECOIN_POOL_NOT_FOUND),
        );
        let pool = borrow_global_mut<CoinPool<CoinType>>(pool_owner);
        let claim_coin = coin::extract<CoinType>(&mut pool.coin, amount);
        coin::deposit<CoinType>(to_address, claim_coin);
    }
    spec withdraw {
        pragma aborts_if_is_partial;

        let to_balance = coin::balance<CoinType>(to_address);
        let liquidity = pool_liquidity<CoinType>(pool_owner);
        aborts_if !exists<CoinPool<CoinType>>(pool_owner);
        
        let post post_to_balance = coin::balance<CoinType>(to_address);
        let post post_liquidity = pool_liquidity<CoinType>(pool_owner);
        ensures post_to_balance - to_balance == amount;
        ensures liquidity - post_liquidity == amount;
    }
}