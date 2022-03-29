import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { Keypair, PublicKey, SystemProgram } from '@solana/web3.js';
import { CoinFlip } from '../target/types/coin_flip';
import {
  getCoreState,
  getVaultAuth
} from './coin-flip_pda';

const program = anchor.workspace.CoinFlip as Program<CoinFlip>;

export async function initialize(admin: Keypair) {
  let [coreState, coreStateNonce] = await getCoreState(program.programId, admin.publicKey);
  let [vaultAuthority, vaultAuthNonce] = await getVaultAuth(program.programId, admin.publicKey);
  await program.rpc.initialize({
    coreStateNonce,
    vaultAuthNonce
  }, {
    accounts: {
      admin: admin.publicKey,
      coreState,
      vaultAuthority,
      systemProgram: SystemProgram.programId
    },
    signers: [admin]
  });
  return { coreState, vaultAuthority };
}

export async function depositSol(admin: Keypair, amount: number) {
  let [coreState, coreStateNonce] = await getCoreState(program.programId, admin.publicKey);
  let [vaultAuthority, vaultAuthNonce] = await getVaultAuth(program.programId, admin.publicKey);
  await program.rpc.depositSol({
    coreStateNonce,
    vaultAuthNonce,
    amount: new anchor.BN(amount)
  }, {
    accounts: {
      admin: admin.publicKey,
      vaultAuthority,
      systemProgram: SystemProgram.programId
    },
    signers: [admin]
  });
  return coreState;
}


export async function withdrawSol(admin: Keypair, amount: number) {
  let [coreState, coreStateNonce] = await getCoreState(program.programId, admin.publicKey);
  let [vaultAuthority, vaultAuthNonce] = await getVaultAuth(program.programId, admin.publicKey);
  await program.rpc.withdrawSol({
    coreStateNonce,
    vaultAuthNonce,
    amount: new anchor.BN(amount)
  }, {
    accounts: {
      admin: admin.publicKey,
      vaultAuthority,
      systemProgram: SystemProgram.programId
    },
    signers: [admin]
  });
  return coreState;
}
