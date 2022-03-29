use anchor_lang::prelude::*;

declare_id!("D62yCVSQs6cerJJrfhnpSRVRUVk6HQU72YLmfXjWTzGC");

#[program]
pub mod coin_flip {
    use super::*;
    pub fn initialize(ctx: Context<Initialize>) -> ProgramResult {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
