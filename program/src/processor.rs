use crate::error::AuctionError;
use crate::instruction::AuctionInstruction;
use crate::state::Auction;
use solana_program::account_info::{next_account_info, AccountInfo};
use solana_program::clock::Clock;
use solana_program::entrypoint::ProgramResult;
use solana_program::msg;
use solana_program::program::{invoke, invoke_signed};
use solana_program::program_error::ProgramError;
use solana_program::program_pack::{IsInitialized, Pack};
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;
use spl_token::state::Account as TokenAccount;
use std::ops::Add;

pub struct Processor;

impl Processor {
    pub fn process(
        // program_id is the public key for this program
        // This time, The program_id is '5bkMHSLS77FtUTEYTcmKw6Gouf6HVV1dAYDwTgnMJP2y'
        program_id: &Pubkey,
        // accounts contains the account information that corresponds to the account's public key given by the client.
        accounts: &[AccountInfo],
        // instruction_data is the data passed by the client
        instruction_data: &[u8],
    ) -> ProgramResult {
        // Decode byte array to AuctionInstruction struct.
        let instruction = AuctionInstruction::unpack(instruction_data)?;
        match instruction {
            AuctionInstruction::Exhibit {
                initial_price,
                seconds,
            } => {
                msg!("Instruction: Exhibit");
                Self::process_exhibit(accounts, initial_price, seconds, program_id)
            }
            AuctionInstruction::Bid { price } => {
                msg!("Instruction: Bid");
                Self::process_bid(accounts, price, program_id)
            }
            AuctionInstruction::Cancel {} => {
                msg!("Instruction: Cancel");
                Self::process_cancel(accounts, program_id)
            }
            AuctionInstruction::Close {} => {
                msg!("Instruction: Close ");
                Self::process_close(accounts, program_id)
            }
        }
    }

    fn process_exhibit(
        accounts: &[AccountInfo],
        initial_price: u64,
        auction_duration_sec: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let exhibitor_account = next_account_info(account_info_iter)?;

        // Make sure the seller is signed with the client's private key
        // In the blockchain, the public key is known by solscan etc., so it is necessary to confirm that the call is correctly signed to prevent spoofing.
        if !exhibitor_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let exhibitor_nft_account = next_account_info(account_info_iter)?;
        let exhibitor_nft_temp_account = next_account_info(account_info_iter)?;
        let exhibitor_ft_receiving_account = next_account_info(account_info_iter)?;

        // 'escrow_account' here is the organizer of the auction.
        // Generally, 'Escrow' is an account for exchanging tokens to prevent them from being taken away.
        // When exchanging tokens between two parties, the one who sent it first without the intervention of 'Escrow' runs the risk of being taken away.
        // - reference: https://paulx.dev/blog/2021/01/14/programming-on-solana-an-introduction/
        // This time, we will prevent fraud by saving information such as listing NFT, bid amount, deadline, etc. in escrow_account.
        let escrow_account = next_account_info(account_info_iter)?;
        let sys_var_rent_account = next_account_info(account_info_iter)?;

        // In Solana, rent will be charged according to the amount of data stored in the blockchain.
        // If payment is made for each epoch and the balance becomes zero, the data in the account will be lost.
        // By keeping the rent equivalent to 2 years from the beginning, you will be exempt from paying the rent.
        // 'is_exempt' checks if there is a minimum balance that will be exempt from payment.
        // This check is required as it is not good for data to disappear during the auction period.
        // By the way, the amount of data held by this auction system is 209 bytes. Therefore, the required rent will be 0.00234552 SOL.
        let rent = &Rent::from_account_info(sys_var_rent_account)?;
        if !rent.is_exempt(escrow_account.lamports(), escrow_account.data_len()) {
            return Err(AuctionError::NotRentExempt.into());
        }

        // I am extracting auction data from the blockchain.
        // Since there is nothing at first, it contains an empty value obtained from fixed-length data.
        let mut auction_info = Auction::unpack_unchecked(&escrow_account.try_borrow_data()?)?;
        if auction_info.is_initialized() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        let sys_var_clock_account = next_account_info(account_info_iter)?;
        let clock = &Clock::from_account_info(sys_var_clock_account)?;

        // Create auction data
        auction_info.is_initialized = true;
        auction_info.exhibitor_pubkey = *exhibitor_account.key;
        auction_info.exhibiting_nft_temp_pubkey = *exhibitor_nft_temp_account.key;
        auction_info.exhibitor_ft_receiving_pubkey = *exhibitor_ft_receiving_account.key;
        auction_info.price = initial_price;
        auction_info.end_at = clock.unix_timestamp.add(auction_duration_sec as i64);
        Auction::pack(auction_info, &mut escrow_account.try_borrow_mut_data()?)?;

        // 'PDA' is an account specific to this program
        //   - reference: https://solanacookbook.com/core-concepts/pdas.html#facts
        // Here, the word 'escrow' is fixedly specified for seed, so no matter who calls it from anywhere, the PDA will be the same.
        // You can sign by using bump_seed in the Solana Program.
        // In this auction system, it is used to move the seller's NFT held by 'Escrow' to the winning bidder, and to move it with a PDA signature without the seller's signature.
        let (pda, _bump_seed) = Pubkey::find_program_address(&[b"escrow"], program_id);
        let token_program = next_account_info(account_info_iter)?;

        // Transfer NFT to escrow
        let exhibit_ix = spl_token::instruction::transfer(
            token_program.key,
            exhibitor_nft_account.key,
            exhibitor_nft_temp_account.key,
            exhibitor_account.key,
            &[], // authority_pubkey is default signer when the signer_pubkeys is empty.
            1,
        )?;
        msg!("Calling the token program to transfer NFT to PDA...");
        invoke(
            &exhibit_ix,
            &[
                exhibitor_nft_account.clone(),
                exhibitor_nft_temp_account.clone(),
                exhibitor_account.clone(),
                token_program.clone(),
            ],
        )?;

        let owner_change_ix = spl_token::instruction::set_authority(
            token_program.key,
            exhibitor_nft_temp_account.key,
            Some(&pda),
            spl_token::instruction::AuthorityType::AccountOwner,
            exhibitor_account.key,
            &[], // owner_pubkey is default signer when the signer_pubkeys is empty.
        )?;
        msg!("Calling the token program to transfer token account ownership...");
        invoke(
            &owner_change_ix,
            &[
                exhibitor_nft_temp_account.clone(),
                exhibitor_account.clone(),
                token_program.clone(),
            ],
        )?;
        Ok(())
    }

