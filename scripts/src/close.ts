import {
  getOrCreateAssociatedTokenAccount,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
  PublicKey,
  SYSVAR_CLOCK_PUBKEY,
  Transaction,
  TransactionInstruction,
} from "@solana/web3.js";
import {
  getKeypair,
  getProgramId,
  getPublicKey,
  getTokenBalance,
  logAuction,
  getCurrentAuction,
  getConnection,
} from "./utils";

const close = async () => {
  const bidderNo = process.env.BIDDER;
  const auctionProgramId = getProgramId();
  const bidderKeypair = getKeypair(`bidder${bidderNo}`);
  const connection = getConnection();

  const escrowAccountPubkey = getPublicKey("escrow");
  const { pda, auction } = await getCurrentAuction(
    connection,
    escrowAccountPubkey,
    auctionProgramId
  );

  const nftMintPubkey = getPublicKey("mint_nft_x");
  const bidderNftReceivingAccount = await getOrCreateAssociatedTokenAccount(
    connection,
    bidderKeypair,
    nftMintPubkey,
    bidderKeypair.publicKey,
    undefined,
    undefined
  );

  const closeInstruction = new TransactionInstruction({
    programId: auctionProgramId,
    data: Buffer.from(Uint8Array.of(3)),
    keys: [
      { pubkey: bidderKeypair.publicKey, isSigner: true, isWritable: false },
      {
        pubkey: new PublicKey(auction.exhibitorPubkey),
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: new PublicKey(auction.exhibitingNftTempPubkey),
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: new PublicKey(auction.exhibitorFtReceivingPubkey),
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: new PublicKey(auction.highestBidderFtTempPubkey),
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: bidderNftReceivingAccount.address,
        isSigner: false,
        isWritable: true,
      },
      { pubkey: escrowAccountPubkey, isSigner: false, isWritable: true },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: pda[0], isSigner: false, isWritable: false },
    ],
  });

  console.log("Sending close transaction...");
  await connection.sendTransaction(
    new Transaction().add(closeInstruction),
    [bidderKeypair],
    { skipPreflight: false, preflightCommitment: "confirmed" }
  );

  // sleep to allow time to update
  await new Promise((resolve) => setTimeout(resolve, 2000));

  const exhibitorNftPubkey = getPublicKey("exhibitor_nft_x");
  const exhibitorFtPubkey = new PublicKey(auction.exhibitorFtReceivingPubkey);
  const bidderFtPubkey = new PublicKey(auction.highestBidderFtReturningPubkey);

  await logAuction(connection);
  const data = {
    NFT: {
      exhibitor: await getTokenBalance(exhibitorNftPubkey, connection),
      "Exhibitor Token Account": exhibitorNftPubkey.toBase58(),
      bidder: await getTokenBalance(
        bidderNftReceivingAccount.address,
        connection
      ),
      "Bidder Token Account": bidderNftReceivingAccount.address.toBase58(),
    },
    FT: {
      exhibitor: await getTokenBalance(exhibitorFtPubkey, connection),
      "Exhibitor Token Account": exhibitorFtPubkey.toBase58(),
      bidder: await getTokenBalance(bidderFtPubkey, connection),
      "Bidder Token Account": bidderFtPubkey.toBase58(),
    },
  };
  console.table(data);
};

close();
