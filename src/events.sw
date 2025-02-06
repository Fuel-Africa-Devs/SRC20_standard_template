library;

pub struct Transfer {
    pub to: Address,
    pub amount: u64,
}

pub struct TransferFrom {
    pub from: Address,
    pub to: Address,
    pub amount: u64,
}

pub struct Approve {
    pub from: Address,
    pub spender: Address,
    pub amount: u64,
}

pub struct IncreaseAllowance {
    pub from: Address,
    pub spender: Address,
    pub amount: u64,
}

pub struct DecreaseAllowance {
    pub from: Address,
    pub spender: Address,
    pub amount: u64,
}

pub struct Mint {
    pub to: Address,
    pub amount: u64,
}
pub struct Burn {
    pub from: Address,
    pub amount: u64,
}