    fn process_bid(accounts: &[AccountInfo], price: u64, program_id: &Pubkey) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let bidder_account = next_account_info(account_info_iter)?;

        // Make sure the bidder is signed with their private key
        if !bidder_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let highest_bidder_account = next_account_info(account_info_iter)?;
        let highest_bidder_ft_temp_account = next_account_info(account_info_iter)?;
        let highest_bidder_ft_returning_account = next_account_info(account_info_iter)?;

        let bidder_ft_temp_account = next_account_info(account_info_iter)?;
        let bidder_ft_account = next_account_info(account_info_iter)?;

        let escrow_account = next_account_info(account_info_iter)?;
        let mut auction_info = Auction::unpack(&escrow_account.try_borrow_data()?)?;

        let sys_var_clock_account = next_account_info(account_info_iter)?;
        let clock = &Clock::from_account_info(sys_var_clock_account)?;

        if auction_info.end_at <= clock.unix_timestamp {
            return Err(AuctionError::InactiveAuction.into());
        }

        // If it is not the highest bid, it will be an error
        // It works as expected even if multiple bidders bid at the same time and any amount is specified.
        // For example, if an instruction is issued that specifies 201 and 202 for the price of solana, the price that will be retained in the account will eventually be 202.
        // The instruction can be either 201 and 202 succeed, or 202 succeeds and 201 fails.
        if auction_info.price >= price {
            return Err(AuctionError::InsufficientBidPrice.into());
        }

        // Check if the data held in the account matches the information passed by the client.
        // The account used must be included in the Instruction argument 'accounts', as it is not possible to retrieve account information from within the Instruction using the account's public key.
        if auction_info.highest_bidder_ft_temp_pubkey != *highest_bidder_ft_temp_account.key {
            return Err(AuctionError::InvalidInstruction.into());
        }
        if auction_info.highest_bidder_ft_returning_pubkey
            != *highest_bidder_ft_returning_account.key
        {
            return Err(AuctionError::InvalidInstruction.into());
        }
        if auction_info.highest_bidder_pubkey != *highest_bidder_account.key {
            return Err(AuctionError::InvalidInstruction.into());
        }
        if auction_info.highest_bidder_pubkey == *bidder_account.key {
            return Err(AuctionError::AlreadyBid.into());
        }
        let token_program = next_account_info(account_info_iter)?;
        let pda_account = next_account_info(account_info_iter)?;
        let (pda, bump_seed) = Pubkey::find_program_address(&[b"escrow"], program_id);

