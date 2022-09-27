module justa::coin_pool {
    use aptos_framework::coin::{Self, Coin};

    struct CoinPool<phantom CoinType> has key {
        coin: Coin<CoinType>,
    }

    public fun create_pool<CoinType>(account: &signer) {
        let coin_pool = CoinPool { coin: coin::zero<CoinType>() };
        move_to(account, coin_pool);
    }

    public fun destroy_empty_pool<CoinType>(account: address) acquires CoinPool {
        let CoinPool { coin } = move_from<CoinPool<CoinType>>(account);
        coin::destroy_zero(coin);
    }

    public fun put_in<CoinType>(
        putter: &signer,
        pool_owner: address,
        amount: u64,
    ) acquires CoinPool {
        let coin_in = coin::withdraw<CoinType>(putter, amount);
        let pool = borrow_global_mut<CoinPool<CoinType>>(pool_owner);
        coin::merge(&mut pool.coin, coin_in);
    }

    public fun take_out<CoinType>(
        taker: address,
        pool_owner: address,
        amount: u64,
    ) acquires CoinPool {
        let pool = borrow_global_mut<CoinPool<CoinType>>(pool_owner);
        let claim_coin = coin::extract<CoinType>(&mut pool.coin, amount);
        coin::deposit<CoinType>(taker, claim_coin);
    }
}