import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { CoinFlip } from '../target/types/coin_flip';
import { Keypair, SystemProgram } from '@solana/web3.js';
import {
  initialize,
  deposit,
  withdraw
} from './coin-flip_instruction';

describe('coin-flip', () => {
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.CoinFlip as Program<CoinFlip>;

  const admin = Keypair.generate();
  let vaultAuth;

  it('Is initialized!', async () => {
    // airdrop to admin account
    await program.provider.connection.confirmTransaction(
      await program.provider.connection.requestAirdrop(
        admin.publicKey,
        1_000_000_000
      ),
      "confirmed"
    );

    // initialize
    const { coreState, vaultAuthority } = await initialize(admin);
    vaultAuth = vaultAuthority;
    console.log("Core State: ", await program.account.coreState.fetch(coreState));
    console.log("Vault Authority: ", vaultAuthority.toBase58());
  });

  it('Deposit', async () => {
    const amount = 100_000_000;
    const balanceBefore = await provider.connection.getBalance(vaultAuth);
    
    await deposit(admin, amount);

    const balanceAfter = await provider.connection.getBalance(vaultAuth);

    console.log({balanceBefore, balanceAfter});
  });

  it('Withdraw Sol', async () => {
    const amount = 50_000_000;
    const balanceBefore = await provider.connection.getBalance(vaultAuth);
    
    await withdraw(admin, amount);

    const balanceAfter = await provider.connection.getBalance(vaultAuth);

    console.log({balanceBefore, balanceAfter});
  });
});
