use ink_env::AccountId;
use ink_storage::traits::{PackedLayout, SpreadAllocate, SpreadLayout, StorageLayout};

use crate::errors::ContractError;

/// Minimum duration that a stream can have.
pub const STREAM_MINIMUM_DURATION: u64 = 300;

/// Struct for storing streams
#[derive(
    PartialEq,
    Debug,
    Eq,
    Clone,
    Copy,
    scale::Encode,
    scale::Decode,
    SpreadLayout,
    PackedLayout,
    SpreadAllocate,
)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
pub struct Stream {
    /// AccountId of the payer.
    pub payer: AccountId,
    /// AccountId of the recipient.
    pub recipient: AccountId,
    /// Initial balance of the stream.
    pub original_balance: u128,
    /// Current balance of the stream.
    pub current_balance: u128,
    /// Date when the stream started. Measured in seconds.
    pub start_date: u64,
    /// Date when the stream will end. Measured in seconds.
    pub end_date: u64,
}

impl Stream {
    pub fn new(
        payer: AccountId,
        recipient: AccountId,
        stream_funds: u128,
        start_date: u64,
        end_date: u64,
    ) -> Stream {
        Stream {
            payer,
            recipient,
            original_balance: stream_funds,
            current_balance: stream_funds,
            start_date,
            end_date,
        }
    }

    /// Reduces the current stream balance by the specified amount.
    ///
    /// Parameters:
    /// - `amount`: Amount to reduce the stream balance by.
    ///
    /// Validations:
    /// - `amount` should be smaller than `stream.current_balance`.
    ///
    /// Errors:
    /// - ExpectedWithdrawalAmountExceedsStreamAvailableBalance
    pub fn withdraw(&mut self, amount: u128) -> Result<(), ContractError> {
        if self.current_balance < amount {
            return Err(ContractError::ExpectedWithdrawalAmountExceedsStreamAvailableBalance);
        }

        self.current_balance -= amount;

        Ok(())
    }

    /// Calculates the stream availabe balance based on the elapsed time.
    ///
    /// Parameters:
    /// - `current_time`: Current time in seconds.
    ///
    /// Behavior:
    /// - The stream available balance will be calculated based on the elapsed time and the withdrawn balance.
    ///
    /// Returns:
    /// - The stream available balance.
    pub fn get_available_balance(&self, current_time: u64) -> Result<u128, ContractError> {
        let balance_withdrawn: u128 = self.original_balance - self.current_balance;

        let available_balance = if self.is_finished(current_time) {
            self.original_balance - balance_withdrawn
        } else {
            let elapsed_time = current_time - self.start_date;

            let unlocked_balance =
                self.original_balance * (elapsed_time as u128) / (self.total_duration() as u128);

            unlocked_balance - balance_withdrawn
        };

        if available_balance == 0 {
            return Err(ContractError::StreamAvailableBalanceIsZero);
        }

        Ok(available_balance)
    }

    /// Check if the caller has permission to withdraw from the stream.
    ///
    /// Parameters:
    /// - `caller`: AccountId of the caller.
    ///
    /// Validations:
    /// - `caller` should be the stream recipient.
    ///
    /// Errors:
    /// - Unauthorized
    pub fn has_permission_to_withdraw(&self, caller: AccountId) -> Result<(), ContractError> {
        if caller != self.recipient {
            return Err(ContractError::Unauthorized);
        }

        Ok(())
    }

    /// Check if the stream is finished.
    ///
    /// Parameters:
    /// - `current_time`: Current time in seconds.
    ///
    /// Returns:
    /// - `true` if the stream has finished, `false` otherwise.
    fn is_finished(&self, current_time: u64) -> bool {
        current_time > self.end_date
    }

    /// Calculates the total duration of the stream.
    ///
    /// Returns:
    /// - The difference between `stream.end_date` and `stream.start_date`.
    fn total_duration(&self) -> u64 {
        self.end_date - self.start_date
    }
}
