import {
  AccountLayout,
  createInitializeAccountInstruction,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
  Keypair,
  SystemProgram,
  SYSVAR_CLOCK_PUBKEY,
  SYSVAR_RENT_PUBKEY,
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
  AUCTION_ACCOUNT_DATA_LAYOUT,
  writePublicKey,
  getTokenBalance,
  logAuction,
  getConnection,
} from "./utils";

const exhibitor = async () => {
  const price = process.argv[2] || 200;
  console.log(`initial price = ${price}`);
  const duration = process.argv[3] || 3600;
  console.log(`auction duration = ${duration} sec`);
  const auctionProgramId = getProgramId();
  const exhibitorNftAccountPubkey = getPublicKey("exhibitor_nft_x");
  const nftMintPubkey = getPublicKey("mint_nft_x");
  const exhibitorKeypair = getKeypair("exhibitor");
  const exhibitorFtReceivingPubkey = getPublicKey("exhibitor_ft_nao");
  const exhibitingNftTempAccountKeypair = new Keypair();
  const connection = getConnection();

  // Create escrow's temp account for NFT
  const createTempNftAccountIx = SystemProgram.createAccount({
    programId: TOKEN_PROGRAM_ID,
    space: AccountLayout.span,
    lamports: await connection.getMinimumBalanceForRentExemption(
      AccountLayout.span
    ),
    fromPubkey: exhibitorKeypair.publicKey,
    newAccountPubkey: exhibitingNftTempAccountKeypair.publicKey,
  });
  const initTempNftAccountIx = createInitializeAccountInstruction(
    exhibitingNftTempAccountKeypair.publicKey,
    nftMintPubkey,
    exhibitorKeypair.publicKey,
    TOKEN_PROGRAM_ID
  );

  // Here, a new escrow key pair is generated. Only the seller knows the escrow key pair because it is the seller who performs this process.
  const escrowKeypair = new Keypair();
  const createEscrowAccountIx = SystemProgram.createAccount({
    space: AUCTION_ACCOUNT_DATA_LAYOUT.span,
    lamports: await connection.getMinimumBalanceForRentExemption(
      AUCTION_ACCOUNT_DATA_LAYOUT.span
    ),
    fromPubkey: exhibitorKeypair.publicKey,
    newAccountPubkey: escrowKeypair.publicKey,
    programId: auctionProgramId,
  });
  // The programId, keys, and data passed here will be passed to the Solana Program.
  const exhibitIx = new TransactionInstruction({
    programId: auctionProgramId,
    keys: [
      { pubkey: exhibitorKeypair.publicKey, isSigner: true, isWritable: false },
      { pubkey: exhibitorNftAccountPubkey, isSigner: false, isWritable: true },
      {
        pubkey: exhibitingNftTempAccountKeypair.publicKey,
        isSigner: false,
        isWritable: true,
      },
      { pubkey: exhibitorFtReceivingPubkey, isSigner: false, isWritable: true },
      { pubkey: escrowKeypair.publicKey, isSigner: false, isWritable: true },
      { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    ],
    data: Buffer.from(
      Uint8Array.of(
        0,
        ...new BN(price).toArray("le", 8),
        ...new BN(duration).toArray("le", 8)
      )
    ),
  });

  const tx = new Transaction().add(
    createTempNftAccountIx,
    initTempNftAccountIx,
    createEscrowAccountIx,
    exhibitIx
  );
  console.log("Sending Exhibitor's transaction...");
  await connection.sendTransaction(
    tx,
    // A key pair is required for public keys that are set to 'isSigner: true' in the instruction.
    [exhibitorKeypair, exhibitingNftTempAccountKeypair, escrowKeypair],
    { skipPreflight: false, preflightCommitment: "confirmed" }
  );

  // sleep to allow time to update
  await new Promise((resolve) => setTimeout(resolve, 1500));

  const exhibitorXBalance = await getTokenBalance(
    exhibitorNftAccountPubkey,
    connection
  );
  const escrowXBalance = await getTokenBalance(
    exhibitingNftTempAccountKeypair.publicKey,
    connection
  );
  writePublicKey(escrowKeypair.publicKey, "escrow");

  await logAuction(connection);
  const data = {
    NFT: {
      exhibitor: exhibitorXBalance,
      "Exhibitor Token Account": exhibitorNftAccountPubkey.toBase58(),
      escrow: escrowXBalance,
      "Temp Token Account":
        exhibitingNftTempAccountKeypair.publicKey.toBase58(),
    },
    FT: {
      exhibitor: await getTokenBalance(exhibitorFtReceivingPubkey, connection),
      "Exhibitor Token Account": exhibitorFtReceivingPubkey.toBase58(),
      escrow: 0,
      "Temp Token Account": "-",
    },
  };
  console.table(data);

  console.log("");
};

exhibitor();
