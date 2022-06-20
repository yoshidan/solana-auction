import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import {
  PublicKey,
  Transaction,
  TransactionInstruction,
} from "@solana/web3.js";
import {
  getConnection,
  getCurrentAuction,
  getKeypair,
  getProgramId,
  getPublicKey,
  getTokenBalance,
  logAuction,
} from "./utils";

const cancel = async () => {
  const auctionProgramId = getProgramId();
  const exhibitorNftTokenAccountPubkey = getPublicKey("exhibitor_nft_x");
  const exhibitorKeypair = getKeypair("exhibitor");
  const escrowAccountPubkey = getPublicKey("escrow");

  const connection = getConnection();
  const { pda, auction } = await getCurrentAuction(
    connection,
    escrowAccountPubkey,
    auctionProgramId
  );

  const cancelInstruction = new TransactionInstruction({
    programId: auctionProgramId,
    data: Buffer.from(Uint8Array.of(2)),
    keys: [
      {
        pubkey: new PublicKey(auction.exhibitorPubkey),
        isSigner: true,
        isWritable: false,
      },
      {
        pubkey: new PublicKey(auction.exhibitingNftTempPubkey),
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: exhibitorNftTokenAccountPubkey,
        isSigner: false,
        isWritable: true,
      },
      { pubkey: escrowAccountPubkey, isSigner: false, isWritable: true },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: pda[0], isSigner: false, isWritable: false },
    ],
  });

  console.log("Sending cancel transaction...");
  await connection.sendTransaction(
    new Transaction().add(cancelInstruction),
    [exhibitorKeypair],
    { skipPreflight: false, preflightCommitment: "confirmed" }
  );

  // sleep to allow time to update
  await new Promise((resolve) => setTimeout(resolve, 1000));

  await logAuction(connection);
  const data = {
    NFT: {
      own: await getTokenBalance(exhibitorNftTokenAccountPubkey, connection),
      "Exhibitor Token Account": exhibitorNftTokenAccountPubkey.toBase58(),
    },
  };
  console.table(data);
};

cancel();
