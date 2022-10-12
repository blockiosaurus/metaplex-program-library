import { NftWithToken, SftWithToken } from '@metaplex-foundation/js';
import { Connection, Keypair, PublicKey, Transaction } from '@solana/web3.js';
import {
  createCreateEscrowAccountInstruction,
  createTransferIntoEscrowInstruction,
} from '@metaplex-foundation/mpl-token-metadata/src/generated';
import { findToePda } from './pdas';
import { getAssociatedTokenAddress } from '@solana/spl-token';

export const createTOE = async (connection: Connection, nft: NftWithToken, keypair: Keypair) => {
  const escrowAccountAddress = await findToePda(nft.mint.address);

  const createIX = createCreateEscrowAccountInstruction({
    escrow: escrowAccountAddress[0],
    metadata: nft.metadataAddress,
    mint: nft.mint.address,
    edition: nft.edition.address,
    payer: keypair.publicKey,
    escrowConstraintModel: undefined,
  });

  const tx = new Transaction().add(createIX);

  const { blockhash } = await connection.getLatestBlockhash();
  tx.recentBlockhash = blockhash;
  tx.feePayer = keypair.publicKey;
  await connection.sendTransaction(tx, [keypair]);

  return escrowAccountAddress;
};

export const transferIn = async (
  connection: Connection,
  escrowNft: NftWithToken,
  escrowAccountAddress: PublicKey,
  nft: NftWithToken | SftWithToken,
  keypair: Keypair,
) => {
  const dst: PublicKey = await getAssociatedTokenAddress(
    nft.mint.address,
    escrowAccountAddress,
    true,
  );
  console.log('dst:');
  console.log(dst.toString());
  const transferIX = createTransferIntoEscrowInstruction(
    {
      escrow: escrowAccountAddress,
      payer: keypair.publicKey,
      attributeMint: nft.mint.address,
      attributeSrc: nft.token.address,
      attributeDst: dst,
      attributeMetadata: nft.metadataAddress,
      escrowMint: escrowNft.mint.address,
      escrowAccount: escrowNft.token.address,
      constraintModel: new PublicKey('CjWwgEJUBdb2iYjZsDng5qkYRC8f3rQeqr4k1j9jExr5'),
    },
    {
      transferIntoEscrowArgs: { amount: 1, index: 1 },
    },
  );

  const tx = new Transaction().add(transferIX);

  // let accountInfo = await connection.getAccountInfo(nft.token.address);
  // if (accountInfo){
  //     let account = AccountLayout.decode(accountInfo.data);
  //     console.log(account);
  // }

  const { blockhash } = await connection.getLatestBlockhash();
  tx.recentBlockhash = blockhash;
  tx.feePayer = keypair.publicKey;
  console.log(tx);
  await connection.sendTransaction(tx, [keypair], { skipPreflight: true });
};
