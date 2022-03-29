import * as anchor from '@project-serum/anchor';
import { PublicKey } from "@solana/web3.js";

const CORE_STATE_SEED: string = "core-state";
const VAULT_AUTH_SEED: string = "vault-auth";

export async function getCoreState(programId: PublicKey, admin: PublicKey) {
  return await anchor.web3.PublicKey.findProgramAddress(
    [
      Buffer.from(anchor.utils.bytes.utf8.encode(CORE_STATE_SEED)),
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
