import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { CoinFlip } from '../target/types/coin_flip';
import { Keypair, SystemProgram } from '@solana/web3.js';
import { createAssociatedTokenAccount, createMint, getAssociatedTokenAddress, getOrCreateAssociatedTokenAccount, mintTo, NATIVE_MINT } from '@solana/spl-token';
import { expect } from "chai";
import {
  initialize,
  register,
  deposit,
  withdraw
} from './coin-flip_instruction';
import {
  getVaultTokenAccount
} from './coin-flip_pda';

describe('coin-flip', () => {
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.CoinFlip as Program<CoinFlip>;
  
  const admin = Keypair.generate();
  const tokenMintAuthority = Keypair.generate();
  const AIRDROP_AMOUNT = 1000_000_000;
  const DEPOSIT_AMOUNT = 100_000_000;
  const WITHDRAW_AMOUNT = 50_000_000;
  let vaultAuth;
  let tokenMint;
  let adminTokenAccount;
  let vaultTokenAccount;

  it('Is initialized!', async () => {
    // airdrop to admin account
    await program.provider.connection.confirmTransaction(
      await program.provider.connection.requestAirdrop(
        admin.publicKey,
        AIRDROP_AMOUNT
      ),
      "confirmed"
    );

    // initialize
    const { coreState, vaultAuthority } = await initialize(admin);
    vaultAuth = vaultAuthority;
    console.log("Core State: ", await program.account.coreState.fetch(coreState));
    console.log("Vault Authority: ", vaultAuthority.toBase58());
  });

  it('Deposit Sol', async () => {
    const balanceBefore = await provider.connection.getBalance(vaultAuth);
    
    await deposit(admin, NATIVE_MINT, DEPOSIT_AMOUNT);

    const balanceAfter = await provider.connection.getBalance(vaultAuth);
    expect(balanceAfter - balanceBefore).to.equal(DEPOSIT_AMOUNT);
  });

  it('Register Spl', async () => {
    // mint a new token
    tokenMint = await createMint(
      provider.connection,
      admin,
      tokenMintAuthority.publicKey,
      tokenMintAuthority.publicKey,
      9
    );

    await register(admin, tokenMint);
  });

  it('Deposit Spl', async () => {
    // create admin token account
    adminTokenAccount = await createAssociatedTokenAccount(
      provider.connection,
      admin,
      tokenMint,
      admin.publicKey
    );

    // mint to admin
    await mintTo(
      provider.connection,
      admin,
      tokenMint,
      adminTokenAccount,
      tokenMintAuthority,
      DEPOSIT_AMOUNT
    );

    let [_vaultTokenAccount, _vaultTokenAccountNonce] = await getVaultTokenAccount(
      program.programId, tokenMint, admin.publicKey
    );

    vaultTokenAccount = _vaultTokenAccount;
    const balanceBefore = parseInt((await provider.connection.getTokenAccountBalance(vaultTokenAccount)).value.amount);
    
    await deposit(admin, tokenMint, DEPOSIT_AMOUNT);

    const balanceAfter = parseInt((await provider.connection.getTokenAccountBalance(vaultTokenAccount)).value.amount);
    expect(balanceAfter - balanceBefore).to.equal(DEPOSIT_AMOUNT);
  });

  it('Withdraw Sol', async () => {
    const balanceBefore = await provider.connection.getBalance(vaultAuth);
    
    await withdraw(admin, NATIVE_MINT, WITHDRAW_AMOUNT);

    const balanceAfter = await provider.connection.getBalance(vaultAuth);
    expect(balanceBefore - balanceAfter).to.equal(WITHDRAW_AMOUNT);
  });

  it('Withdraw Spl', async () => {
    const balanceBefore = parseInt((await provider.connection.getTokenAccountBalance(vaultTokenAccount)).value.amount);
    
    await withdraw(admin, tokenMint, WITHDRAW_AMOUNT);

    const balanceAfter = parseInt((await provider.connection.getTokenAccountBalance(vaultTokenAccount)).value.amount);
    expect(balanceBefore - balanceAfter).to.equal(WITHDRAW_AMOUNT);
  });
});