        // Transfers the FT of the amount specified by 'price' from the bidder to escrow.
        // If you do not move the FT to 'Escrow' at the time of bidding,
        // you will not be able to settle due to insufficient FT at the end of the auction,
        // so this time we will move the FT to 'Escrow' at the time of bidding.
        let transfer_to_escrow_ix = spl_token::instruction::transfer(
            token_program.key,
            bidder_ft_account.key,
            bidder_ft_temp_account.key,
            bidder_account.key,
            &[], // authority_pubkey is default signer when the signer_pubkeys is empty.
            price,
        )?;
        msg!("Calling the token program to transfer FT to the escrow from the bidder");
        invoke(
            &transfer_to_escrow_ix,
            &[
                bidder_ft_account.clone(),
                bidder_ft_temp_account.clone(),
                bidder_account.clone(),
                token_program.clone(),
            ],
        )?;

        let owner_change_ix = spl_token::instruction::set_authority(
            token_program.key,
            bidder_ft_temp_account.key,
            Some(&pda),
            spl_token::instruction::AuthorityType::AccountOwner,
            bidder_account.key,
            &[], // owner_pubkey is default signer when the signer_pubkeys is empty.
        )?;
        msg!("Calling the token program to transfer token account ownership...");
        invoke(
            &owner_change_ix,
            &[
                bidder_ft_temp_account.clone(),
                bidder_account.clone(),
                token_program.clone(),
            ],
        )?;

        if auction_info.highest_bidder_pubkey != Pubkey::default(){
            // Since the highest bidder has changed, we will return the FT that the highest bidder has deposited so far.
            let transfer_to_previous_bidder_ix = spl_token::instruction::transfer(
                token_program.key,
                highest_bidder_ft_temp_account.key,
                highest_bidder_ft_returning_account.key,
                &pda,
                &[], // authority_pubkey is default signer when the signer_pubkeys is empty.
                auction_info.price,
            )?;
            msg!("Calling the token program to transfer FT to the previous highest bidder from the escrow");
            let signers_seeds: &[&[&[u8]]] = &[&[&b"escrow"[..], &[bump_seed]]];
            invoke_signed(
                &transfer_to_previous_bidder_ix,
                &[
                    highest_bidder_ft_temp_account.clone(),
                    highest_bidder_ft_returning_account.clone(),
                    pda_account.clone(),
                    token_program.clone(),
                ],
                signers_seeds,
            )?;

            Self::close_temporary_ft(
                token_program,
                highest_bidder_ft_temp_account,
                highest_bidder_account,
                pda,
                pda_account,
                signers_seeds,
            )?;
        }

