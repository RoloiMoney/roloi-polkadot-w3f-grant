use ink_env::AccountId;
use ink_lang::codegen::Env;

use crate::{
    errors::ContractError, stream::STREAM_MINIMUM_DURATION, streams_contract::StreamsContract,
};

/// Validates and generate the stream end date based on the date parameters of the `create_stream` message.
///
/// Parameters:
/// - `end_date`: Date measured in seconds received as parameter in the `create_stream` message.
/// - `duration`: Duration measured in seconds received as parameter in the `create_stream` message.
/// - `start_date`: Stream creation date measured in seconds.
///
/// Validations:
/// - `end_date` and `duration` cannot be both empty.
/// - `end_date` should be greater than the current date.
/// - The stream duration should be greater than the **minimum duration**.
///
/// Returns:
/// - The stream end date in seconds.
///
/// Errors:
/// - EndDateAndDurationAreEmpty
/// - StreamEndDateShouldBeLater
/// - StreamDurationShouldBeGreater
///
/// NOTES
/// -----
/// - The current stream **minimum duration** is 5 minutes.
pub fn validate_and_generate_stream_end_date(
    end_date: Option<u64>,
    duration: Option<u64>,
    start_date: u64,
) -> Result<u64, ContractError> {
    if end_date == None && duration == None {
        return Err(ContractError::EndDateAndDurationAreEmpty);
    }

    if end_date != None {
        let end_date = end_date.unwrap();
        validate_stream_end_date(start_date, end_date)?;
        return Ok(end_date);
    }

    if duration != None {
        let duration = duration.unwrap();
        validate_stream_duration(duration)?;
        return Ok(start_date + duration);
    };

    Err(ContractError::Unexpected)
}

/// Validates the stream end date based on the `end_date` parameter of the `create_stream` message.
///
/// Parameters:
/// - `start_date`: Stream creation date measured in seconds.
/// - `end_date`: Date measured in seconds received as parameter in the `create_stream` message.
///
/// Validations:
/// - `end_date` should be after the minimum end date according to the `start_date`.
///
/// Errors:
/// - StreamEndDateShouldBeLater
///
/// NOTES
/// -----
/// - The current stream **minimum duration** is 5 minutes.
fn validate_stream_end_date(start_date: u64, end_date: u64) -> Result<(), ContractError> {
    if end_date < start_date + STREAM_MINIMUM_DURATION {
        return Err(ContractError::StreamEndDateShouldBeLater);
    }

    Ok(())
}

/// Validates the stream duration based on the `duration` parameter of the `create_stream` message.
///
/// Parameters:
/// - `duration`: Duration measured in seconds received as parameter in the `create_stream` message.
///
/// Validations:
/// - `duration` should be greater than the stream minimum duration.
///
/// Errors:
/// - StreamDurationShouldBeGreater
///
/// NOTES
/// -----
/// - The current stream **minimum duration** is 5 minutes.
fn validate_stream_duration(duration: u64) -> Result<(), ContractError> {
    if duration < STREAM_MINIMUM_DURATION {
        return Err(ContractError::StreamDurationShouldBeGreater);
    }

    Ok(())
}

/// Validates the `create_stream` message parameters.
///
/// Parameters:
/// - `payer`: Sender of the `create_stream` message.
/// - `recipient`: Recipient AccountId received as parameter in the `create_stream` message.
/// - `funds`: Funds received in the `create_stream` message.
///
/// Validations:
/// - `payer` should be different than `recipient`.
/// - `funds` should be greater than 0.
///
/// Errors:
/// - RecipientCannotBePayer
/// - EmptyFunds
pub fn validate_stream_creation_parameters(
    payer: AccountId,
    recipient: AccountId,
    funds: u128,
) -> Result<(), ContractError> {
    if payer == recipient {
        return Err(ContractError::RecipientCannotBePayer);
    }

    if funds <= 0 {
        return Err(ContractError::EmptyFunds);
    }

    Ok(())
}

/// Validates if the `withdrawal_amount` parameter of the `recipient_withdraw` message is greater than zero.
///
/// Parameters:
/// - `withdrawal_amount`: The expected amount of tokens to be withdrawn received as parameter in the `recipient_withdraw` message. Can be empty.
///
/// Validations:
/// - If `withdrawal_amount` has value, should be greater than zero.
///
/// Errors:
/// - WithdrawalAmountShouldBeGreaterThanZero
pub fn validate_recipient_withdrawal_amount(
    withdrawal_amount: Option<u128>,
) -> Result<(), ContractError> {
    if withdrawal_amount != None && withdrawal_amount.unwrap() == 0 {
        return Err(ContractError::WithdrawalAmountShouldBeGreaterThanZero);
    }

    Ok(())
}

/// Get the time of the current block.
pub fn get_current_time_in_seconds(contract: &StreamsContract) -> u64 {
    contract.env().block_timestamp() / 1000
}
