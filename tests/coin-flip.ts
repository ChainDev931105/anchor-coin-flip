import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { CoinFlip } from '../target/types/coin_flip';
import { Keypair, SystemProgram } from '@solana/web3.js';
import { createAssociatedTokenAccount, createMint, getAssociatedTokenAddress, getOrCreateAssociatedTokenAccount, mintTo, NATIVE_MINT, TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { expect } from "chai";
import * as assert from 'assert';
import * as chai from 'chai';
import chaiAsPromised from 'chai-as-promised';
import {
  initialize,
  register,
  deposit,
  withdraw,
  betDirectly,
  bet,
  betReturn,
  updateCoreState
} from './coin-flip_instruction';
import {
  getVaultTokenAccount
} from './coin-flip_pda';

chai.use(chaiAsPromised);

describe('coin-flip', () => {
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.CoinFlip as Program<CoinFlip>;
  
  const admin = Keypair.generate();
  const user = Keypair.generate();
  const executer = Keypair.generate();
  const false_executer = Keypair.generate();
  const tokenMintAuthority = Keypair.generate();
  const AIRDROP_AMOUNT = 5_000_000_000;
  const DEPOSIT_AMOUNT = 500_000_000;
  const WITHDRAW_AMOUNT = 250_000_000;
  const BET_AMOUNT = 5_000_000;
  const FEE_PERCENT = 5;
  const WIN_RATIO = 45;
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

    await program.provider.connection.confirmTransaction(
        await program.provider.connection.requestAirdrop(
            executer.publicKey,
            AIRDROP_AMOUNT
        ),
        "confirmed"
    );

    await program.provider.connection.confirmTransaction(
        await program.provider.connection.requestAirdrop(
            false_executer.publicKey,
            AIRDROP_AMOUNT
        ),
        "confirmed"
    );

    // initialize
    const { coreState, vaultAuthority } = await initialize(admin, executer, FEE_PERCENT, WIN_RATIO);
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

  it('Fail bet Sol', async () => {
    // airdrop to user account
    await program.provider.connection.confirmTransaction(
        await program.provider.connection.requestAirdrop(
            user.publicKey,
            AIRDROP_AMOUNT
        ),
        "confirmed"
    );

    const balanceBefore = await provider.connection.getBalance(user.publicKey);

    let betState = await bet(admin.publicKey, user, NATIVE_MINT, BET_AMOUNT, true);

    const balanceAfter = await provider.connection.getBalance(user.publicKey);


    await expect(betReturn(admin, false_executer, betState)).to.be.rejectedWith("Wrong Executer");

    const balanceFinal = await provider.connection.getBalance(user.publicKey);
    console.log("Should fail", {balanceBefore, balanceAfter, balanceFinal, result: balanceBefore > balanceFinal ? "lose" : "win"});
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
      
      await betReturn(admin, executer, betState);

      const balanceFinal = await provider.connection.getBalance(user.publicKey);
      console.log("try", i + 1, {balanceBefore, balanceAfter, balanceFinal, result: balanceBefore > balanceFinal ? "lose" : "win"});
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
      
      await betReturn(admin, executer, betState);

      const balanceFinal = parseInt((await provider.connection.getTokenAccountBalance(userTokenAccount)).value.amount);
      console.log("try", i + 1, {balanceBefore, balanceAfter, balanceFinal, result: balanceBefore > balanceFinal ? "lose" : "win"});
    }
  });

  it('Bet Sol Directly', async () => {
    for (let i = 0; i < 10; i++) {
      const balanceBefore = await provider.connection.getBalance(user.publicKey);
  
      await betDirectly(admin.publicKey, user, NATIVE_MINT, BET_AMOUNT, (i % 2) === 0);

      const balanceAfter = await provider.connection.getBalance(user.publicKey);
      console.log("try", i + 1, {balanceBefore, balanceAfter, result: balanceBefore > balanceAfter ? "lose" : "win"});
    }
  });

  it('Bet Spl Directly', async () => {
    for (let i = 0; i < 10; i++) {
      const balanceBefore = parseInt((await provider.connection.getTokenAccountBalance(userTokenAccount)).value.amount);

      await betDirectly(admin.publicKey, user, tokenMint, BET_AMOUNT, (i % 2) === 0);

      const balanceAfter = parseInt((await provider.connection.getTokenAccountBalance(userTokenAccount)).value.amount);
      console.log("try", i + 1, {balanceBefore, balanceAfter, result: balanceBefore > balanceAfter ? "lose" : "win"});
    }
  });

  it('Update CoreState', async () => {
    const NEW_FEE_PERCENT = 2;
    const coreState = await updateCoreState(admin, NEW_FEE_PERCENT, false, false);

    console.log("Core State: ", coreState.toBase58(), await program.account.coreState.fetch(coreState));
  });
});
