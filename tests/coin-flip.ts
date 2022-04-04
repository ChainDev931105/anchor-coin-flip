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
  withdraw,
  bet,
  betReturn
} from './coin-flip_instruction';
import {
  getVaultTokenAccount
} from './coin-flip_pda';

describe('coin-flip', () => {
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.CoinFlip as Program<CoinFlip>;
  
  const admin = Keypair.generate();
  const user = Keypair.generate();
  const tokenMintAuthority = Keypair.generate();
  const AIRDROP_AMOUNT = 1_000_000_000;
  const DEPOSIT_AMOUNT = 100_000_000;
  const WITHDRAW_AMOUNT = 50_000_000;
  const BET_AMOUNT = 5_000_000;
  const FEE_PERCENT = 1;
  let vaultAuth;
  let tokenMint;
  let adminTokenAccount;
  let vaultTokenAccount;
  let userTokenAccount;
  let coreStateAddress;

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
    const { coreState, vaultAuthority } = await initialize(admin, FEE_PERCENT);
    coreStateAddress = coreState;
    vaultAuth = vaultAuthority;
    console.log("Core State: ", coreState.toBase58(), await program.account.coreState.fetch(coreStateAddress));
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

  it('Bet Sol', async () => {
    // airdrop to user account
    await program.provider.connection.confirmTransaction(
      await program.provider.connection.requestAirdrop(
        user.publicKey,
        AIRDROP_AMOUNT
      ),
      "confirmed"
    );

    for (let i = 0; i < 10; i++) {
      const balanceBefore = await provider.connection.getBalance(user.publicKey);
  
      let betState = await bet(admin.publicKey, user, NATIVE_MINT, BET_AMOUNT, (i % 2) === 0);
      let betStateFetch = (await program.account.betState.fetch(betState));

      const balanceAfter = await provider.connection.getBalance(user.publicKey);
      
      await betReturn(admin, betState);

      const balanceFinal = await provider.connection.getBalance(user.publicKey);
      console.log("try", i + 1, {balanceBefore, balanceAfter, balanceFinal, betStateFetch});
    }
  });

  it('Bet Spl', async () => {
    // create user token account
    userTokenAccount = await createAssociatedTokenAccount(
      provider.connection,
      user,
      tokenMint,
      user.publicKey
    );

    // mint to user
    await mintTo(
      provider.connection,
      user,
      tokenMint,
      userTokenAccount,
      tokenMintAuthority,
      DEPOSIT_AMOUNT
    );
    for (let i = 0; i < 10; i++) {
      const balanceBefore = parseInt((await provider.connection.getTokenAccountBalance(userTokenAccount)).value.amount);

      let betState = await bet(admin.publicKey, user, tokenMint, BET_AMOUNT, (i % 2) === 0);
      let betStateFetch = (await program.account.betState.fetch(betState));
      
      const balanceAfter = parseInt((await provider.connection.getTokenAccountBalance(userTokenAccount)).value.amount);
      
      await betReturn(admin, betState);

      const balanceFinal = parseInt((await provider.connection.getTokenAccountBalance(userTokenAccount)).value.amount);
      console.log("try", i + 1, {balanceBefore, balanceAfter, balanceFinal, betStateFetch});
    }
  });
});
