//! # Roloi - Streams Contract
//! Create token streams from wallet to wallet.
//!
//! Notes
//! -----
//! - **IMPORTANT**: Time is measured in seconds.

#![cfg_attr(not(feature = "std"), no_std)]
mod errors;
pub mod stream;
pub mod utils;
use ink_lang as ink;

#[ink::contract]
pub mod streams_contract {
    use crate::errors::ContractError;
    use crate::stream::Stream;
    use crate::utils::{
        get_current_time_in_seconds, validate_and_generate_stream_end_date,
        validate_recipient_withdrawal_amount, validate_stream_creation_parameters,
    };
    use ink_lang::utils::initialize_contract;
    use ink_storage::traits::SpreadAllocate;
    use ink_storage::Mapping;

    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct StreamsContract {
        pub owner: AccountId,
        next_stream_id: u64,
        streams: Mapping<u64, Stream>,
    }

    impl StreamsContract {
        #[ink(constructor)]
        pub fn new() -> Self {
            initialize_contract(|contract: &mut Self| {
                contract.owner = Self::env().caller();
                contract.next_stream_id = 1;
                contract.streams = <Mapping<u64, Stream>>::default();
            })
        }

        /// Creates a token stream from the sender to the specified recipient setting the end date or the duration.
        ///
        /// Parameters:
        /// - `recipient`: The recipient wallet address of the stream.
        /// - `end_date`: The end date of the stream measured in seconds. If not specified, the stream will be created with the duration.
        /// - `duration`: The duration of the stream measured in seconds. If not specified, the stream will be created with the end date.
        /// - **Transaction funds:** The amount of funds to be transferred to the recipient through the stream.
        ///
        /// Validations:
        /// - The sender can't be the recipient.
        /// - The sender should send funds in the transaction.
        /// - The date parameters should be valid.
        ///   * `end_date` and `duration` cannot be both empty.
        ///   * `end_date` should be later than the current date.
        ///   * The stream duration should be greater than the **minimum duration**.
        ///
        /// Behavior:
        /// - A new stream with a unique ID will be stored in a mapping structure.
        /// - The next available ID will be increased by 1.
        ///
        /// Returns:
        /// - The created stream ID.
        ///
        /// Errors:
        /// - RecipientCannotBePayer
        /// - EmptyFunds
        /// - EndDateAndDurationAreEmpty
        /// - StreamEndDateShouldBeLater
        /// - StreamDurationShouldBeGreater
        ///
        /// NOTES
        /// -----
        /// - The current stream **minimum duration** is 5 minutes.
        /// - The stream starts immediately after it is created.
        #[ink(message, payable)]
        pub fn create_stream(
            &mut self,
            recipient: AccountId,
            end_date: Option<u64>,
            duration: Option<u64>,
        ) -> Result<u64, ContractError> {
            let start_date = get_current_time_in_seconds(&self);
            let caller = self.env().caller();
            let stream_funds = self.env().transferred_value();

            validate_stream_creation_parameters(caller, recipient, stream_funds)?;
            let end_date = validate_and_generate_stream_end_date(end_date, duration, start_date)?;

            let new_stream = Stream::new(caller, recipient, stream_funds, start_date, end_date);

            let new_stream_id = self.next_stream_id.clone().into();
            self.streams.insert(new_stream_id, &new_stream);
            self.next_stream_id += 1;

            Ok(new_stream_id)
        }

