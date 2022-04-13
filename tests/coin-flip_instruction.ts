import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { Keypair, PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, Transaction } from '@solana/web3.js';
import { getAssociatedTokenAddress, TOKEN_PROGRAM_ID, NATIVE_MINT } from '@solana/spl-token';
import { CoinFlip } from '../target/types/coin_flip';
import {
  getCoreState,
  getVaultAuth,
  getVaultTokenAccount,
  getBetState, getAllowed
} from './coin-flip_pda';

const program = anchor.workspace.CoinFlip as Program<CoinFlip>;

export async function initialize(admin: Keypair, executer: Keypair, feePercent: number, winRatio: number) {
  const [coreState, coreStateNonce] = await getCoreState(program.programId, admin.publicKey);
  const [vaultAuthority, vaultAuthNonce] = await getVaultAuth(program.programId, admin.publicKey);
  await program.rpc.initialize({
    coreStateNonce,
    vaultAuthNonce,
    feePercent: new anchor.BN(feePercent * 100),
    winRatio: new anchor.BN(winRatio * 100)
  }, {
    accounts: {
      admin: admin.publicKey,
      executer: executer.publicKey,
      coreState,
      vaultAuthority,
      systemProgram: SystemProgram.programId
    },
    signers: [admin]
  });
  return { coreState, vaultAuthority };
}

