import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { PublicKey } from "@solana/web3.js";
import { CoinFlip } from '../target/types/coin_flip';

const ALLOWED: string = "allowed";
const CORE_STATE_SEED: string = "core-state";
const VAULT_AUTH_SEED: string = "vault-auth";
const VAULT_TOKEN_ACCOUNT_SEED: string = "vault-token-account";
const BET_STATE_SEED: string = "bet-state";

const program = anchor.workspace.CoinFlip as Program<CoinFlip>;

export async function getCoreState(programId: PublicKey, admin: PublicKey) {
  return await anchor.web3.PublicKey.findProgramAddress(
    [
      Buffer.from(anchor.utils.bytes.utf8.encode(CORE_STATE_SEED)),
      admin.toBuffer()
    ],
    programId
  );
}

export async function getAllowed(programId: PublicKey, mint: PublicKey, admin: PublicKey) {
    return await anchor.web3.PublicKey.findProgramAddress(
        [
            Buffer.from(anchor.utils.bytes.utf8.encode(ALLOWED)),
            mint.toBuffer(),
            admin.toBuffer()
        ],
        programId
    );
}

export async function getVaultAuth(programId: PublicKey, admin: PublicKey) {
  return await anchor.web3.PublicKey.findProgramAddress(
    [
      Buffer.from(anchor.utils.bytes.utf8.encode(VAULT_AUTH_SEED)),
      admin.toBuffer()
    ],
    programId
  );
}

export async function getVaultTokenAccount(programId: PublicKey, tokenMint: PublicKey, admin: PublicKey) {
  return await anchor.web3.PublicKey.findProgramAddress(
    [
      Buffer.from(anchor.utils.bytes.utf8.encode(VAULT_TOKEN_ACCOUNT_SEED)),
      tokenMint.toBuffer(),
      admin.toBuffer()
    ],
    programId
  );
}

export async function getBetState(programId: PublicKey, admin: PublicKey, user: PublicKey) {
  return await anchor.web3.PublicKey.findProgramAddress(
    [
      Buffer.from(anchor.utils.bytes.utf8.encode(BET_STATE_SEED)),
      admin.toBuffer(),
      user.toBuffer()
    ],
    programId
  );
}
