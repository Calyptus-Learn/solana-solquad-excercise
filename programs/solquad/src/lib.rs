use anchor_lang::prelude::*;

declare_id!("5sFUqUTjAMJARrEafMX8f4J1LagdUQ9Y8TR8HwGNHkU8");

#[program]
pub mod solquad {
    use super::*;

    pub fn initialize_escrow(ctx: Context<InitializeEscrow>, amount: u64) -> Result<()> {
        let escrow_account = &mut ctx.accounts.escrow_account;
        escrow_account.escrow_creator = ctx.accounts.escrow_signer.key();
        escrow_account.creator_deposit_amount = amount;
        escrow_account.total_projects = 0;

        Ok(())
    }

    pub fn initialize_pool(ctx: Context<InitializePool>) -> Result<()> {
        let pool_account = &mut ctx.accounts.pool_account;
        pool_account.pool_creator = ctx.accounts.pool_signer.key();
        pool_account.total_projects = 0;
        pool_account.total_votes = 0;

        Ok(())
    }

    pub fn initialize_project(ctx: Context<InitializeProject>, name: String) -> Result<()> {
        let project_account = &mut ctx.accounts.project_account;

        project_account.project_owner = ctx.accounts.project_owner.key();
        project_account.project_name = name;
        project_account.votes_count = 0;
        project_account.voter_amount = 0;
        project_account.distributed_amt = 0;

        Ok(())
    }

    pub fn add_project_to_pool(ctx: Context<AddProjectToPool>) -> Result<()> {
        let escrow_account = &mut ctx.accounts.escrow_account;
        let pool_account = &mut ctx.accounts.pool_account;
        let project_account = &ctx.accounts.project_account;

        pool_account.projects.push(
            project_account.project_owner
        );
        pool_account.total_projects += 1;

        escrow_account.project_reciever_addresses.push(
            project_account.project_owner
        );

        Ok(())
    }

    pub fn vote_for_project(ctx: Context<VoteForProject>, amount: u64) -> Result<()> {
        let pool_account = &mut ctx.accounts.pool_account;
        let project_account = &mut ctx.accounts.project_account;

        for i in 0..pool_account.projects.len() {
            if pool_account.projects[i] == project_account.project_owner {
                project_account.votes_count += 1;
                project_account.voter_amount += amount;
            }
        }

        pool_account.total_votes += 1;

        Ok(())
    }

    pub fn distribute_escrow_amount(ctx: Context<DistributeEscrowAmount>) -> Result<()> {
        let escrow_account = &mut ctx.accounts.escrow_account;
        let pool_account = &mut ctx.accounts.pool_account;
        let project_account = &mut ctx.accounts.project_account;
  
        for i in 0..escrow_account.project_reciever_addresses.len() {
            let distributable_amt: u64;
            let votes: u64;

            let project = pool_account.projects[i];
            if project == project_account.project_owner {
                votes = project_account.votes_count;
            } else {
                votes = 0;
            }

            if votes != 0 {
                distributable_amt = (votes / pool_account.total_votes) * escrow_account.creator_deposit_amount as u64;
            } else {
                distributable_amt = 0;
            }

            project_account.distributed_amt = distributable_amt;
        }

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeEscrow<'info> {
    #[account(
        init,
        payer = escrow_signer,
        space = 1024,
        seeds = [b"escrow".as_ref(), escrow_signer.key().as_ref()],
        bump,
    )]
    pub escrow_account: Account<'info, Escrow>,
    #[account(mut)]
    pub escrow_signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(
        init,
        payer = pool_signer,
        space = 1024,
        seeds = [b"pool".as_ref(), pool_signer.key().as_ref()],
        bump,
    )]
    pub pool_account: Account<'info, Pool>,
    #[account(mut)]
    pub pool_signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeProject<'info> {
    #[account(
        init,
        payer = project_owner,
        space = 32 + 32 + 8 + 8 + 8 + 8,
        seeds = [b"project".as_ref(), pool_account.key().as_ref(), project_owner.key().as_ref()],
        bump,
    )]
    pub project_account: Account<'info, Project>,
    #[account(mut)]
    pub project_owner: Signer<'info>,
    pub pool_account: Account<'info, Pool>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddProjectToPool<'info> {
    #[account(mut)]
    pub escrow_account: Account<'info, Escrow>,
    #[account(mut)]
    pub pool_account: Account<'info, Pool>,
    pub project_account: Account<'info, Project>,
    pub project_owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct VoteForProject<'info> {
    #[account(mut)]
    pub pool_account: Account<'info, Pool>,
    #[account(mut)]
    pub project_account: Account<'info, Project>,
    #[account(mut)]
    pub voter_sig: Signer<'info>,
}

#[derive(Accounts)]
pub struct DistributeEscrowAmount<'info> {
    #[account(mut)]
    pub escrow_creator: Signer<'info>,
    #[account(mut, has_one = escrow_creator)]
    pub escrow_account: Account<'info, Escrow>,
    #[account(mut)]
    pub pool_account: Account<'info, Pool>,
    #[account(mut)]
    pub project_account: Account<'info, Project>,
}

// Escrow account for quadratic funding
#[account]
pub struct Escrow {
    pub escrow_creator: Pubkey,
    pub creator_deposit_amount: u64,
    pub total_projects: u8,
    pub project_reciever_addresses: Vec<Pubkey>,
}

// Pool for each project 
#[account]
pub struct Pool {
    pub pool_creator: Pubkey,
    pub projects: Vec<Pubkey>,
    pub total_projects: u8,
    pub total_votes: u64,
}

// Projects in each pool
#[account]
pub struct Project {
    pub project_owner: Pubkey,
    pub project_name: String,
    pub votes_count: u64,
    pub voter_amount: u64,
    pub distributed_amt: u64,
}

// Voters voting for the project
#[account]
pub struct Voter {
    pub voter: Pubkey,
    pub voted_for: Pubkey,
    pub token_amount: u64
}