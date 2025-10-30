//! 市场指令：创建市场（含YES mint、元数据、金库ATA等）

use crate::{
    constants::{CONFIG, GLOBAL, MARKET, METADATA, YES_NAME},
    errors::*,
    state::{config::*, market::*},
    events::CreateEvent,
};
use anchor_lang::{prelude::*, solana_program::sysvar::SysvarId, system_program};
use anchor_spl::{
    associated_token::{self, AssociatedToken},
    metadata::{self, mpl_token_metadata::types::DataV2, Metadata},
    token::{self, spl_token::instruction::AuthorityType, Mint, Token},
};

/// 账户集合：创建市场所需账户
#[derive(Accounts)]
pub struct CreateMarket<'info> {
    /// 全局配置
    #[account(
        mut,
        seeds = [CONFIG.as_bytes()],
        bump,
    )]
    global_config: Box<Account<'info, Config>>,

    /// 全局金库（PDA，存放SOL）
    /// CHECK: global vault pda which stores SOL
    #[account(
        mut,
        seeds = [GLOBAL.as_bytes()],
        bump,
    )]
    pub global_vault: AccountInfo<'info>,

    /// 创建者
    #[account(mut)]
    creator: Signer<'info>,

    /// YES代币mint（由全局金库作为mint authority）
    #[account(
        init,
        payer = creator,
        mint::decimals = global_config.token_decimals_config,
        mint::authority = global_vault.key(),
    )]
    yes_token: Box<Account<'info, Mint>>,

    /// NO代币mint（需在mint_no_token指令中创建）
    pub no_token: Box<Account<'info, Mint>>,

    /// 市场账户（以YES/NO mint作为种子）
    #[account(
        init,
        payer = creator,
        space = 8 + std::mem::size_of::<Market>(),
        seeds = [MARKET.as_bytes(), &yes_token.key().to_bytes(), &no_token.key().to_bytes()],
        bump
    )]
    market: Box<Account<'info, Market>>,

    /// YES元数据账户（传递给 Metadata 程序）
    /// CHECK: passed to token metadata program
    #[account(mut,
        seeds = [
            METADATA.as_bytes(),
            metadata::ID.as_ref(),
            yes_token.key().as_ref(),
        ],
        bump,
        seeds::program = metadata::ID
    )]
    yes_token_metadata_account: UncheckedAccount<'info>,

    /// NO元数据账户（传递给 Metadata 程序）
    /// CHECK: passed to token metadata program
    #[account(
        mut,
        seeds = [
            METADATA.as_bytes(),
            metadata::ID.as_ref(),
            no_token.key().as_ref(),
        ],
        bump,
        seeds::program = metadata::ID
    )]
    no_token_metadata_account: UncheckedAccount<'info>,

    /// 全局金库的YES ATA（在指令中创建）
    /// CHECK: created in instruction
    #[account(
        mut,
        seeds = [
            global_vault.key().as_ref(),
            token::spl_token::ID.as_ref(),
            yes_token.key().as_ref(),
        ],
        bump,
        seeds::program = associated_token::ID
    )]
    global_yes_token_account: UncheckedAccount<'info>,

    /// 系统/租金/代币/ATA/元数据程序
    #[account(address = system_program::ID)]
    system_program: Program<'info, System>,
    #[account(address = Rent::id())]
    rent: Sysvar<'info, Rent>,
    #[account(address = token::ID)]
    token_program: Program<'info, Token>,
    #[account(address = associated_token::ID)]
    associated_token_program: Program<'info, AssociatedToken>,
    #[account(address = metadata::ID)]
    mpl_token_metadata_program: Program<'info, Metadata>,

    /// 团队钱包（需与配置一致）
    /// CHECK: should be same with the address in the global_config
    #[account(
        mut,
        constraint = global_config.team_wallet == team_wallet.key() @PredictionMarketError::IncorrectAuthority
    )]
    pub team_wallet: UncheckedAccount<'info>,
}

