module injoy_labs::injoy_nft_launchpad {

    use std::string::{Self, String};
    use std::signer;
    use std::error;
    use aptos_token::token;
    use aptos_framework::account;
    use aptos_framework::aptos_account;
    use aptos_framework::timestamp;
    use aptos_framework::table;

    const EAGENT_ACCOUNT_NOT_SET: u64 = 0;

    const ENOT_IN_BUYING_STAGE: u64 = 1;

    const INJOY_PREFIX: vector<u8> = b"injoy";

    struct AgentConfig has key {
        creator: address,
        beneficiary: address,
        presale_mint_fee: u64,
        public_mint_fee: u64 ,
        presale_start_time: u64,
        presale_end_time: u64,
        public_start_time: u64,
        signer_capability: account::SignerCapability,
        token_id_counter: u64,
    }

    public entry fun create_collection(
        creator: &signer,
        // 0x3::token -- Unique name within this creators account for this collection
        name: String,
        // 0x3::token -- Describes the collection
        description: String,
        // 0x3::token -- Base URI for every token, base_uri{0} is cover and base_uri{i} for token i
        base_uri: String,
        // 0x3::token -- Maximum number of token_data allowed within this collections
        maximum: u64,
        // 0x3::token -- Mutate setting [description, uri, maximum]
        mutate_settings: vector<bool>,
        // this -- Account to receive fund
        beneficiary: address,
        // this -- Fee for minting token in presale stage
        presale_mint_fee: u64,
        // this -- Fee for minting token in public stage
        public_mint_fee: u64,
        // this -- Start time of presale stage
        presale_start_time: u64,
        // this -- End time of presale stage
        presale_end_time: u64,
        // this -- Start time of public stage
        public_start_time: u64,
    ) {
        let cover_uri = base_uri;
        string::append(&mut cover_uri, string::utf8(b"0"));
        token::create_collection(
            creator,
            name,
            description,
            cover_uri,
            maximum,
            mutate_settings,
        );
        let seed = string::bytes(&name);
        let (resource_signer, resource_signer_cap) = account::create_resource_account(creator, *seed);
        let creator_addr = signer::address_of(creator);
        let agent_config = AgentConfig {
            creator: creator_addr,
            beneficiary,
            presale_mint_fee,
            public_mint_fee,
            presale_start_time,
            presale_end_time,
            public_start_time,
            signer_capability: resource_signer_cap,
            token_id_counter: 1,
        };
            
        move_to(&resource_signer, agent_config);
    }

    public entry fun mint_tokens(
        buyer: &signer,
        creator: address,
        collection: String,
        amount: u64,
    ) acquires AgentConfig {
        let agent_addr = account::create_resource_address(
            &creator, *string::bytes(&collection)
        );
        
        assert!(
            exists<AgentConfig>(agent_addr),
            error::not_found(EAGENT_ACCOUNT_NOT_SET),
        );

        let agent_config = borrow_global<AgentConfig>(agent_addr);

        let presale_stage = false;
        let public_stage = false;
        let now_timestamp = timestamp::now_microseconds();
        if (
            now_timestamp >= agent_config.presale_start_time &&
            now_timestamp <= agent_config.presale_end_time
        ) {
            presale_stage = true;
        };
        if (
            now_timestamp >= agent_config.public_start_time
        ) {
            public_stage = true;
        };
        assert!(presale_stage || public_stage, error::permission_denied(ENOT_IN_BUYING_STAGE));
        aptos_account::transfer(buyer, agent_config.beneficiary, 0);
    }
}