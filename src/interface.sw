library;
use std::string::String;
abi Transactions {
    #[storage(read, write)]
    fn initializer(name: str[9], symbol: str[6], decimal: u8, total_supply: u64);
    #[storage(read)]
    fn balance_of(account: Identity) -> u64;
    #[storage(read)]
    fn owner() -> Identity;
    #[storage(read)]
    fn allowance(owner: Identity, spender: Identity) -> u64;
    #[storage(read, write)]
    fn mint(user: Identity, amount: u64);
    #[storage(read, write)]
    fn transfer(recipient: Identity, amount: u64);
    #[storage(read, write)]
    fn transfer_from(owner: Identity, recipient: Identity, amount: u64);
    #[storage(write)]
    fn approve(spender: Identity, amount: u64);
    #[storage(read, write)]
    fn increase_allowance(spender: Identity, amount: u64);
    #[storage(read, write)]
    fn decrease_allowance(spender: Identity, amount: u64);
    #[storage(read, write)]
    fn burn(caller: Identity, amount: u64);
    #[storage(read)]
    fn get_asset_id() -> AssetId;
}
