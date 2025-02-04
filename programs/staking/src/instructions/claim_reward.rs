use {
    crate::*,
    
    anchor_spl::token::{self, Token, TokenAccount, Transfer}
};

#[derive(Accounts)]
pub struct ClaimReward<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub user_pool: AccountLoader<'info, UserPool>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump
    )]
    pub global_authority: Account<'info, GlobalPool>,

    #[account(
        mut,
        constraint = reward_vault.mint == REWARD_TOKEN_MINT_PUBKEY.parse::<Pubkey>().unwrap(),
        constraint = reward_vault.owner == global_authority.key(),
    )]
    pub reward_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = user_reward_account.mint == REWARD_TOKEN_MINT_PUBKEY.parse::<Pubkey>().unwrap(),
        constraint = user_reward_account.owner == owner.key(),
    )]
    pub user_reward_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn claim_reward_handler(ctx: Context<ClaimReward>) -> Result<()> {
    let timestamp = Clock::get()?.unix_timestamp;
    let mut user_pool = ctx.accounts.user_pool.load_mut()?;
    let reward: u64 = user_pool.claim_reward_all(timestamp)?;
    msg!("Reward: {}", reward);
    if ctx.accounts.reward_vault.amount < 1000_000_000 + reward {
        return Err(StakingError::LackLamports.into());
    }
    let seeds = &[GLOBAL_AUTHORITY_SEED.as_bytes(), &[ctx.bumps.global_authority]];
    let signer = &[&seeds[..]];
    let token_program = ctx.accounts.token_program.to_account_info();
    let cpi_accounts = Transfer {
        from: ctx.accounts.reward_vault.to_account_info(),
        to: ctx.accounts.user_reward_account.to_account_info(),
        authority: ctx.accounts.global_authority.to_account_info(),
    };
    token::transfer(
        CpiContext::new_with_signer(token_program.clone(), cpi_accounts, signer),
        reward,
    )?;

    Ok(())

}
