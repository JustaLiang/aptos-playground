#[test_only]
module deployer::coin_pool_test {
    use std::signer::address_of;
    use std::debug;
    use aptos_framework::account::create_account_for_test;
    use aptos_framework::coin::{Self, FakeMoney};

    // target module
    use deployer::coin_pool;

    fun setup_account(account: &signer): address {
        let addr = address_of(account);
        create_account_for_test(addr);
        addr
    }

    // section 1
    #[test(pooler=@0x11)]
    fun owner_can_create_pool(pooler: signer) {
        let pooler_addr = setup_account(&pooler);

        coin_pool::create_pool<FakeMoney>(&pooler);
        assert!(coin_pool::has_pool<FakeMoney>(pooler_addr), 10);
    }

    // section 2
    #[test(coin_owner=@0x1, pooler=@0x21, player=@0x22)]
    fun player_can_put_in(
        coin_owner: signer,
        pooler: signer,
        player: signer,
    ): (signer, signer, signer) {
        setup_account(&coin_owner);
        let pooler_addr = setup_account(&pooler);
        let player_addr = setup_account(&player);

        let amount: u64 = 100;
        coin::create_fake_money(&coin_owner, &player, amount);

        coin::transfer<FakeMoney>(&coin_owner, player_addr, amount);
        let balance_before = coin::balance<FakeMoney>(player_addr);
        assert!(balance_before == amount, 20);

        coin_pool::create_pool<FakeMoney>(&pooler);

        let put_in_amount = 20;
        coin_pool::put_in<FakeMoney>(&player, pooler_addr, put_in_amount);
        let balance_after = coin::balance<FakeMoney>(player_addr);
        assert!(balance_after == balance_before - put_in_amount, 21);
        assert!(coin_pool::pool_liquidity<FakeMoney>(pooler_addr) == put_in_amount, 23);
        (coin_owner, pooler, player)
    }

    // section 3
    #[test(coin_owner=@0x1, pooler=@0x31, player=@0x32)]
    #[expected_failure(abort_code = 0x10006)]
    fun player_can_put_in_and_take_out(
        coin_owner: signer,
        pooler: signer,
        player: signer,
    ) {
        let (_, pooler, player) = player_can_put_in(
            coin_owner,
            pooler,
            player,
        );
        let pooler_addr = address_of(&pooler);
        let player_addr = address_of(&player);
        let pool_liquidity = coin_pool::pool_liquidity<FakeMoney>(pooler_addr);
        debug::print(&pool_liquidity);

        let take_out_amount = pool_liquidity + 1;
        debug::print(&take_out_amount);
        coin_pool::take_out<FakeMoney>(&pooler, player_addr, take_out_amount);
    }
}