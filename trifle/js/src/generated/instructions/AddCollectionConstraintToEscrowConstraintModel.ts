/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';
import {
  AddCollectionConstraintToEscrowConstraintModelArgs,
  addCollectionConstraintToEscrowConstraintModelArgsBeet,
} from '../types/AddCollectionConstraintToEscrowConstraintModelArgs';

/**
 * @category Instructions
 * @category AddCollectionConstraintToEscrowConstraintModel
 * @category generated
 */
export type AddCollectionConstraintToEscrowConstraintModelInstructionArgs = {
  addCollectionConstraintToEscrowConstraintModelArgs: AddCollectionConstraintToEscrowConstraintModelArgs;
};
/**
 * @category Instructions
 * @category AddCollectionConstraintToEscrowConstraintModel
 * @category generated
 */
export const AddCollectionConstraintToEscrowConstraintModelStruct = new beet.FixableBeetArgsStruct<
  AddCollectionConstraintToEscrowConstraintModelInstructionArgs & {
    instructionDiscriminator: number;
  }
>(
  [
    ['instructionDiscriminator', beet.u8],
    [
      'addCollectionConstraintToEscrowConstraintModelArgs',
      addCollectionConstraintToEscrowConstraintModelArgsBeet,
    ],
  ],
  'AddCollectionConstraintToEscrowConstraintModelInstructionArgs',
);
/**
 * Accounts required by the _AddCollectionConstraintToEscrowConstraintModel_ instruction
 *
 * @property [_writable_] constraintModel Constraint model account
 * @property [_writable_, **signer**] payer Wallet paying for the transaction and new account, will be set as the creator of the constraint model
 * @property [**signer**] updateAuthority Update authority of the constraint model
 * @property [] collectionMint Collection mint account
 * @property [] collectionMintMetadata Collection mint metadata account
 * @property [] sysvarInstructions Instructions sysvar account
 * @category Instructions
 * @category AddCollectionConstraintToEscrowConstraintModel
 * @category generated
 */
export type AddCollectionConstraintToEscrowConstraintModelInstructionAccounts = {
  constraintModel: web3.PublicKey;
  payer: web3.PublicKey;
  updateAuthority: web3.PublicKey;
  collectionMint: web3.PublicKey;
  collectionMintMetadata: web3.PublicKey;
  systemProgram?: web3.PublicKey;
  sysvarInstructions: web3.PublicKey;
};

export const addCollectionConstraintToEscrowConstraintModelInstructionDiscriminator = 5;

/**
 * Creates a _AddCollectionConstraintToEscrowConstraintModel_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 *
 * @category Instructions
 * @category AddCollectionConstraintToEscrowConstraintModel
 * @category generated
 */
export function createAddCollectionConstraintToEscrowConstraintModelInstruction(
  accounts: AddCollectionConstraintToEscrowConstraintModelInstructionAccounts,
  args: AddCollectionConstraintToEscrowConstraintModelInstructionArgs,
  programId = new web3.PublicKey('trifMWutwBxkSuatmpPVnEe7NoE3BJKgjVi8sSyoXWX'),
) {
  const [data] = AddCollectionConstraintToEscrowConstraintModelStruct.serialize({
    instructionDiscriminator:
      addCollectionConstraintToEscrowConstraintModelInstructionDiscriminator,
    ...args,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: accounts.constraintModel,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.payer,
      isWritable: true,
      isSigner: true,
    },
    {
      pubkey: accounts.updateAuthority,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: accounts.collectionMint,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.collectionMintMetadata,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.systemProgram ?? web3.SystemProgram.programId,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.sysvarInstructions,
      isWritable: false,
      isSigner: false,
    },
  ];

  const ix = new web3.TransactionInstruction({
    programId,
    keys,
    data,
  });
  return ix;
}