        /// Withdraws tokens from a stream. The recipient can specify the expected amount of tokens or withdraw all the available balance.
        ///
        /// Parameters:
        /// - `stream_id`: The stream ID.
        /// - `withdrawal_amount`: The amount of tokens to be withdrawn. If not specified, all the available balance will be withdrawn.
        ///
        /// Validations:
        /// - The stream should exist.
        /// - The sender should be the recipient of the stream.
        /// - The expected withdrawal amount should be greater or equal than the available balance.
        ///
        /// Behavior:
        /// - The stream available balance will be calculated based on the elapsed time.
        /// - The current stream balance will be reduced by the withdrawal amount.
        /// - The requested funds will be transfered to the sender.
        ///
        /// Returns:
        /// - The amount of tokens withdrawn.
        ///
        /// Errors:
        /// - Unauthorized
        /// - WithdrawalAmountShouldBeGreaterThanZero
        /// - StreamAvailableBalanceisZero
        /// - ExpectedWithdrawalAmountExceedsStreamAvailableBalance
        /// - WithdrawTransferFailed
        #[ink(message)]
        pub fn recipient_withdraw(
            &mut self,
            stream_id: u64,
            withdrawal_amount: Option<u128>,
        ) -> Result<u128, ContractError> {
            validate_recipient_withdrawal_amount(withdrawal_amount)?;
            let mut stream = self.get_stream_by_id(stream_id)?;
            stream.has_permission_to_withdraw(self.env().caller())?;

            let available_balance =
                stream.get_available_balance(get_current_time_in_seconds(&self))?;

            let amount_to_withdraw = withdrawal_amount.unwrap_or(available_balance);

            if amount_to_withdraw > available_balance {
                return Err(ContractError::ExpectedWithdrawalAmountExceedsStreamAvailableBalance);
            }

            stream.withdraw(amount_to_withdraw)?;
            self.streams.insert(stream_id, &stream);

            if self
                .env()
                .transfer(stream.recipient, amount_to_withdraw)
                .is_err()
            {
                return Err(ContractError::WithdrawTransferFailed);
            }

            Ok(amount_to_withdraw)
        }

