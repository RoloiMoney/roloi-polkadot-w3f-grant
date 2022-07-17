#[derive(PartialEq, Debug, Eq, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum ContractError {
    ValidationError,
    Unauthorized,
    GeneralError,
    StreamDoesNotExist,
    EndDateAndDurationAreEmpty,
    StreamDurationShouldBeGreater,
    StreamEndDateShouldBeLater,
    StreamAvailableBalanceIsZero,
    EmptyFunds,
    ExpectedWithdrawalAmountExceedsStreamAvailableBalance,
    RecipientCannotBePayer,
    WithdrawTransferFailed,
    WithdrawalAmountShouldBeGreaterThanZero,
    Unexpected,
}