impl<'info> CreateMarket<'info> {
    pub fn handler(&mut self, params: CreateMarketParams, global_vault_bump: u8) -> Result<()> {
        let global_config = &self.global_config;
        let creator = &self.creator;
        let yes_token: &Box<Account<'info, Mint>> = &self.yes_token;
        let no_token: &Box<Account<'info, Mint>> = &self.no_token;
        let global_vault = &self.global_vault;
        let global_yes_token_account = &self.global_yes_token_account;

        // 1) 创建全局金库的 YES ATA
        associated_token::create(CpiContext::new(
            self.associated_token_program.to_account_info(),
            associated_token::Create {
                payer: creator.to_account_info(),
                associated_token: global_yes_token_account.to_account_info(),
                authority: global_vault.to_account_info(),
                mint: yes_token.to_account_info(),
                token_program: self.token_program.to_account_info(),
                system_program: self.system_program.to_account_info(),
            },
        ))?;

        // PDA签名 seeds（global vault 作为 mint authority）
        let signer_seeds: &[&[&[u8]]] = &[&[
            GLOBAL.as_bytes(),
            &[global_vault_bump],
        ]];

        // 2) 铸造 YES 代币到全局金库 ATA（总供应）
        token::mint_to(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                token::MintTo {
                    mint: yes_token.to_account_info(),
                    to: global_yes_token_account.to_account_info(),
                    authority: global_vault.to_account_info(),
                },
                signer_seeds,
            ),
            global_config.token_supply_config,
        )?;

        // 3) 创建 YES 元数据（NO 元数据在 mint_no_token 指令已创建）
        metadata::create_metadata_accounts_v3(
            CpiContext::new_with_signer(
                self.mpl_token_metadata_program.to_account_info(),
                metadata::CreateMetadataAccountsV3 {
                    metadata: self.yes_token_metadata_account.to_account_info(),
                    mint: yes_token.to_account_info(),
                    mint_authority: global_vault.to_account_info(),
                    payer: creator.to_account_info(),
                    update_authority: global_vault.to_account_info(),
                    system_program: self.system_program.to_account_info(),
                    rent: self.rent.to_account_info(),
                },
                signer_seeds,
            ),
            DataV2 {
                name: YES_NAME.to_string(),
                symbol: params.yes_symbol,
                uri: params.yes_uri,
                seller_fee_basis_points: 0,
                creators: None,
                collection: None,
                uses: None,
            },
            false,
            true,
            None,
        )?;

        // 4) 撤销 YES mint 的铸造权限（禁止再次铸造）
        token::set_authority(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                token::SetAuthority {
                    current_authority: global_vault.to_account_info(),
                    account_or_mint: yes_token.to_account_info(),
                },
                signer_seeds,
            ),
            AuthorityType::MintTokens,
            None,
        )?;

        // 5) 初始化市场账户状态
        let market = &mut self.market;
        market.yes_token_mint = yes_token.key();
        market.no_token_mint = no_token.key();
        market.creator = creator.key();

        // 初始虚拟储备与总供应量（采用 Config 中的 token_supply_config）
        let total_supply = global_config.token_supply_config;
        market.initial_yes_token_reserves = total_supply;
        market.initial_no_token_reserves = total_supply;
        market.token_yes_total_supply = total_supply;
        market.token_no_total_supply = total_supply;

        // 初始真实储备（采用 Config 中的 initial_real_token_reserves_config）
        let initial_real = global_config.initial_real_token_reserves_config;
        market.real_yes_token_reserves = initial_real;
        market.real_no_token_reserves = initial_real;

        // 初始 SOL 储备为 0（后续通过 add_liquidity 注入）
        market.real_yes_sol_reserves = 0;
        market.real_no_sol_reserves = 0;

        // 市场状态
        market.is_completed = false;
        market.start_slot = params.start_slot;
        market.ending_slot = params.ending_slot;

        market.lps = Vec::new();
        market.total_lp_amount = 0;

        // 6) 触发创建事件（Option 转换为 0 表示未设置）
        emit!(CreateEvent {
            creator: creator.key(),
            market: market.key(),
            token_yes: yes_token.key(),
            metadata_yes: self.yes_token_metadata_account.key(),
            token_yes_total_supply: market.token_yes_total_supply,
            real_yes_sol_reserves: market.real_yes_sol_reserves,
            token_no: no_token.key(),
            metadata_no: self.no_token_metadata_account.key(),
            token_no_total_supply: market.token_no_total_supply,
            real_no_sol_reserves: market.real_no_sol_reserves,
            start_slot: market.start_slot.unwrap_or(0),
            ending_slot: market.ending_slot.unwrap_or(0),
        });

        msg!("CreateMarket completed: market={}, yes_mint={}, no_mint={}",
            market.key(), yes_token.key(), no_token.key());

        Ok(())
    }
}
