use solana_program::{
    program_error::ProgramError,
    program_pack::{Pack, Sealed},
    pubkey::Pubkey,
};

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_program::program_pack::IsInitialized;

pub struct Auction {
    pub is_initialized: bool,
    /// Wallet pubkey of exhibitor
    pub exhibitor_pubkey: Pubkey,
    /// Exhibiting NFT pubkey of exhibitor in escrow
    pub exhibiting_nft_temp_pubkey: Pubkey,
    /// FT pubkey of exhibitor to get token into
    pub exhibitor_ft_receiving_pubkey: Pubkey,
    /// Current price
    pub price: u64,
    /// Auction end time
    pub end_at: i64,
    /// Wallet pubkey of highest bidder
    pub highest_bidder_pubkey: Pubkey,
    /// FT pubkey of highest bidder in escrow
    pub highest_bidder_ft_temp_pubkey: Pubkey,
    /// FT pubkey of highest bidder to return to when the auction cancel or other bidder become the highest bidder.
    pub highest_bidder_ft_returning_pubkey: Pubkey,
}

impl Sealed for Auction {}

impl IsInitialized for Auction {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for Auction {
    const LEN: usize = 209;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, Auction::LEN];
        let (
            is_initialized_dst,
            exhibitor_pubkey_dst,
            exhibiting_nft_temp_pubkey_dst,
            exhibitor_ft_receiving_pubkey_dst,
            price_dst,
            end_at_dst,
            highest_bidder_pubkey_dst,
            highest_bidder_ft_temp_pubkey_dst,
            highest_bidder_ft_returning_pubkey_dst,
        ) = mut_array_refs![dst, 1, 32, 32, 32, 8, 8, 32, 32, 32];

        let Auction {
            is_initialized,
            exhibitor_pubkey,
            exhibiting_nft_temp_pubkey,
            exhibitor_ft_receiving_pubkey,
            price,
            end_at,
            highest_bidder_pubkey,
            highest_bidder_ft_temp_pubkey,
            highest_bidder_ft_returning_pubkey,
        } = self;

        is_initialized_dst[0] = *is_initialized as u8;
        exhibitor_pubkey_dst.copy_from_slice(exhibitor_pubkey.as_ref());
        exhibiting_nft_temp_pubkey_dst.copy_from_slice(exhibiting_nft_temp_pubkey.as_ref());
        exhibitor_ft_receiving_pubkey_dst.copy_from_slice(exhibitor_ft_receiving_pubkey.as_ref());
        *price_dst = price.to_le_bytes();
        *end_at_dst = end_at.to_le_bytes();
        highest_bidder_pubkey_dst.copy_from_slice(highest_bidder_pubkey.as_ref());
        highest_bidder_ft_temp_pubkey_dst.copy_from_slice(highest_bidder_ft_temp_pubkey.as_ref());
        highest_bidder_ft_returning_pubkey_dst
            .copy_from_slice(highest_bidder_ft_returning_pubkey.as_ref());
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, Auction::LEN];
        let (
            is_initialized,
            exhibitor_pubkey,
            exhibiting_nft_temp_pubkey,
            exhibitor_ft_receiving_pubkey,
            price,
            end_at,
            highest_bidder_pubkey,
            highest_bidder_ft_temp_pubkey,
            highest_bidder_ft_returning_pubkey,
        ) = array_refs![src, 1, 32, 32, 32, 8, 8, 32, 32, 32];
        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };

        Ok(Auction {
            is_initialized,
            exhibitor_pubkey: Pubkey::new_from_array(*exhibitor_pubkey),
            exhibiting_nft_temp_pubkey: Pubkey::new_from_array(*exhibiting_nft_temp_pubkey),
            exhibitor_ft_receiving_pubkey: Pubkey::new_from_array(*exhibitor_ft_receiving_pubkey),
            price: u64::from_le_bytes(*price),
            end_at: i64::from_le_bytes(*end_at),
            highest_bidder_pubkey: Pubkey::new_from_array(*highest_bidder_pubkey),
            highest_bidder_ft_temp_pubkey: Pubkey::new_from_array(*highest_bidder_ft_temp_pubkey),
            highest_bidder_ft_returning_pubkey: Pubkey::new_from_array(
                *highest_bidder_ft_returning_pubkey,
            ),
        })
    }
}
