contract;
pub mod interface;
pub mod events;
use standards::src20::{SetDecimalsEvent, SetNameEvent, SetSymbolEvent, SRC20, TotalSupplyEvent};
use std::{auth::msg_sender, bytes::Bytes, hash::*, string::String};
use ::interface::Transactions;
use ::events::{Approve, Burn, DecreaseAllowance, IncreaseAllowance, Mint, Transfer, TransferFrom};
storage {
    initialized: bool = false,
    name: str[9] = __to_str_array("TokenName"),
    symbol: str[6] = __to_str_array("Symbol"),
    decimals: Option<u8> = None,
    total_supply: Option<u64> = None,
    total_assets: u64 = 0,
    owner: Address = Address::zero(),
    balances: StorageMap<Address, u64> = StorageMap {},
    allowances: StorageMap<(Address, Address), u64> = StorageMap {},
}
//  SRC20 is like an interface f or you to make any impl you want..
impl SRC20 for Contract {
    #[storage(read)]
    fn total_assets() -> u64 {
        storage.total_assets.read()
    }
    #[storage(read)]
    fn total_supply(asset: AssetId) -> Option<u64> {
        if asset == AssetId::default() {
            storage.total_supply.read()
        } else {
            None
        }
    }
    #[storage(read)]
    fn name(asset: AssetId) -> Option<String> {
        if asset == AssetId::default() {
            Some(String::from_ascii_str(from_str_array(storage.name.read())))
        } else {
            None
        }
    }
    #[storage(read)]
    fn symbol(asset: AssetId) -> Option<String> {
        if asset == AssetId::default() {
            Some(String::from_ascii_str(from_str_array(storage.symbol.read())))
        } else {
            None
        }
    }
    #[storage(read)]
    fn decimals(asset: AssetId) -> Option<u8> {
        if asset == AssetId::default() {
            storage.decimals.read()
        } else {
            None
        }
    }
}
impl Transactions for Contract {
    #[storage(read, write)]
    fn initializer(name: str[9], symbol: str[6], decimal: u8, total_supply: u64) {
        let sender = msg_sender().unwrap();
        let asset = AssetId::default();
        assert(storage.initialized.read() == false); // "Storage already initialized"
        let owner = msg_sender().unwrap();
        storage.name.write(name);
        storage.symbol.write(symbol);
        storage.decimals.write(Some(decimal));
        storage.total_supply.write(Some(total_supply));
        storage.initialized.write(true);
        match owner {
            Identity::Address(address) => {
                storage.owner.write(address);
            },
            Identity::ContractId(_) => (),
        }
        log(SetNameEvent::new(
            asset,
            Some(String::from_ascii_str(from_str_array(storage.name.read()))),
            sender,
        ));
        SetSymbolEvent::new(
            asset,
            Some(String::from_ascii_str(from_str_array(storage.symbol.read()))),
            sender,
        )
            .log();
        SetDecimalsEvent::new(asset, storage.decimals.read().unwrap(), sender)
            .log();
        TotalSupplyEvent::new(asset, storage.total_supply.read().unwrap(), sender)
            .log();
    }
    #[storage(read)]
    fn balance_of(account: Identity) -> u64 {
        match account {
            Identity::Address(address) => {
                storage.balances.get(address).try_read().unwrap_or(0)
            },
            Identity::ContractId(_) => 0,
        }
    }
    #[storage(read)]
    fn owner() -> Identity {
        Identity::Address(storage.owner.read())
    }
    #[storage(read)]
    fn allowance(owner: Identity, spender: Identity) -> u64 {
        match owner {
            Identity::Address(owner_address) => match spender {
                Identity::Address(spender_address) => {
                    storage.allowances.get((owner_address, spender_address)).try_read().unwrap_or(0)
                },
                Identity::ContractId(_) => 0,
            },
            Identity::ContractId(_) => 0,
        }
    }
    #[storage(read, write)]
    fn mint(user: Identity, amount: u64) {
        assert(storage.total_supply.read().unwrap_or(0) >= storage.total_assets.read()); //more tokens cant be minted
        match user {
            Identity::Address(user_address) => {
                // Update the recipient's balance
                let current_balance = storage.balances.get(user_address).try_read().unwrap_or(0);
                storage
                    .balances
                    .insert(user_address, current_balance + amount);
                storage
                    .total_assets
                    .write(storage.total_assets.read() + amount);
                log(Mint {
                    to: user_address,
                    amount,
                });
            },
            _ => (),
        }
    }
    #[storage(read, write)]
    fn transfer(recipient: Identity, amount: u64) {
        let caller = msg_sender().unwrap();
        match caller {
            Identity::Address(caller_address) => {
                // Get the caller's current balance
                let caller_balance = storage.balances.get(caller_address).try_read().unwrap_or(0);
                // Ensure the caller has enough tokens
                assert(caller_balance >= amount);
                // Reduce caller's balance
                storage
                    .balances
                    .insert(caller_address, caller_balance - amount);
                // Increase recipient's balance
                match recipient {
                    Identity::Address(to_address) => {
                        let recipient_balance = storage.balances.get(to_address).try_read().unwrap_or(0);
                        storage
                            .balances
                            .insert(to_address, recipient_balance + amount);
                        log(Transfer {
                            to: to_address,
                            amount,
                        });
                    },
                    _ => (),
                }
            },
            Identity::ContractId => (),
        }
    }
    #[storage(read, write)]
    fn transfer_from(owner: Identity, recipient: Identity, amount: u64) {
        let caller = msg_sender().unwrap();
        assert(caller == owner);
        match owner {
            Identity::Address(owner_address) => {
                match recipient {
                    Identity::Address(recipient_address) => {
                        let current_allowance = storage.allowances.get((owner_address, recipient_address)).try_read().unwrap_or(0);
                        assert(current_allowance >= amount);

                        let owner_balance = storage.balances.get(owner_address).try_read().unwrap_or(0);
                        assert(owner_balance <= amount);
                        storage
                            .balances
                            .insert(owner_address, owner_balance - amount);
                        let recipient_balance = storage.balances.get(recipient_address).try_read().unwrap_or(0);
                        storage
                            .balances
                            .insert(recipient_address, recipient_balance + amount);
                        // Reduce allowance
                        storage
                            .allowances
                            .insert(
                                (owner_address, recipient_address),
                                current_allowance - amount,
                            );
                        log(TransferFrom {
                            from: owner_address,
                            to: recipient_address,
                            amount,
                        });
                    },
                    Identity::ContractId(_) => (),
                }
            },
            Identity::ContractId(_) => (),
        }
    }
    #[storage(write)]
    fn approve(spender: Identity, amount: u64) {
        let caller = msg_sender().unwrap();
        match caller {
            Identity::Address(caller_address) => {
                match spender {
                    Identity::Address(spender_address) => {
                        storage
                            .allowances
                            .insert((caller_address, spender_address), amount);
                        log(Approve {
                            from: caller_address,
                            spender: spender_address,
                            amount,
                        });
                    },
                    Identity::ContractId(_) => (),
                }
            },
            Identity::ContractId(_) => (),
        }
    }
    #[storage(read, write)]
    fn increase_allowance(spender: Identity, amount: u64) {
        let caller = msg_sender().unwrap();
        match caller {
            Identity::Address(caller_address) => {
                match spender {
                    Identity::Address(spender_address) => {
                        let current_allowance = storage.allowances.get((caller_address, spender_address)).try_read().unwrap_or(0);
                        storage
                            .allowances
                            .insert(
                                (caller_address, spender_address),
                                current_allowance + amount,
                            );
                        log(IncreaseAllowance {
                            from: caller_address,
                            spender: spender_address,
                            amount,
                        });
                    },
                    Identity::ContractId(_) => (),
                }
            },
            Identity::ContractId(_) => (),
        }
    }
    #[storage(read, write)]
    fn decrease_allowance(spender: Identity, amount: u64) {
        let caller = msg_sender().unwrap();
        match caller {
            Identity::Address(caller_address) => {
                match spender {
                    Identity::Address(spender_address) => {
                        let current_allowance = storage.allowances.get((caller_address, spender_address)).try_read().unwrap_or(0);
                        assert(current_allowance >= amount);
                        storage
                            .allowances
                            .insert(
                                (caller_address, spender_address),
                                current_allowance - amount,
                            );
                        log(DecreaseAllowance {
                            from: caller_address,
                            spender: spender_address,
                            amount,
                        });
                    },
                    Identity::ContractId(_) => (),
                }
            },
            Identity::ContractId(_) => (),
        }
    }
    #[storage(read, write)]
    fn burn(caller: Identity, amount: u64) {
        assert(msg_sender().unwrap() == caller);
        match caller {
            Identity::Address(caller_address) => {
                // Get the current balance of the caller
                let current_balance = storage.balances.get(caller_address).try_read().unwrap_or(0);
                let total_supply = storage.total_supply.read().unwrap_or(0);
                // Ensure the caller has enough tokens to burn
                assert(current_balance >= amount);

                // Update the caller's balance
                storage
                    .balances
                    .insert(caller_address, current_balance - amount);

                // Decrease the total supply
                storage.total_supply.write(Some(total_supply - amount));
                storage
                    .total_assets
                    .write(storage.total_assets.read() - amount);

                // Log the burn event
                log(Burn {
                    from: caller_address,
                    amount,
                });
            },
            Identity::ContractId(_) => (),
        }
    }
    #[storage(read)]
    fn get_asset_id() -> AssetId {
        AssetId::default()
    }
}
