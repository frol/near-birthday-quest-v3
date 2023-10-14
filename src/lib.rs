use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{ext_contract, near_bindgen, require, PanicOnDefault};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Quest {
    balances: near_sdk::collections::LookupMap<near_sdk::AccountId, near_sdk::Balance>,
}

#[ext_contract(ext_staking_pool)]
trait StakingPool {
    fn get_account_staked_balance(&self, account_id: near_sdk::AccountId) -> near_sdk::Balance;
}

#[near_bindgen]
impl Quest {
    #[init]
    #[payable]
    pub fn new(operator_account_id: near_sdk::AccountId) -> Self {
        let mut balances = near_sdk::collections::LookupMap::new(b"b".to_vec());
        balances.insert(&operator_account_id, &near_sdk::env::attached_deposit());
        Self { balances }
    }

    pub fn get_balance(&self, account_id: near_sdk::AccountId) -> near_sdk::json_types::U128 {
        self.balances.get(&account_id).unwrap_or(0).into()
    }

    #[payable]
    pub fn near_deposit(&mut self) {
        let deposit = near_sdk::env::attached_deposit();
        let account_id = near_sdk::env::predecessor_account_id();
        let balance = self.balances.get(&account_id).expect(
            "User must have some balance on the contract before you can transfer tokens to them",
        );
        self.balances.insert(&account_id, &(balance + deposit));
    }

    pub fn near_transfer(
        &mut self,
        receiver_account_id: near_sdk::AccountId,
        amount: near_sdk::json_types::U128,
    ) {
        let sender_account_id = near_sdk::env::signer_account_id();
        let sender_balance = self.balances.get(&sender_account_id).expect(
            "Your account must have some balance on the contract before you can transfer tokens",
        );
        require!(amount.0 <= sender_balance, "Not enough balance to transfer");
        self.balances
            .insert(&sender_account_id, &(sender_balance - amount.0));

        let receiver_balance = self.balances.get(&receiver_account_id).unwrap_or(0);
        self.balances
            .insert(&receiver_account_id, &(receiver_balance + amount.0));
    }

    pub fn near_withdraw(&mut self, amount: near_sdk::json_types::U128) {
        let account_id = near_sdk::env::predecessor_account_id();
        let balance = self.balances.get(&account_id).unwrap_or(0);
        require!(amount.0 <= balance, "Not enough balance to withdraw");

        ext_staking_pool::ext("qbit.poolv1.near".parse().unwrap())
            .get_account_staked_balance(account_id.clone())
            .then(Self::ext(near_sdk::env::current_account_id()).on_withdraw(account_id, amount));
    }

    #[private]
    pub fn on_withdraw(
        &mut self,
        account_id: near_sdk::AccountId,
        amount: near_sdk::json_types::U128,
    ) {
        let balance = self.balances.get(&account_id).unwrap_or(0);
        self.balances.insert(&account_id, &(balance - amount.0));
        near_sdk::Promise::new(account_id).transfer(amount.0);
    }
}
