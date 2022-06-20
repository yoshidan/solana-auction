use solana_program::msg;
use thiserror::Error;

use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum AuctionError {
    /// Invalid instruction
    #[error("Invalid Instruction")]
    InvalidInstruction,
    /// Not Rent Exempt
    #[error("Not Rent Exempt")]
    NotRentExempt,
    /// Expected Amount Mismatch
    #[error("Expected Amount Mismatch")]
    ExpectedAmountMismatch,
    /// Amount Overflow
    #[error("Amount Overflow")]
    AmountOverflow,
    /// Insufficient Bid Price
    #[error("Insufficient Bid Price")]
    InsufficientBidPrice,
    /// Already Bid
    #[error("Already Bid")]
    AlreadyBid,
    /// Auction Finished
    #[error("Inactive Auction")]
    InactiveAuction,
    /// Active Auction
    #[error("Active Auction")]
    ActiveAuction,
    /// No Bidder Found
    #[error("No Bidder Found")]
    NoBidderFound,
}

impl From<AuctionError> for ProgramError {
    fn from(e: AuctionError) -> Self {
        msg!("{:?}", e);
        ProgramError::Custom(e as u32)
    }
}
