#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod payable {
    use ink::{
        prelude::vec::Vec
    };

    /// Represents the company and the company's transaction
    /// fee for each checkout
    #[ink(storage)]
    pub struct Payable {
        /// The company's address
        company_id: AccountId,
        /// The fee of the transaction as a percentage
        /// of the overall payment
        /// In calculations,the result will be divided
        /// by 100 to get the actual fee.
        fee: u128,
    }

    #[derive(Clone, Debug, scale::Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]    pub struct Benefactor {
        id: AccountId,
        amount: u128
    }

    impl Payable {
        /// Initialize the payable struct with the
        /// expected values.
        #[ink(constructor)]
        pub fn new(
            company_id: AccountId,
            fee: u128,
        ) -> Self {
            Self { company_id, fee }
        }

        #[ink(message, payable)]
        pub fn distribute(
            &mut self, 
            benefactors: Vec<Benefactor>,
        ) {
            
            let value = self.env().transferred_value();
            let fee = (value * self.fee) / 100;

            //  Assert that transferred value equals to
            //  the funds required for settlement
            let mut actual_amount = fee;
            for benefactor in &benefactors {
                actual_amount += benefactor.amount;
            }
            assert_eq!(
                actual_amount,
                value,
                "Incorrect settlement figures provided. The value
                transferred is not equal to the required amount"
            );

            //  If conditions are met, settle
            self.env()
                .transfer(self.company_id, fee)
                .expect("Failed to transfer fee to company");

            for benefactor in benefactors {
                self.env()
                .transfer(benefactor.id, benefactor.amount)
                .expect("Failed to transfer value to the {benefactor.id}");    
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use ink::{
            env::{
                DefaultEnvironment, 
                test::{
                    default_accounts,
                    get_account_balance, 
                    DefaultAccounts, 
                    set_caller, set_value_transferred,

                }, pay_with_call
            }, primitives::AccountId,
        };

        use super::*;

        #[ink::test]
        pub fn it_sends_funds() {
            let accounts = get_default_accounts();
            let company = accounts.frank;
            let total = 100;

            let expected_company_fee: u128 = 15;
            let receiver = Benefactor{
                id: accounts.django,
                amount: (total-15) % 2
            };
            let receiver2 = Benefactor{
                id: accounts.eve,
                amount: (total-15) - receiver.amount
            };
            let benefactors = vec![receiver, receiver2];
        
            let mut contract = Payable::new(
                company,
                expected_company_fee
            );

            set_sender(accounts.charlie);
            value_transfered(total);

            pay_with_call!(
                contract.distribute(benefactors.clone()),
                total
            );

            //  Assert
            let actual_company_balance = get_balance(company);
            assert_eq!(
                expected_company_fee,
                actual_company_balance
            );

            for benefactor in benefactors {
                let actual_receiver_balance = get_balance(benefactor.id);
                assert_eq!(
                    benefactor.amount,
                    actual_receiver_balance
                )
            }
        }

        fn set_sender(sender: AccountId) {
            set_caller::<DefaultEnvironment>(sender)
        }

        fn value_transfered(amount: u128) {
            set_value_transferred::<DefaultEnvironment>(amount)
        }

        fn get_default_accounts() -> DefaultAccounts<DefaultEnvironment> {
            default_accounts::<DefaultEnvironment>()
        }

        fn get_balance(account_id: AccountId) -> Balance {
            get_account_balance::<DefaultEnvironment>(account_id)
                .expect("Cannot get account balance")
        }
    }

    

}