export async function register(admin: Keypair, tokenMint: PublicKey, amounts: number[]) {
  const [coreState] = await getCoreState(program.programId, admin.publicKey);
  const [vaultAuthority] = await getVaultAuth(program.programId, admin.publicKey);
  const [vaultTokenAccount, vaultTokenAccountNonce] = await getVaultTokenAccount(program.programId, tokenMint, admin.publicKey);
  const [allowed] = await getAllowed(program.programId, tokenMint, admin.publicKey);
  await program.rpc.register({
    vaultTokenAccountNonce,
    amounts: amounts.map(i => new anchor.BN(i))
  }, {
    accounts: {
      allowedBets:
      allowed,
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
  const [coreState] = await getCoreState(program.programId, admin.publicKey);
  const [vaultAuthority] = await getVaultAuth(program.programId, admin.publicKey);
  
  const adminTokenAccount = (tokenMint.toBase58() === NATIVE_MINT.toBase58()) ?
    admin.publicKey : (await getAssociatedTokenAddress(tokenMint, admin.publicKey));
  let vaultTokenAccount;
  if (tokenMint.toBase58() === NATIVE_MINT.toBase58()) {
    vaultTokenAccount = vaultAuthority;
  } else {
    const [_vaultTokenAccount] = await getVaultTokenAccount(program.programId, tokenMint, admin.publicKey);
    vaultTokenAccount = _vaultTokenAccount;
  }
    
  await program.rpc.deposit({
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
}

export async function withdraw(admin: Keypair, tokenMint: PublicKey, amount: number) {
  const [coreState] = await getCoreState(program.programId, admin.publicKey);
  const [vaultAuthority] = await getVaultAuth(program.programId, admin.publicKey);
  
  const adminTokenAccount = (tokenMint.toBase58() === NATIVE_MINT.toBase58()) ?
    admin.publicKey : (await getAssociatedTokenAddress(tokenMint, admin.publicKey));
  let vaultTokenAccount;
  if (tokenMint.toBase58() === NATIVE_MINT.toBase58()) {
    vaultTokenAccount = vaultAuthority;
  } else {
    let [_vaultTokenAccount] = await getVaultTokenAccount(program.programId, tokenMint, admin.publicKey);
    vaultTokenAccount = _vaultTokenAccount;
  }
  
  await program.rpc.withdraw({
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
}

export async function betDirectly(admin: PublicKey, user: Keypair, tokenMint: PublicKey, amount: number, betSide: boolean) {
  const [coreState] = await getCoreState(program.programId, admin);
  const [vaultAuthority] = await getVaultAuth(program.programId, admin);
  
  const userTokenAccount = (tokenMint.toBase58() === NATIVE_MINT.toBase58()) ?
    user.publicKey : (await getAssociatedTokenAddress(tokenMint, user.publicKey));
  let vaultTokenAccount;
  if (tokenMint.toBase58() === NATIVE_MINT.toBase58()) {
    vaultTokenAccount = vaultAuthority;
  } else {
    let [_vaultTokenAccount] = await getVaultTokenAccount(program.programId, tokenMint, admin);
    vaultTokenAccount = _vaultTokenAccount;
  }

  const [allowed, allowedNonce] = await getAllowed(program.programId, tokenMint, admin);
  await program.rpc.betDirectly({
    amount: new anchor.BN(amount),
    betSide,
    allowedAmountsNonce: allowedNonce
  }, {
    accounts: {
      coreState,
      allowedBets: allowed,
      user: user.publicKey,
      vaultAuthority,
      tokenMint,
      userTokenAccount,
      vaultTokenAccount,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId
    },
    signers: [user]
  });
}

export async function bet(admin: PublicKey, user: Keypair, tokenMint: PublicKey, amount: number, betSide: boolean) {
  const [coreState] = await getCoreState(program.programId, admin);
  const [vaultAuthority] = await getVaultAuth(program.programId, admin);
  const flipCounter = parseInt((await program.account.coreState.fetch(coreState)).flipCounter);
  const [betState, betStateNonce] = await getBetState(program.programId, admin, user.publicKey, flipCounter);
  
  const userTokenAccount = (tokenMint.toBase58() === NATIVE_MINT.toBase58()) ?
    user.publicKey : (await getAssociatedTokenAddress(tokenMint, user.publicKey));
  let vaultTokenAccount;
  if (tokenMint.toBase58() === NATIVE_MINT.toBase58()) {
    vaultTokenAccount = vaultAuthority;
  } else {
    let [_vaultTokenAccount] = await getVaultTokenAccount(program.programId, tokenMint, admin);
    vaultTokenAccount = _vaultTokenAccount;
  }
  const [allowed, allowedNonce] = await getAllowed(program.programId, tokenMint, admin);

  await program.rpc.bet({
    amount: new anchor.BN(amount),
    betSide,
    flipCounter: new anchor.BN(flipCounter),
    betStateNonce,
    allowedNonce
  }, {
    accounts: {
      coreState,
      allowedBets: allowed,
      user: user.publicKey,
      vaultAuthority,
      tokenMint,
      userTokenAccount,
      vaultTokenAccount,
      betState,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
      rent: SYSVAR_RENT_PUBKEY
    },
    signers: [user]
  });
  return betState;
}

export async function betReturn(admin: Keypair, executer: Keypair, betState: PublicKey) {
  const [coreState] = await getCoreState(program.programId, admin.publicKey);
  const {user, tokenMint } = (await program.account.betState.fetch(betState));
  const [vaultAuthority] = await getVaultAuth(program.programId, admin.publicKey);
  
  const userTokenAccount = (tokenMint.toBase58() === NATIVE_MINT.toBase58()) ?
    user : (await getAssociatedTokenAddress(tokenMint, user));
  let vaultTokenAccount;
  if (tokenMint.toBase58() === NATIVE_MINT.toBase58()) {
    vaultTokenAccount = vaultAuthority;
  } else {
    let [_vaultTokenAccount] = await getVaultTokenAccount(program.programId, tokenMint, admin.publicKey);
    vaultTokenAccount = _vaultTokenAccount;
  }

  await program.rpc.betReturn({
    accounts: {
      admin: admin.publicKey,
      executer: executer.publicKey,
      coreState,
      user,
      vaultAuthority,
      tokenMint,
      userTokenAccount,
      vaultTokenAccount,
      betState,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
      rent: SYSVAR_RENT_PUBKEY
    },
    signers: [executer]
  });
  return betState;
}

export async function updateCoreState(admin: Keypair, feePercent: number, active: boolean, allowDirectBet: boolean) {
  const [coreState] = await getCoreState(program.programId, admin.publicKey);

  await program.rpc.updateCoreState({
    feePercent: new anchor.BN(feePercent * 100),
    active,
    allowDirectBet
  }, {
    accounts: {
      admin: admin.publicKey,
      coreState,
    },
    signers: [admin]
  });
  return coreState;
}
