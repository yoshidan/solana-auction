use crate::error::AuctionError::InvalidInstruction;
use solana_program::program_error::ProgramError;

pub enum AuctionInstruction {
    /// Starts the auction by creating and populating an escrow account and transferring ownership of the given temp NFT account to the PDA
    ///
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person starting the auction
    /// 1. `[writable]` Temporary NFT account that should be created prior to this instruction and owned by the exhibitor
    /// 2. `[]` The exhibitor's NFT account for the token they will receive should the trade go through
    /// 3. `[writable]` The escrow account, it will hold all necessary info about the auction.
    /// 4. `[]` The rent sysvar
    /// 4. `[]` The clock sysvar
    /// 5. `[]` The token program
    Exhibit {
        /// Initial NFT price
        initial_price: u64,
        /// Auction duration
        seconds: u64,
    },

    /// Bid on the auction and transfer ownership of the given temp FT account to the PDA
    ///
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]`  The account of the person bidding on the auction
    /// 1. `[writable]` The current highest bidder's temporary FT account
    /// 2. `[writable]` The current highest bidder's FT account to get back to when the other person become the highest bidder
    /// 3. `[writable]` The bidder's temporary FT account for depositing FT in escrow
    /// 4. `[writable]` The bidder's FT account to get back to when the other person become the highest bidder
    /// 5. `[writable]` The escrow account, it will hold all necessary info about the auction.
    /// 6. `[]` The clock sysvar
    /// 7. `[]` The token program
    /// 8. `[]` The PDA account
    Bid {
        /// Bidding price
        price: u64,
    },

    /// Cancels a auction
    /// Auction can't be cancelled if any bidder found
    ///
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person started the auction
    /// 1. `[writable]` The PDA's temporary NFT account
    /// 2. `[writable]` The exhibitor's NFT account to get the token back to
    /// 3. `[writable]` The escrow account holding the escrow info
    /// 4. `[]` The token program
    /// 5. `[]` The PDA account
    Cancel {},

    /// Closes a auction
    /// Only the successful bidder can close the auction
    ///
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the successful bidder
    /// 1. `[]` The account ot the person started the auction to close the escrow
    /// 2. `[writable]` The temporary NFT account to send to successful bidder
    /// 3. `[writable]` The FT account to sent FT to the exhibitor'
    /// 4. `[writable]` The temporary FT account that holds the successful bidder's FT
    /// 5. `[writable]` The NFT account that will receive NFT
    /// 6. `[writable]` The escrow account holding the escrow info
    /// 7. `[]` The clock sysvar
    /// 8. `[]` The token program
    /// 9. `[]` The PDA account
    Close {},
}

impl AuctionInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (instruction_type, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match instruction_type {
            0 => Self::Exhibit {
                initial_price: Self::unpack64(rest, 0)?,
                seconds: Self::unpack64(rest, 8)?,
            },
            1 => Self::Bid {
                price: Self::unpack64(rest, 0)?,
            },
            2 => Self::Cancel {},
            3 => Self::Close {},
            _ => return Err(InvalidInstruction.into()),
        })
    }

    fn unpack64(input: &[u8], start: usize) -> Result<u64, ProgramError> {
        let v = input
            .get(start..start + 8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InvalidInstruction)?;
        Ok(v)
    }
}
