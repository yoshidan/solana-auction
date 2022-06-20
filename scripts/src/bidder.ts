import {
  AccountLayout,
  createInitializeAccountInstruction,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_CLOCK_PUBKEY,
  Transaction,
  TransactionInstruction,
} from "@solana/web3.js";
// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore
import BN = require("bn.js");
import {
  getKeypair,
  getProgramId,
  getPublicKey,
  getTokenBalance,
  logAuction,
  getCurrentAuction,
  getConnection,
} from "./utils";

const bidder = async () => {
  const price = process.argv[2];
  console.log(`bidding price = ${price}`);
  const bidderNo = process.env.BIDDER;
  const auctionProgramId = getProgramId();
  const bidderFtAccountPubkey = getPublicKey(`bidder${bidderNo}_ft_nao`);
  const bidderKeypair = getKeypair(`bidder${bidderNo}`);
  const bidderFtTempAccountKeypair = new Keypair();
  const ftMintPubkey = getPublicKey("mint_ft_nao");
  const connection = getConnection();

  const escrowAccountPubkey = getPublicKey("escrow");
  const { pda, auction } = await getCurrentAuction(
    connection,
    escrowAccountPubkey,
    auctionProgramId
  );

  const createTempFtAccountIx = SystemProgram.createAccount({
    programId: TOKEN_PROGRAM_ID,
    space: AccountLayout.span,
    lamports: await connection.getMinimumBalanceForRentExemption(
      AccountLayout.span
    ),
    fromPubkey: bidderKeypair.publicKey,
    newAccountPubkey: bidderFtTempAccountKeypair.publicKey,
  });
  const initTempFtAccountIx = createInitializeAccountInstruction(
    bidderFtTempAccountKeypair.publicKey,
    ftMintPubkey,
    bidderKeypair.publicKey,
    TOKEN_PROGRAM_ID
  );

  const highestBidderPubkey = new PublicKey(auction.highestBidderPubkey);
  const highestBidderFtTempPubkey = new PublicKey(
    auction.highestBidderFtTempPubkey
  );
  const highestBidderFtReturningPubkey = new PublicKey(
    auction.highestBidderFtReturningPubkey
  );
  const bidIx = new TransactionInstruction({
    programId: auctionProgramId,
    keys: [
      { pubkey: bidderKeypair.publicKey, isSigner: true, isWritable: false },
      { pubkey: highestBidderPubkey, isSigner: false, isWritable: true },
      { pubkey: highestBidderFtTempPubkey, isSigner: false, isWritable: true },
      {
        pubkey: highestBidderFtReturningPubkey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: bidderFtTempAccountKeypair.publicKey,
        isSigner: false,
        isWritable: true,
      },
      { pubkey: bidderFtAccountPubkey, isSigner: false, isWritable: true },
      { pubkey: escrowAccountPubkey, isSigner: false, isWritable: true },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: pda[0], isSigner: false, isWritable: false },
    ],
    data: Buffer.from(Uint8Array.of(1, ...new BN(price).toArray("le", 8))),
  });

  const tx = new Transaction().add(
    createTempFtAccountIx,
    initTempFtAccountIx,
    bidIx
  );
  console.log("Sending Bidder's transaction...");
  await connection.sendTransaction(
    tx,
    [bidderKeypair, bidderFtTempAccountKeypair],
    { skipPreflight: false, preflightCommitment: "confirmed" }
  );

  // sleep to allow time to update
  await new Promise((resolve) => setTimeout(resolve, 1000));

  await logAuction(connection);
  const data = {
    "Bidder's FT ": {
      own: await getTokenBalance(bidderFtAccountPubkey, connection),
      "Bidder Token Account": bidderFtAccountPubkey.toBase58(),
      escrow: await getTokenBalance(
        bidderFtTempAccountKeypair.publicKey,
        connection
      ),
      "Temp Token Account": bidderFtTempAccountKeypair.publicKey.toBase58(),
    },
    "Previous Highest Bidder's FT": {
      own: await getTokenBalance(highestBidderFtReturningPubkey, connection),
      "Bidder Token Account": highestBidderFtReturningPubkey.toBase58(),
      escrow: await getTokenBalance(highestBidderFtTempPubkey, connection),
      "Temp Token Account": highestBidderFtTempPubkey.toBase58(),
    },
  };
  console.table(data);
  console.log("");
};

bidder();
