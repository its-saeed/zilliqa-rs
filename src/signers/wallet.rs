/// FIXME: NOT BELONG TO ANY MODULE FOR NOW
///
use std::{collections::HashMap, rc::Rc};

use jsonrpsee::rpc_params;

use crate::{
    blockchain::BalanceResponse,
    core::HTTPProvider,
    crypto::util::{generate_private_key, get_address_from_public_key},
};

use super::{error::AccountError, Account, Transaction};

pub struct Wallet {
    default_account: Option<Account>,
    provider: Rc<HTTPProvider>,
    accounts: HashMap<String, Account>,
}

impl Wallet {
    pub fn new(provider: Rc<HTTPProvider>) -> Self {
        Self {
            default_account: None,
            provider,
            accounts: HashMap::new(),
        }
    }

    pub fn new_with_accounts(accounts: Vec<Account>, provider: Rc<HTTPProvider>) -> Self {
        // TODO: Consider using refs here
        let default_account = if !accounts.is_empty() {
            Some(accounts[0].clone())
        } else {
            None
        };

        let accounts = accounts
            .into_iter()
            // TODO: Consider using refs here
            .map(|account| (account.address.clone(), account))
            .collect::<HashMap<_, _>>();

        Self {
            accounts,
            provider,
            default_account,
        }
    }

    pub fn create(&mut self) -> Result<String, AccountError> {
        let private_key = generate_private_key();
        self.add_by_private_key(&private_key)
    }

    pub fn add_by_private_key(&mut self, private_key: &str) -> Result<String, AccountError> {
        let account = Account::new(private_key)?;
        let address = account.address.clone();
        if self.default_account.is_none() {
            self.default_account = Some(account.clone())
        }
        self.accounts.insert(account.address.clone(), account);
        Ok(address)
    }

    pub fn remove(&mut self, address: &str) -> Option<Account> {
        if let Some(account) = &self.default_account {
            if account.address == address {
                self.default_account = None;
            }
        }
        self.accounts.remove(address)
    }

    pub fn set_default(&mut self, address: &str) -> Result<(), AccountError> {
        let account = self
            .accounts
            .get(address)
            .ok_or(AccountError::AccountDoesNotExist(address.to_string()))?;

        self.default_account = Some(account.clone());

        Ok(())
    }

    pub fn default_account(&self) -> Option<&Account> {
        self.default_account.as_ref()
    }

    pub async fn nonce(&self, account: &Account) -> Result<u64, AccountError> {
        let response: BalanceResponse = self
            .provider
            .send(crate::core::RPCMethod::GetBalance.to_string(), rpc_params![&account.address])
            .await?;

        Ok(response.nonce)
    }

    pub async fn sign_transaction(&self, mut tx: Transaction) -> Result<Transaction, AccountError> {
        let account = if let Some(pub_key) = &tx.pub_key {
            let address = get_address_from_public_key(pub_key)?;
            self.accounts.get(&address).ok_or(AccountError::AccountDoesNotExist(address))
        } else if let Some(account) = &self.default_account {
            Ok(account)
        } else {
            Err(AccountError::NeitherPubKeyNorDefaultAccountProvided)
        }?;

        // TODO: Is it a sane condition?
        if tx.nonce == u64::default() {
            tx.nonce = self.nonce(account).await? + 1;
        }

        Ok(account.sign_transaction(tx))
    }
}

// TODO: Re-enable them. Need to mock http provider.
// #[cfg(test)]
// mod tests {
//     use claim::assert_none;

//     use crate::{
//         crypto::util::generate_private_key,
//         util::validation::{is_address, is_private_key},
//     };

//     use super::Wallet;

//     #[test]
//     fn wallet_create_function_should_create_a_new_account() {
//         let mut wallet = Wallet::new();
//         let address = wallet.create().unwrap();
//         assert!(is_address(&address));
//         assert_eq!(wallet.accounts.len(), 1);

//         let account = wallet.accounts.get(&address).unwrap();

//         assert!(is_private_key(&account.private_key));

//         assert!(wallet.default_account.is_some());
//         assert_eq!(wallet.default_account.unwrap(), *account);
//     }

//     #[test]
//     fn add_by_private_key_function_should_create_a_new_account_in_wallet() {
//         let mut wallet = Wallet::new();
//         let private_key = generate_private_key();

//         let address = wallet.add_by_private_key(&private_key).unwrap();
//         assert!(is_address(&address));
//         assert_eq!(wallet.accounts.len(), 1);

//         let account = wallet.accounts.get(&address).unwrap();

//         assert!(is_private_key(&account.private_key));

//         assert!(wallet.default_account.is_some());
//         assert_eq!(wallet.default_account.unwrap(), *account);
//     }

//     #[test]
//     fn remove_should_return_non_if_address_does_not_exist_in_wallet() {
//         let mut wallet = Wallet::new();
//         wallet.create().unwrap();

//         assert_none!(wallet.remove("invalid address"));
//     }

//     #[test]
//     fn remove_should_return_remove_account_from_wallet_if_address_exist() {
//         let mut wallet = Wallet::new();
//         let address = wallet.create().unwrap();

//         let removed_account = wallet.remove(&address).unwrap();
//         assert_eq!(removed_account.address, address);
//         assert_eq!(0, wallet.accounts.len());
//         assert_none!(wallet.default_account); // Because we deleted the only available account in the wallet.
//     }

//     #[test]
//     fn set_default_should_set_the_default_account_correctly() {
//         let mut wallet = Wallet::new();
//         let address1 = wallet.create().unwrap();
//         let address2 = wallet.create().unwrap();

//         assert_eq!(wallet.default_account().unwrap().address, address1);

//         wallet.set_default(&address2).unwrap();
//         assert_eq!(wallet.default_account().unwrap().address, address2);
//     }
// }
