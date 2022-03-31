import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { Keypair, PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY } from '@solana/web3.js';
import { getAssociatedTokenAddress, TOKEN_PROGRAM_ID, NATIVE_MINT } from '@solana/spl-token';
import { CoinFlip } from '../target/types/coin_flip';
import {
  getCoreState,
  getVaultAuth,
  getVaultTokenAccount
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

export async function register(admin: Keypair, tokenMint: PublicKey) {
  let [coreState, coreStateNonce] = await getCoreState(program.programId, admin.publicKey);
  let [vaultAuthority, vaultAuthNonce] = await getVaultAuth(program.programId, admin.publicKey);
  let [vaultTokenAccount, vaultTokenAccountNonce] = await getVaultTokenAccount(program.programId, tokenMint, admin.publicKey);
  await program.rpc.register({
    vaultTokenAccountNonce
  }, {
    accounts: {
      coreState,
      admin: admin.publicKey,
      tokenMint,
      vaultTokenAccount,
      vaultAuthority,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      rent: SYSVAR_RENT_PUBKEY
    },
    signers: [admin]
  });
}

export async function deposit(admin: Keypair, tokenMint: PublicKey, amount: number) {
  let [coreState, coreStateNonce] = await getCoreState(program.programId, admin.publicKey);
  let [vaultAuthority, vaultAuthNonce] = await getVaultAuth(program.programId, admin.publicKey);
  
  let adminTokenAccount = (tokenMint == NATIVE_MINT) ? 
    admin.publicKey : (await getAssociatedTokenAddress(tokenMint, admin.publicKey));
  let vaultTokenAccount;
  if (tokenMint === NATIVE_MINT) vaultTokenAccount = vaultAuthority;
  else {
    let [_vaultTokenAccount, _vaultTokenAccountNonce] = await getVaultTokenAccount(program.programId, tokenMint, admin.publicKey);
    vaultTokenAccount = _vaultTokenAccount;
  }
    
  let tx = await program.rpc.deposit({
    amount: new anchor.BN(amount)
  }, {
    accounts: {
      coreState,
      admin: admin.publicKey,
      vaultAuthority,
      tokenMint,
      adminTokenAccount,
      vaultTokenAccount,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    },
    signers: [admin]
  });
  return tx;
}

export async function withdraw(admin: Keypair, tokenMint: PublicKey, amount: number) {
  let [coreState, coreStateNonce] = await getCoreState(program.programId, admin.publicKey);
  let [vaultAuthority, vaultAuthNonce] = await getVaultAuth(program.programId, admin.publicKey);
  
  let adminTokenAccount = (tokenMint == NATIVE_MINT) ? 
    admin.publicKey : (await getAssociatedTokenAddress(tokenMint, admin.publicKey));
  let vaultTokenAccount;
  if (tokenMint === NATIVE_MINT) vaultTokenAccount = vaultAuthority;
  else {
    let [_vaultTokenAccount, _vaultTokenAccountNonce] = await getVaultTokenAccount(program.programId, tokenMint, admin.publicKey);
    vaultTokenAccount = _vaultTokenAccount;
  }
    
  let tx = await program.rpc.withdraw({
    amount: new anchor.BN(amount)
  }, {
    accounts: {
      coreState,
      admin: admin.publicKey,
      vaultAuthority,
      tokenMint,
      adminTokenAccount,
      vaultTokenAccount,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    },
    signers: [admin]
  });
  return tx;
}

export async function bet(admin: Keypair, amount: number) {
  let [coreState, coreStateNonce] = await getCoreState(program.programId, admin.publicKey);
  let [vaultAuthority, vaultAuthNonce] = await getVaultAuth(program.programId, admin.publicKey);
  // await program.rpc.withdraw({
  //   coreStateNonce,
  //   vaultAuthNonce,
  //   amount: new anchor.BN(amount)
  // }, {
  //   accounts: {
  //     admin: admin.publicKey,
  //     vaultAuthority,
  //     systemProgram: SystemProgram.programId
  //   },
  //   signers: [admin]
  // });
  return coreState;
}
