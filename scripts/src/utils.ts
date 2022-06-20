import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import * as BufferLayout from "@solana/buffer-layout";

// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore
import BN = require("bn.js");
import * as fs from "fs";

export const getConnection = () => {
  return new Connection(
    process.env.NETWORK || "http://localhost:8899",
    "confirmed"
  );
};

export const logError = (msg: string) => {
  console.log(`\x1b[31m${msg}\x1b[0m`);
};

export const writePublicKey = (publicKey: PublicKey, name: string) => {
  fs.writeFileSync(
    `./keys/${name}_pub.json`,
    JSON.stringify(publicKey.toString())
  );
};

export const getPublicKey = (name: string) =>
  new PublicKey(
    JSON.parse(fs.readFileSync(`./keys/${name}_pub.json`) as unknown as string)
  );

export const getPrivateKey = (name: string) =>
  Uint8Array.from(
    JSON.parse(fs.readFileSync(`./keys/${name}.json`) as unknown as string)
  );

export const getKeypair = (name: string) =>
  new Keypair({
    publicKey: getPublicKey(name).toBytes(),
    secretKey: getPrivateKey(name),
  });

export const getProgramId = () => {
  try {
    return getPublicKey("program");
  } catch (e) {
    logError("Given programId is missing or incorrect");
    process.exit(1);
  }
};

export const getTokenBalance = async (
  pubkey: PublicKey,
  connection: Connection
) => {
  try {
    return parseInt(
      (await connection.getTokenAccountBalance(pubkey)).value.amount
    );
  } catch (e) {
    logError(`Not a token account ${pubkey}`);
    return NaN;
  }
};

export const AUCTION_ACCOUNT_DATA_LAYOUT = BufferLayout.struct<Auction>([
  BufferLayout.u8("isInitialized"),
  BufferLayout.blob(32, "exhibitorPubkey"),
  BufferLayout.blob(32, "exhibitingNftTempPubkey"),
  BufferLayout.blob(32, "exhibitorFtReceivingPubkey"),
  BufferLayout.blob(8, "price"),
  BufferLayout.blob(8, "endAt"),
  BufferLayout.blob(32, "highestBidderPubkey"),
  BufferLayout.blob(32, "highestBidderFtTempPubkey"),
  BufferLayout.blob(32, "highestBidderFtReturningPubkey"),
]);

export interface Auction {
  isInitialized: number;
  exhibitorPubkey: Uint8Array;
  exhibitingNftTempPubkey: Uint8Array;
  exhibitorFtReceivingPubkey: Uint8Array;
  price: Uint8Array;
  endAt: Uint8Array;
  highestBidderPubkey: Uint8Array;
  highestBidderFtTempPubkey: Uint8Array;
  highestBidderFtReturningPubkey: Uint8Array;
}

export async function logAuction(connection: Connection) {
  const escrowPubkey = getPublicKey("escrow");
  const escrowAccount = await connection.getAccountInfo(escrowPubkey);
  if (!escrowAccount || escrowAccount.data.length === 0) {
    return;
  }
  const encodedAuctionState = escrowAccount!.data;
  const auction = AUCTION_ACCOUNT_DATA_LAYOUT.decode(
    encodedAuctionState
  ) as Auction;
  console.table({
    isInitialized: { value: auction.isInitialized },
    exhibitorPubkey: new PublicKey(auction.exhibitorPubkey).toBase58(),
    exhibitingNftTempPubkey: new PublicKey(
      auction.exhibitingNftTempPubkey
    ).toBase58(),
    exhibitorFtReceivingPubkey: new PublicKey(
      auction.exhibitorFtReceivingPubkey
    ).toBase58(),
    price: new BN(auction.price, 10, "le").toNumber(),
    endAt: new Date(
      new BN(auction.endAt, 10, "le").toNumber() * 1000
    ).toISOString(),
    highestBidderPubkey: new PublicKey(auction.highestBidderPubkey).toBase58(),
    highestBidderFtTempPubkey: new PublicKey(
      auction.highestBidderFtTempPubkey
    ).toBase58(),
    highestBidderFtReturningPubkey: new PublicKey(
      auction.highestBidderFtReturningPubkey
    ).toBase58(),
  });
}

export async function getCurrentAuction(
  connection: Connection,
  escrowPubkey: PublicKey,
  auctionProgramId: PublicKey
) {
  const escrowAccount = await connection.getAccountInfo(escrowPubkey);
  if (!escrowAccount || escrowAccount.data.length === 0) {
    throw new Error(`escrow account data found ${escrowPubkey}`);
  }
  const encodedAuctionState = escrowAccount.data;
  const auction = AUCTION_ACCOUNT_DATA_LAYOUT.decode(
    encodedAuctionState
  ) as Auction;
  const pda = await PublicKey.findProgramAddress(
    [Buffer.from("escrow")],
    auctionProgramId
  );
  return { pda, auction };
}
