#[test_only]
module deployer::coin_pool_test {
    use std::signer::address_of;
    // use std::debug;
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
    #[test(owner=@0x11)]
    fun owner_can_create_pool(owner: &signer) {
        let owner_addr = setup_account(owner);

        coin_pool::create_pool<FakeMoney>(owner);
        assert!(coin_pool::has_pool<FakeMoney>(owner_addr), 10);
    }

    // section 2
    #[test(coin_owner=@0x1, owner=@0x21, player=@0x22)]
    fun player_can_deposit(
        coin_owner: &signer,
        owner: &signer,
        player: &signer,
    ) {
        setup_account(coin_owner);
        let owner_addr = setup_account(owner);
        let player_addr = setup_account(player);

        let amount: u64 = 100;
        coin::create_fake_money(coin_owner, player, amount);

        coin::transfer<FakeMoney>(coin_owner, player_addr, amount);
        let balance_before = coin::balance<FakeMoney>(player_addr);
        assert!(balance_before == amount, 20);

        coin_pool::create_pool<FakeMoney>(owner);

        let put_in_amount = 20;
        coin_pool::deposit<FakeMoney>(player, owner_addr, put_in_amount);
        let balance_after = coin::balance<FakeMoney>(player_addr);
        assert!(balance_after == balance_before - put_in_amount, 21);
        assert!(coin_pool::pool_liquidity<FakeMoney>(owner_addr) == put_in_amount, 23);
    }

    // section 3
    #[test(coin_owner=@0x1, owner=@0x31, player=@0x32)]
    #[expected_failure(abort_code = 0x10006)]
    fun player_withdraw_too_much(
        coin_owner: &signer,
        owner: &signer,
        player: &signer,
    ) {
        player_can_deposit(
            coin_owner,
            owner,
            player,
        );
        let owner_addr = address_of(owner);
        let player_addr = address_of(player);
        let pool_liquidity = coin_pool::pool_liquidity<FakeMoney>(owner_addr);

        let take_out_amount = pool_liquidity + 1;
        coin_pool::withdraw<FakeMoney>(owner_addr, player_addr, take_out_amount);
    }

    // section 4
    #[test(owner=@0x41)]
    fun owner_destroy_empty_pool(
        owner: &signer,
    ) {
        owner_can_create_pool(owner);
        coin_pool::destroy_empty_pool<FakeMoney>(owner);
    }

    // section 5
    #[test(coin_owner=@0x1, owner=@0x51, player=@0x52)]
    #[expected_failure(abort_code=0x10007)]
    fun owner_destroy_non_empty_pool(
        coin_owner: &signer,
        owner: &signer,
        player: &signer,
    ) {
        player_can_deposit(coin_owner, owner, player);
        coin_pool::destroy_empty_pool<FakeMoney>(owner);
    }
}