        /// Returns a stream by its ID.
        ///
        /// Parameters:
        /// - `stream_id`: The expected stream ID.
        ///
        /// Validations:
        /// - The stream should exist.
        ///
        /// Returns:
        /// - The expected stream.
        ///
        /// Errors:
        /// - StreamDoesNotExist
        #[ink(message)]
        pub fn get_stream_by_id(&self, stream_id: u64) -> Result<Stream, ContractError> {
            match self.streams.get(&stream_id) {
                Some(stream) => Ok(stream),
                None => Err(ContractError::StreamDoesNotExist),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_lang as ink;

        fn get_contract_id() -> AccountId {
            ink_env::test::callee::<ink_env::DefaultEnvironment>()
        }

        fn get_default_accounts() -> ink_env::test::DefaultAccounts<ink_env::DefaultEnvironment> {
            ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
        }

        fn set_sender(sender: AccountId) {
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(sender);
        }

        fn set_balance(account_id: AccountId, balance: u128) {
            ink_env::test::set_account_balance::<ink_env::DefaultEnvironment>(account_id, balance)
        }

        fn advance_block() {
            let _ = ink_env::test::advance_block::<ink_env::DefaultEnvironment>();
        }

        fn set_value_transferred(amount: u128) {
            ink_env::test::set_value_transferred::<ink_env::DefaultEnvironment>(amount);
        }

        fn init() -> (
            StreamsContract,
            ink_env::test::DefaultAccounts<ink_env::DefaultEnvironment>,
        ) {
            (StreamsContract::new(), get_default_accounts())
        }

        #[ink::test]
        fn create_stream_with_duration_works() {
            // Arrange
            let (mut contract, accounts) = init();
            let funds = 1;
            let sender = accounts.alice;
            let recipient = accounts.bob;
            let duration = 10000;
            set_sender(sender);
            set_value_transferred(funds);

            // Act
            let current_time = get_current_time_in_seconds(&contract);
            let stream_id = contract
                .create_stream(recipient, None, Some(duration))
                .unwrap();

            // Assert
            assert_eq!(stream_id, contract.next_stream_id - 1);
            let stream = contract.get_stream_by_id(stream_id).unwrap();
            assert_eq!(stream.payer, sender);
            assert_eq!(stream.recipient, recipient);
            assert_eq!(stream.original_balance, funds);
            assert_eq!(stream.current_balance, funds);
            assert_eq!(stream.start_date, current_time);
            assert_eq!(
                stream.end_date,
                get_current_time_in_seconds(&contract) + duration
            );
        }

        #[ink::test]
        fn create_stream_with_end_date_works() {
            // Arrange
            let (mut contract, accounts) = init();
            let funds = 1;
            let sender = accounts.alice;
            let recipient = accounts.bob;
            let end_date = 1910126705;
            set_sender(sender);
            set_value_transferred(funds);

            // Act
            let current_time = get_current_time_in_seconds(&contract);
            let stream_id = contract
                .create_stream(recipient, Some(end_date), None)
                .unwrap();

            // Assert
            assert_eq!(stream_id, contract.next_stream_id - 1);
            let stream = contract.get_stream_by_id(stream_id).unwrap();
            assert_eq!(stream.payer, sender);
            assert_eq!(stream.recipient, recipient);
            assert_eq!(stream.original_balance, funds);
            assert_eq!(stream.current_balance, funds);
            assert_eq!(stream.start_date, current_time);
            assert_eq!(stream.end_date, end_date);
        }

        #[ink::test]
        fn create_stream_without_funds_fails() {
            // Arrange
            let (mut contract, accounts) = init();
            let sender = accounts.alice;
            let recipient = accounts.bob;
            set_sender(sender);

            // Act
            let result = contract.create_stream(recipient, None, None);

            // Assert
            assert_eq!(result, Err(ContractError::EmptyFunds));
        }

        #[ink::test]
        fn create_stream_without_end_date_and_duration_fails() {
            // Arrange
            let (mut contract, accounts) = init();
            let funds = 1;
            let sender = accounts.alice;
            let recipient = accounts.bob;
            set_sender(sender);
            set_value_transferred(funds);

            // Act
            let result = contract.create_stream(recipient, None, None);

            // Assert
            assert_eq!(result, Err(ContractError::EndDateAndDurationAreEmpty));
        }

        #[ink::test]
        fn create_stream_with_same_payer_and_recipient_fails() {
            // Arrange
            let (mut contract, accounts) = init();
            let funds = 1;
            let sender = accounts.bob;
            let recipient = accounts.bob;
            set_sender(sender);
            set_value_transferred(funds);

            // Act
            let result = contract.create_stream(recipient, None, None);

            // Assert
            assert_eq!(result, Err(ContractError::RecipientCannotBePayer));
        }

        #[ink::test]
        fn create_stream_with_short_duration_fails() {
            // Arrange
            let (mut contract, accounts) = init();
            let funds = 1;
            let sender = accounts.alice;
            let recipient = accounts.bob;
            let duration = 100;
            set_sender(sender);
            set_value_transferred(funds);

            // Act
            let result = contract.create_stream(recipient, None, Some(duration));

            // Assert
            assert_eq!(result, Err(ContractError::StreamDurationShouldBeGreater));
        }

        #[ink::test]
        fn create_stream_with_short_end_date_fails() {
            // Arrange
            let (mut contract, accounts) = init();
            let funds = 1;
            let sender = accounts.alice;
            let recipient = accounts.bob;
            let end_date = 100;
            set_sender(sender);
            set_value_transferred(funds);

            // Act
            let result = contract.create_stream(recipient, Some(end_date), None);

            // Assert
            assert_eq!(result, Err(ContractError::StreamEndDateShouldBeLater));
        }

        #[ink::test]
        fn recipient_withdraw_all_works() {
            // Arrange
            let (mut contract, accounts) = init();
            let funds = 3000000000;
            let recipient = accounts.alice;
            contract.streams.insert(
                1,
                &Stream::new(accounts.bob, recipient, funds.clone().into(), 0, 300),
            );
            set_balance(get_contract_id(), funds);
            set_sender(recipient);

            // Stream finished
            for _ in 0..50000 {
                advance_block();
            }

            // Act
            let amount_withdrawn = contract.recipient_withdraw(1, None).unwrap();

            // Assert
            assert_eq!(amount_withdrawn, funds);
        }

        #[ink::test]
        fn recipient_withdraw_specific_amount_works() {
            // Arrange
            let (mut contract, accounts) = init();
            let funds = 3000000000;
            let expected_withdrawal_amount = 1500000000;
            let recipient = accounts.alice;
            contract.streams.insert(
                1,
                &Stream::new(accounts.bob, recipient, funds.clone().into(), 0, 300),
            );
            set_balance(get_contract_id(), funds);
            set_sender(recipient);

            // Stream finished
            for _ in 0..50000 {
                advance_block();
            }

            // Act
            let amount_withdrawn = contract
                .recipient_withdraw(1, Some(expected_withdrawal_amount))
                .unwrap();

            // Assert
            assert_eq!(amount_withdrawn, expected_withdrawal_amount);
        }

        #[ink::test]
        fn recipient_withdraw_with_unauthorized_wallet_fails() {
            // Arrange
            let (mut contract, accounts) = init();
            let funds = 3000000000;
            let expected_withdrawal_amount = 1500000000;
            let recipient = accounts.alice;
            contract.streams.insert(
                1,
                &Stream::new(accounts.bob, recipient, funds.clone().into(), 0, 300),
            );
            set_balance(get_contract_id(), funds);
            set_sender(accounts.charlie);

            // Stream finished
            for _ in 0..50000 {
                advance_block();
            }

            // Act
            let result = contract.recipient_withdraw(1, Some(expected_withdrawal_amount));

            // Assert
            assert_eq!(result, Err(ContractError::Unauthorized));
        }

        #[ink::test]
        fn recipient_withdraw_from_non_existent_stream_fails() {
            // Arrange
            let (mut contract, accounts) = init();
            let funds = 3000000000;
            let expected_withdrawal_amount = 1500000000;
            let recipient = accounts.alice;
            contract.streams.insert(
                1,
                &Stream::new(accounts.bob, recipient, funds.clone().into(), 0, 300),
            );
            set_balance(get_contract_id(), funds);
            set_sender(recipient);

            // Stream finished
            for _ in 0..50000 {
                advance_block();
            }

            // Act
            let result = contract.recipient_withdraw(999, Some(expected_withdrawal_amount));

            // Assert
            assert_eq!(result, Err(ContractError::StreamDoesNotExist));
        }

        #[ink::test]
        fn recipient_withdraw_with_expected_amount_greater_than_available_balance_fails() {
            // Arrange
            let (mut contract, accounts) = init();
            let funds = 3000000000;
            let expected_withdrawal_amount = 3000000000;
            let recipient = accounts.alice;
            contract.streams.insert(
                1,
                &Stream::new(accounts.bob, recipient, funds.clone().into(), 0, 300),
            );
            set_balance(get_contract_id(), funds);
            set_sender(recipient);

            // Stream finished
            for _ in 0..25000 {
                advance_block();
            }

            // Act
            let result = contract.recipient_withdraw(1, Some(expected_withdrawal_amount));

            // Assert
            assert_eq!(
                result,
                Err(ContractError::ExpectedWithdrawalAmountExceedsStreamAvailableBalance)
            );
        }

        #[ink::test]
        fn recipient_withdraw_with_expected_amount_equal_to_zero_fails() {
            // Arrange
            let (mut contract, accounts) = init();
            let funds = 3000000000;
            let expected_withdrawal_amount = 0;
            let recipient = accounts.alice;
            contract.streams.insert(
                1,
                &Stream::new(accounts.bob, recipient, funds.clone().into(), 0, 300),
            );
            set_balance(get_contract_id(), funds);
            set_sender(recipient);

            // Stream finished
            for _ in 0..25000 {
                advance_block();
            }

            // Act
            let result = contract.recipient_withdraw(1, Some(expected_withdrawal_amount));

            // Assert
            assert_eq!(
                result,
                Err(ContractError::WithdrawalAmountShouldBeGreaterThanZero)
            );
        }

        #[ink::test]
        fn recipient_withdraw_with_available_balance_equal_to_zero_fails() {
            // Arrange
            let (mut contract, accounts) = init();
            let funds = 3000000000;
            let expected_withdrawal_amount = 100;
            let recipient = accounts.alice;
            contract.streams.insert(
                1,
                &Stream::new(accounts.bob, recipient, funds.clone().into(), 0, 300),
            );
            set_balance(get_contract_id(), funds);
            set_sender(recipient);

            // Act
            let result = contract.recipient_withdraw(1, Some(expected_withdrawal_amount));

            // Assert
            assert_eq!(result, Err(ContractError::StreamAvailableBalanceIsZero));
        }

        #[ink::test]
        fn get_stream_by_id_works() {
            // Arrange
            let (mut contract, accounts) = init();
            let funds = 3000000000;
            let payer = accounts.alice;
            let recipient = accounts.bob;
            let start_date = 0;
            let end_date = 300;
            contract
                .streams
                .insert(1, &Stream::new(payer, recipient, funds, 0, 300));

            // Act
            let stream = contract.get_stream_by_id(1).unwrap();

            // Assert
            assert_eq!(stream.payer, payer);
            assert_eq!(stream.recipient, recipient);
            assert_eq!(stream.original_balance, funds);
            assert_eq!(stream.current_balance, funds);
            assert_eq!(stream.start_date, start_date);
            assert_eq!(stream.end_date, end_date);
        }

        #[ink::test]
        fn get_stream_by_id_with_invalid_parameters_fails() {
            // Arrange
            let (contract, _) = init();

            // Act
            let result = contract.get_stream_by_id(1);

            // Assert
            assert_eq!(result, Err(ContractError::StreamDoesNotExist));
        }
    }
}
