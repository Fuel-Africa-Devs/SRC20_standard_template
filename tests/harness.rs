use fuels::types::{Identity, SizedAsciiString};
use fuels::{prelude::*, types::ContractId};
use std::str::FromStr;
// Load abi from json
abigen!(Contract(
    name = "MyContract",
    abi = "out/debug/SRC20_standard-abi.json"
));

async fn get_contract_instance() -> (MyContract<WalletUnlocked>, ContractId) {
    // Launch a local network and deploy the contract
    let mut wallets = launch_custom_provider_and_get_wallets(
        WalletsConfig::new(
            Some(1),             /* Single wallet */
            Some(1),             /* Single coin (UTXO) */
            Some(1_000_000_000), /* Amount per coin */
        ),
        None,
        None,
    )
    .await
    .unwrap();
    let wallet = wallets.pop().unwrap();

    let id = Contract::load_from(
        "./out/debug/SRC20_standard.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap();

    let instance = MyContract::new(id.clone(), wallet);

    (instance, id.into())
}

#[tokio::test]
async fn can_get_contract_id() {
    let (_instance, _id) = get_contract_instance().await;

    // Now you have an instance of your contract you can use to test each function
}

#[tokio::test]
async fn test_token_metadata() {
    let (instance, id) = get_contract_instance().await;
    let fake_asset_id = instance
        .methods()
        .get_asset_id()
        .call()
        .await
        .unwrap()
        .value;
    // let real_asset_id = AssetId::new(id).unwrap();
    let token_name = "mayowaTKN";
    let token_symbol = "mayowa";
    let name = SizedAsciiString::new(token_name.to_string()).expect("name too long");
    let symbol = SizedAsciiString::new(token_symbol.to_string()).expect("symbol too long");
    let decimal = 8;
    let total_supply = 1_000_000_000;

    let data = instance
        .methods()
        .initializer(name, symbol, decimal, total_supply)
        .call()
        .await
        .unwrap();
    println!("logs {:?}", data.decode_logs().filter_succeeded());
    let name = instance.methods().name(fake_asset_id).call().await.unwrap();
    println!("token name is {}", name.value.clone().unwrap());
    assert_eq!(name.value.unwrap(), token_name.to_string());
    let symbol = instance
        .methods()
        .symbol(fake_asset_id)
        .call()
        .await
        .unwrap();
    println!("token symbol is  {:?}", symbol.value.clone().unwrap());
    assert_eq!(symbol.value.unwrap(), token_symbol.to_string());

    let decimals = instance
        .methods()
        .decimals(fake_asset_id)
        .call()
        .await
        .unwrap();
    println!("token decimals are  {:?}", decimals.value.clone().unwrap());
    assert_eq!(decimals.value.unwrap(), decimal);
}

#[tokio::test]
async fn test_transfer_tokens() {
    let (instance, id) = get_contract_instance().await;
    // println!("{:?}instance is ", instance.account().address());
    let binding = instance.account();
    let contract_address = binding.address();

    let wallet = WalletUnlocked::new_random(None);
    let recipient_wallet = WalletUnlocked::new_random(None);

    let user1 = "0x0000000000000000000000000000000000000000000000000000000000000000";
    let user2 = "0x0000000000000000000000000000000000000000000000000000000000000001";
    let user2_address = Address::from_str(user2).unwrap();
    let sender = Identity::Address(Address::from(binding.address()));
    let recipient = Identity::Address(Address::from(recipient_wallet.address()));
    let amount = 1_000;

    // Mint tokens to the sender first
    let mint_result = instance
        .methods()
        .mint(sender.clone(), 10_000)
        .call()
        .await
        .unwrap();
    println!(
        "logs minted {:?}",
        mint_result.decode_logs().filter_succeeded()
    );

    // Check sender's balance before transfer
    let sender_balance = instance
        .methods()
        .balance_of(sender.clone())
        .call()
        .await
        .unwrap();
    println!("Sender balance before transfer: {:?}", sender_balance.value);
    assert_eq!(sender_balance.value, 10_000);

    // Transfer tokens

    let transfer_result = instance
        .clone()
        .with_account(binding)
        .methods()
        .transfer(recipient.clone(), amount)
        .call()
        .await
        .unwrap();

    // // Check balances after transfer
    let sender_balance = instance
        .methods()
        .balance_of(sender.clone())
        .call()
        .await
        .unwrap();
    let recipient_balance = instance
        .methods()
        .balance_of(recipient.clone())
        .call()
        .await
        .unwrap();

    println!("Sender balance after transfer: {:?}", sender_balance.value);
    println!(
        "Recipient balance after transfer: {:?}",
        recipient_balance.value
    );

    assert_eq!(sender_balance.value, 9_000);
    assert_eq!(recipient_balance.value, amount);
}

#[tokio::test]
async fn test_approve_and_allowance() {
    let (instance, _) = get_contract_instance().await;
    let binding = instance.account();

    let owner = Identity::Address(Address::from(binding.address()));
    let user2 = "0x0000000000000000000000000000000000000000000000000000000000000001";

    let spender = Identity::Address(Address::from_str(user2).unwrap());
    let allowance_amount = 5_000;

    // Approve allowance
    let approve_result = instance
        .clone()
        .with_account(binding)
        .methods()
        .approve(spender.clone(), allowance_amount)
        .call()
        .await
        .unwrap();

    // Check allowance
    let allowance = instance
        .methods()
        .allowance(owner.clone(), spender.clone())
        .call()
        .await
        .unwrap();
    println!("Allowance set is {:?}", allowance.value);
    assert_eq!(allowance.value, allowance_amount);
}

#[tokio::test]
async fn test_increase_and_decrease_allowance() {
    let (instance, _) = get_contract_instance().await;
    let binding = instance.account();

    let owner = Identity::Address(Address::from(binding.address()));
    let user2 = "0x0000000000000000000000000000000000000000000000000000000000000001";

    let spender = Identity::Address(Address::from_str(user2).unwrap());
    let initial_allowance = 3_000;

    // Approve initial allowance
    instance
        .clone()
        .with_account(binding.clone())
        .methods()
        .approve(spender.clone(), initial_allowance)
        .call()
        .await
        .unwrap();

    // // Increase allowance
    let increase_amount = 2_000;
    let increase_result = instance
        .clone()
        .with_account(binding.clone())
        .methods()
        .increase_allowance(spender.clone(), increase_amount)
        .call()
        .await
        .unwrap();

    let updated_allowance = instance
        .clone()
        .with_account(binding.clone())
        .methods()
        .allowance(owner.clone(), spender.clone())
        .call()
        .await
        .unwrap();
    println!("Allowance after increase: {:?}", updated_allowance.value);
    assert_eq!(updated_allowance.value, 5_000);

    // // Decrease allowance
    let decrease_amount = 1_000;
    let decrease_result = instance
        .clone()
        .with_account(binding.clone())
        .methods()
        .decrease_allowance(spender.clone(), decrease_amount)
        .call()
        .await
        .unwrap();

    let final_allowance = instance
        .clone()
        .with_account(binding)
        .methods()
        .allowance(owner.clone(), spender.clone())
        .call()
        .await
        .unwrap();
    println!("Allowance after decrease: {:?}", final_allowance.value);
    assert_eq!(final_allowance.value, 4_000);
}

#[tokio::test]
async fn test_transfer_from_with_connected_wallet() {
    let (instance, id) = get_contract_instance().await;

    // Set up the provider and wallets
    let binding = instance.account();
    let contract_address = binding.address();

    // let wallet = WalletUnlocked::new_random(None);
    let recipient_wallet = WalletUnlocked::new_random(None);
    // Use the address from the recipient's wallet
    let recipient_identity = Identity::Address(Address::from(recipient_wallet.address()));
    let sender = Identity::Address(Address::from(binding.address()));

    let amount = 1_000;

    // Mint tokens to the wallet first (ensure the wallet has enough tokens for transfer)
    let mint_result = instance
        .clone()
        .with_account(binding.clone())
        .methods()
        .mint(sender, amount)
        .call()
        .await
        .unwrap();
    println!("Mint result: {:?}", mint_result);

    // Assert the wallet's balance after minting
    let balance_after_mint = instance.methods().balance_of(sender).call().await.unwrap();
    assert_eq!(
        balance_after_mint.value, amount,
        "Minted tokens don't match expected balance"
    );

    // Approve the wallet to transfer tokens from the user's balance
    let approve_result = instance
        .clone()
        .with_account(binding.clone())
        .methods()
        .approve(recipient_identity, amount)
        .call()
        .await
        .unwrap();
    println!("Approve result: {:?}", approve_result);

    // Assert the allowance
    let allowance = instance
        .methods()
        .allowance(sender, recipient_identity)
        .call()
        .await
        .unwrap();
    assert_eq!(
        allowance.value, amount,
        "Allowance didn't match expected value"
    );

    // Now use `transferFrom` to transfer tokens on behalf of the wallet
    let transfer_from_result = instance
        .clone()
        .with_account(binding.clone()) // This wallet will initiate the transfer
        .methods()
        .transfer_from(sender, recipient_identity, amount)
        .call()
        .await
        .unwrap();
    println!("Transfer From result: {:?}", transfer_from_result);

    // Assert the wallet's balance after transfer
    let balance_after_transfer = instance.methods().balance_of(sender).call().await.unwrap();
    assert_eq!(
        balance_after_transfer.value, 0,
        "Wallet balance didn't decrease as expected"
    );

    // Assert the recipient's balance after transfer
    let balance_after_recipient_transfer = instance
        .methods()
        .balance_of(recipient_identity)
        .call()
        .await
        .unwrap();
    assert_eq!(
        balance_after_recipient_transfer.value, amount,
        "Recipient balance didn't increase as expected"
    );
}
#[tokio::test]
async fn test_burn_tokens() {
    let (instance, id) = get_contract_instance().await;
    let binding = instance.account();
    let holder = Identity::Address(Address::from(binding.address()));
    let fake_asset_id = instance
        .methods()
        .get_asset_id()
        .call()
        .await
        .unwrap()
        .value;

    let token_name = "mayowaTKN";
    let token_symbol = "mayowa";
    let name = SizedAsciiString::new(token_name.to_string()).expect("name too long");
    let symbol = SizedAsciiString::new(token_symbol.to_string()).expect("symbol too long");
    let decimal = 8;
    let total_supply = 1_000_000_000;

    let data = instance
        .methods()
        .initializer(name, symbol, decimal, total_supply)
        .call()
        .await
        .unwrap();
    // Mint tokens first
    instance
        .methods()
        .mint(holder.clone(), 5_000)
        .call()
        .await
        .unwrap();
    let total_supply_before_burn = instance
        .methods()
        .total_supply(fake_asset_id)
        .call()
        .await
        .unwrap();
    println!(
        "total supply before burn is {:?}",
        total_supply_before_burn.value.unwrap()
    );
    let total_asset_after_mint = instance.methods().total_assets().call().await.unwrap();
    println!(
        "total asset  before burn is {:?}",
        total_asset_after_mint.value
    );
    // Check initial balance
    let initial_balance = instance
        .methods()
        .balance_of(holder.clone())
        .call()
        .await
        .unwrap();
    println!("Initial balance before burn: {:?}", initial_balance.value);
    assert_eq!(initial_balance.value, 5_000);

    // Burn tokens
    let burn_amount = 2_000;
    let burn_result = instance
        .methods()
        .burn(holder.clone(), burn_amount)
        .call()
        .await
        .unwrap();

    let total_asset_after_burn = instance.methods().total_assets().call().await.unwrap();
    println!(
        "total asset  after burn is {:?}",
        total_asset_after_burn.value
    );
    assert!(total_asset_after_burn.value < total_asset_after_mint.value);

    let total_supply_after_burn = instance
        .methods()
        .total_supply(fake_asset_id)
        .call()
        .await
        .unwrap();
    println!(
        "total supply after burn is {:?}",
        total_supply_after_burn.value.unwrap()
    );

    assert!(total_supply_before_burn.value > total_supply_after_burn.value);
    // // Check balance after burn
    let final_balance = instance
        .methods()
        .balance_of(holder.clone())
        .call()
        .await
        .unwrap();
    println!("Final balance after burn: {:?}", final_balance.value);
    assert_eq!(final_balance.value, 3_000);
}
