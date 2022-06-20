import {
  Connection,
  LAMPORTS_PER_SOL,
  PublicKey,
  sendAndConfirmTransaction,
  Signer,
  Transaction,
} from "@solana/web3.js";

import {
  createMint as createMintInternal,
  createAccount,
  TOKEN_PROGRAM_ID,
  mintTo,
  createSetAuthorityInstruction,
  AuthorityType,
} from "@solana/spl-token";
import {
  getConnection,
  getKeypair,
  getPublicKey,
  getTokenBalance,
  writePublicKey,
} from "./utils";

const createMint = (
  connection: Connection,
  { publicKey, secretKey }: Signer
) => {
  return createMintInternal(
    connection,
    {
      publicKey,
      secretKey,
    },
    publicKey,
    null,
    0,
    undefined,
    undefined,
    TOKEN_PROGRAM_ID
  );
};

const setupMintNFT = async (
  name: string,
  connection: Connection,
  exhibitorPublicKey: PublicKey,
  clientKeypair: Signer
): Promise<PublicKey> => {
  console.log(`Creating Mint NFT ${name}...`);
  const mintPubkey = await createMint(connection, clientKeypair);
  writePublicKey(mintPubkey, `mint_nft_${name.toLowerCase()}`);

  const exhibitorTokenAccount = await createAccount(
    connection,
    clientKeypair,
    mintPubkey,
    exhibitorPublicKey,
    undefined,
    undefined,
    TOKEN_PROGRAM_ID
  );
  writePublicKey(exhibitorTokenAccount, `exhibitor_nft_${name.toLowerCase()}`);

  await mintTo(
    connection,
    clientKeypair,
    mintPubkey,
    exhibitorTokenAccount,
    clientKeypair.publicKey,
    1,
    [],
    undefined,
    TOKEN_PROGRAM_ID
  );
  const transaction = new Transaction().add(
    createSetAuthorityInstruction(
      mintPubkey,
      clientKeypair.publicKey,
      AuthorityType.MintTokens,
      null
    )
  );
  await sendAndConfirmTransaction(connection, transaction, [clientKeypair]);
  return exhibitorTokenAccount;
};

const setupMintFT = async (
  name: string,
  connection: Connection,
  exhibitorPublicKey: PublicKey,
  bidder1PublicKey: PublicKey,
  bidder2PublicKey: PublicKey,
  clientKeypair: Signer
): Promise<[PublicKey, PublicKey, PublicKey, PublicKey]> => {
  console.log(`Creating Mint FT ${name}...`);
  const mintPubkey = await createMint(connection, clientKeypair);
  writePublicKey(mintPubkey, `mint_ft_${name.toLowerCase()}`);
  const exhibitorTokenAccount = await createAccount(
    connection,
    clientKeypair,
    mintPubkey,
    exhibitorPublicKey,
    undefined,
    undefined,
    TOKEN_PROGRAM_ID
  );
  writePublicKey(exhibitorTokenAccount, `exhibitor_ft_${name.toLowerCase()}`);
  const bidder1TokenAccount = await createAccount(
    connection,
    clientKeypair,
    mintPubkey,
    bidder1PublicKey,
    undefined,
    undefined,
    TOKEN_PROGRAM_ID
  );
  writePublicKey(bidder1TokenAccount, `bidder1_ft_${name.toLowerCase()}`);
  const bidder2TokenAccount = await createAccount(
    connection,
    clientKeypair,
    mintPubkey,
    bidder2PublicKey,
    undefined,
    undefined,
    TOKEN_PROGRAM_ID
  );
  writePublicKey(bidder2TokenAccount, `bidder2_ft_${name.toLowerCase()}`);
  return [
    mintPubkey,
    exhibitorTokenAccount,
    bidder1TokenAccount,
    bidder2TokenAccount,
  ];
};

const setup = async () => {
  const exhibitorPublicKey = getPublicKey("exhibitor");
  const bidder1PublicKey = getPublicKey("bidder1");
  const bidder2PublicKey = getPublicKey("bidder2");
  const clientKeypair = getKeypair("id");

  const connection = getConnection();
  // some networks like the local network provide an airdrop function (mainnet of course does not)
  await connection.requestAirdrop(exhibitorPublicKey, LAMPORTS_PER_SOL * 2);
  await connection.requestAirdrop(bidder1PublicKey, LAMPORTS_PER_SOL * 2);
  await connection.requestAirdrop(bidder2PublicKey, LAMPORTS_PER_SOL * 2);
  await connection.requestAirdrop(
    clientKeypair.publicKey,
    LAMPORTS_PER_SOL * 2
  );

  const exhibitorNftAccount = await setupMintNFT(
    "X",
    connection,
    exhibitorPublicKey,
    clientKeypair
  );

  const [mintFt, exhibitorFTAccount, bidder1FTAccount, bidder2FTAccount] =
    await setupMintFT(
      "NAO",
      connection,
      exhibitorPublicKey,
      bidder1PublicKey,
      bidder2PublicKey,
      clientKeypair
    );
  await mintTo(
    connection,
    clientKeypair,
    mintFt,
    exhibitorFTAccount,
    clientKeypair.publicKey,
    500,
    [],
    undefined,
    TOKEN_PROGRAM_ID
  );
  await mintTo(
    connection,
    clientKeypair,
    mintFt,
    bidder1FTAccount,
    clientKeypair.publicKey,
    500,
    [],
    undefined,
    TOKEN_PROGRAM_ID
  );
  await mintTo(
    connection,
    clientKeypair,
    mintFt,
    bidder2FTAccount,
    clientKeypair.publicKey,
    500,
    [],
    undefined,
    TOKEN_PROGRAM_ID
  );

  const data = {
    exhibitor: {
      "Wallet Pubkey": exhibitorPublicKey.toBase58(),
      FT: await getTokenBalance(exhibitorFTAccount, connection),
      "FT(NAO) Account PubKey": exhibitorFTAccount.toBase58(),
      NFT: await getTokenBalance(exhibitorNftAccount, connection),
      "NFT(X) Account PubKey": exhibitorNftAccount.toBase58(),
    },
    bidder1: {
      "Wallet Pubkey": getPublicKey("bidder1").toBase58(),
      FT: await getTokenBalance(bidder1FTAccount, connection),
      "FT(NAO) Account PubKey": bidder1FTAccount.toBase58(),
      NFT: 0,
      "NFT(X) Account PubKey": "",
    },
    bidder2: {
      "Wallet Pubkey": getPublicKey("bidder2").toBase58(),
      FT: await getTokenBalance(bidder2FTAccount, connection),
      "FT(NAO) Account PubKey": bidder2FTAccount.toBase58(),
      NFT: 0,
      "NFT(X) Account PubKey": "",
    },
  };
  console.table(data);
  console.log("");
};

setup();