        // Save the auction data on solana chain
        auction_info.price = price;
        auction_info.highest_bidder_pubkey = *bidder_account.key;
        auction_info.highest_bidder_ft_temp_pubkey = *bidder_ft_temp_account.key;
        auction_info.highest_bidder_ft_returning_pubkey = *bidder_ft_account.key;
        Auction::pack(auction_info, &mut escrow_account.try_borrow_mut_data()?)?;
        Ok(())
    }

    fn process_cancel(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let exhibitor_account = next_account_info(account_info_iter)?;

        // Check if it is signed with the seller's private key
        if !exhibitor_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let exhibiting_nft_temp_account = next_account_info(account_info_iter)?;
        let exhibiting_nft_returning_account = next_account_info(account_info_iter)?;
        let escrow_account = next_account_info(account_info_iter)?;
        let auction_info = Auction::unpack(&escrow_account.try_borrow_data()?)?;

        // Allow only exhibitor to cancel
        if auction_info.exhibitor_pubkey != *exhibitor_account.key {
            return Err(ProgramError::InvalidAccountData);
        }
        if auction_info.exhibiting_nft_temp_pubkey != *exhibiting_nft_temp_account.key {
            return Err(ProgramError::InvalidAccountData);
        }

        // Prevents cancellation if someone has already bid
        if auction_info.highest_bidder_pubkey != Pubkey::default() {
            return Err(AuctionError::AlreadyBid.into());
        }

        let (pda, bump_seed) = Pubkey::find_program_address(&[b"escrow"], program_id);
        let token_program = next_account_info(account_info_iter)?;
        let pda_account = next_account_info(account_info_iter)?;
        let signers_seeds: &[&[&[u8]]] = &[&[&b"escrow"[..], &[bump_seed]]];

        // The exhibitor will have the NFT returned.
        let exhibiting_nft_temp_account_data =
            TokenAccount::unpack(&exhibiting_nft_temp_account.try_borrow_data()?)?;
        let transfer_nft_to_exhibitor_ix = spl_token::instruction::transfer(
            token_program.key,
            exhibiting_nft_temp_account.key,
            exhibiting_nft_returning_account.key,
            &pda,
            &[], // authority_pubkey is default signer when the signer_pubkeys is empty.
            exhibiting_nft_temp_account_data.amount,
        )?;
        msg!("Calling the token program to transfer NFT to the exhibitor...");
        invoke_signed(
            &transfer_nft_to_exhibitor_ix,
            &[
                exhibiting_nft_temp_account.clone(),
                exhibiting_nft_returning_account.clone(),
                pda_account.clone(),
                token_program.clone(),
            ],
            signers_seeds,
        )?;

        // End the auction
        // If 'Escrow' has a token account with one or more quantities, close will fail.
        // In other words, all deposited FTs and NFTs must be returned to the owner.
        Self::close_escrow(
            token_program,
            exhibiting_nft_temp_account,
            exhibitor_account,
            pda,
            pda_account,
            escrow_account,
            signers_seeds,
        )
    }

    fn process_close(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let highest_bidder_account = next_account_info(account_info_iter)?;

        // The token account for receiving the winning bidder's NFT will be specified at the time of bidding,
        // so make sure that it is signed with the winning bidder's private key.
        if !highest_bidder_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let exhibitor_account = next_account_info(account_info_iter)?;
        let exhibiting_nft_temp_account = next_account_info(account_info_iter)?;
        let exhibitor_ft_receiving_account = next_account_info(account_info_iter)?;
        let highest_bidder_ft_temp_account = next_account_info(account_info_iter)?;
        let highest_bidder_nft_receiving_account = next_account_info(account_info_iter)?;
        let escrow_account = next_account_info(account_info_iter)?;
        let auction_info = Auction::unpack(&escrow_account.try_borrow_data()?)?;

        let sys_var_clock_account = next_account_info(account_info_iter)?;
        let clock = &Clock::from_account_info(sys_var_clock_account)?;

        if auction_info.end_at > clock.unix_timestamp {
            msg!(
                "Auction will be finished in {} seconds",
                (auction_info.end_at - clock.unix_timestamp)
            );
            return Err(AuctionError::ActiveAuction.into());
        }
        if auction_info.exhibitor_pubkey != *exhibitor_account.key {
            return Err(ProgramError::InvalidAccountData);
        }
        if auction_info.exhibiting_nft_temp_pubkey != *exhibiting_nft_temp_account.key {
            return Err(ProgramError::InvalidAccountData);
        }
        if auction_info.exhibitor_ft_receiving_pubkey != *exhibitor_ft_receiving_account.key {
            return Err(ProgramError::InvalidAccountData);
        }
        if auction_info.highest_bidder_ft_temp_pubkey != *highest_bidder_ft_temp_account.key {
            return Err(ProgramError::InvalidAccountData);
        }

        // Only the highest bidder can close auction.
        if auction_info.highest_bidder_pubkey != *highest_bidder_account.key {
            return Err(ProgramError::InvalidAccountData);
        }

        let (pda, bump_seed) = Pubkey::find_program_address(&[b"escrow"], program_id);
        let token_program = next_account_info(account_info_iter)?;
        let pda_account = next_account_info(account_info_iter)?;
        let signers_seeds: &[&[&[u8]]] = &[&[&b"escrow"[..], &[bump_seed]]];

        let exhibiting_nft_temp_account_data =
            TokenAccount::unpack(&exhibiting_nft_temp_account.try_borrow_data()?)?;

        // Transfer the winning NFT from Escrow to the winning bidder's token account
        let transfer_nft_to_highest_bidder_ix = spl_token::instruction::transfer(
            token_program.key,
            exhibiting_nft_temp_account.key,
            &highest_bidder_nft_receiving_account.key,
            &pda,
            &[], // authority_pubkey is default signer when the signer_pubkeys is empty.
            exhibiting_nft_temp_account_data.amount,
        )?;
        msg!("Calling the token program to transfer NFT to the highest bidder...");
        invoke_signed(
            &transfer_nft_to_highest_bidder_ix,
            &[
                exhibiting_nft_temp_account.clone(),
                highest_bidder_nft_receiving_account.clone(),
                pda_account.clone(),
                token_program.clone(),
            ],
            signers_seeds,
        )?;

        // Transfer FT deposited in Escrow to exhibitor
        let highest_bidder_ft_temp_account_data =
            TokenAccount::unpack(&highest_bidder_ft_temp_account.try_borrow_data()?)?;
        let transfer_ft_to_exhibitor_ix = spl_token::instruction::transfer(
            token_program.key,
            highest_bidder_ft_temp_account.key,
            &exhibitor_ft_receiving_account.key,
            &pda,
            &[], // authority_pubkey is default signer when the signer_pubkeys is empty.
            highest_bidder_ft_temp_account_data.amount,
        )?;
        msg!("Calling the token program to transfer FT to the exhibitor...");
        invoke_signed(
            &transfer_ft_to_exhibitor_ix,
            &[
                highest_bidder_ft_temp_account.clone(),
                exhibitor_ft_receiving_account.clone(),
                pda_account.clone(),
                token_program.clone(),
            ],
            signers_seeds,
        )?;

        Self::close_temporary_ft(
            token_program,
            highest_bidder_ft_temp_account,
            highest_bidder_account,
            pda,
            pda_account,
            signers_seeds,
        )?;

        Self::close_escrow(
            token_program,
            exhibiting_nft_temp_account,
            exhibitor_account,
            pda,
            pda_account,
            escrow_account,
            signers_seeds,
        )
    }

    fn close_escrow<'a, 'b>(
        token_program: &'a AccountInfo<'b>,
        exhibiting_nft_temp_account: &'a AccountInfo<'b>,
        exhibitor_account: &'a AccountInfo<'b>,
        pda: Pubkey,
        pda_account: &'a AccountInfo<'b>,
        escrow_account: &'a AccountInfo<'b>,
        signers_seed: &[&[&[u8]]],
    ) -> ProgramResult {
        let close_pdas_temp_acc_ix = spl_token::instruction::close_account(
            token_program.key,
            exhibiting_nft_temp_account.key,
            exhibitor_account.key,
            &pda,
            &[], // owner_pubkey is default signer when the signer_pubkeys is empty.
        )?;
        msg!("Calling the token program to close exhibitor's NFT temp account...");
        invoke_signed(
            &close_pdas_temp_acc_ix,
            &[
                exhibiting_nft_temp_account.clone(),
                exhibitor_account.clone(),
                pda_account.clone(),
                token_program.clone(),
            ],
            signers_seed,
        )?;

        msg!("Closing the escrow account...");
        **exhibitor_account.try_borrow_mut_lamports()? = exhibitor_account
            .lamports()
            .checked_add(escrow_account.lamports())
            .ok_or(AuctionError::AmountOverflow)?;
        **escrow_account.try_borrow_mut_lamports()? = 0;
        *escrow_account.try_borrow_mut_data()? = &mut [];

        Ok(())
    }

    fn close_temporary_ft<'a, 'b>(
        token_program: &'a AccountInfo<'b>,
        highest_bidder_ft_temp_account: &'a AccountInfo<'b>,
        highest_bidder_account: &'a AccountInfo<'b>,
        pda: Pubkey,
        pda_account: &'a AccountInfo<'b>,
        signers_seeds: &[&[&[u8]]],
    ) -> ProgramResult {
        let close_highest_bidder_ft_temp_acc_ix = spl_token::instruction::close_account(
            token_program.key,
            highest_bidder_ft_temp_account.key,
            highest_bidder_account.key,
            &pda,
            &[], // owner_pubkey is default signer when the signer_pubkeys is empty.
        )?;
        msg!("Calling the token program to close highest bidder FT temp account...");
        invoke_signed(
            &close_highest_bidder_ft_temp_acc_ix,
            &[
                highest_bidder_ft_temp_account.clone(),
                highest_bidder_account.clone(),
                pda_account.clone(),
                token_program.clone(),
            ],
            signers_seeds,
        )?;

        Ok(())
    }
}
