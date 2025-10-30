//! 市场指令：铸造 NO 代币（创建NO mint、全局ATA与元数据，并撤销铸造权限）

use crate::{
    constants::{CONFIG, GLOBAL, METADATA, NO_NAME},
    state::config::*,
};
use anchor_lang::{prelude::*, solana_program::sysvar::SysvarId, system_program};
use anchor_spl::{
    associated_token::{self, AssociatedToken},
    metadata::{self, mpl_token_metadata::types::DataV2, Metadata},
    token::{self, spl_token::instruction::AuthorityType, Mint, Token},
};

/// 账户集合：铸造NO代币所需账户
#[derive(Accounts)]
pub struct MintNoToken<'info> {
    /// 全局配置
    #[account(
        mut,
        seeds = [CONFIG.as_bytes()],
        bump,
    )]
    global_config: Box<Account<'info, Config>>,

    /// 全局金库（PDA，作为铸造与更新权限）
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

    /// NO代币mint（由全局金库作为mint authority）
    #[account(
        init,
        payer = creator,
        mint::decimals = global_config.token_decimals_config ,
        mint::authority = global_vault.key(),
    )]
    no_token: Box<Account<'info, Mint>>,

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

    /// 全局金库的NO ATA（在指令中创建）
    /// CHECK: created in instruction
    #[account(
        mut,
        seeds = [
            global_vault.key().as_ref(),
            token::spl_token::ID.as_ref(),
            no_token.key().as_ref(),
        ],
        bump,
        seeds::program = associated_token::ID
    )]
    global_no_token_account: UncheckedAccount<'info>,

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
}

impl<'info> MintNoToken<'info> {
    /// 执行NO代币铸造、账户创建、元数据创建与撤销铸造权限
    pub fn handler(
        &mut self,
        // 元数据字段
        no_symbol: String,
        no_uri: String,
        global_vault_bump: u8,
    ) -> Result<()> {
        let global_config = &self.global_config;
        let creator = &self.creator;
        let no_token: &Box<Account<'info, Mint>> = &self.no_token;
        let global_no_token_account = &self.global_no_token_account;
        let global_vault = &self.global_vault;
        let no_name = NO_NAME;

        // 创建全局金库的NO ATA
        associated_token::create(CpiContext::new(
            self.associated_token_program.to_account_info(),
            associated_token::Create {
                payer: creator.to_account_info(),
                associated_token: global_no_token_account.to_account_info(),
                authority: global_vault.to_account_info(),
                mint: no_token.to_account_info(),
                token_program: self.token_program.to_account_info(),
                system_program: self.system_program.to_account_info(),
            },
        ))?;

        let signer_seeds: &[&[&[u8]]] = &[&[GLOBAL.as_bytes(), &[global_vault_bump]]];

        // 铸造NO代币到全局金库ATA
        token::mint_to(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                token::MintTo {
                    mint: no_token.to_account_info(),
                    to: global_no_token_account.to_account_info(),
                    authority: global_vault.to_account_info(),
                },
                signer_seeds,
            ),
            global_config.token_supply_config,
        )?;

        // 创建元数据
        metadata::create_metadata_accounts_v3(
            CpiContext::new_with_signer(
                self.mpl_token_metadata_program.to_account_info(),
                metadata::CreateMetadataAccountsV3 {
                    metadata: self.no_token_metadata_account.to_account_info(),
                    mint: no_token.to_account_info(),
                    mint_authority: global_vault.to_account_info(),
                    payer: creator.to_account_info(),
                    update_authority: global_vault.to_account_info(),
                    system_program: self.system_program.to_account_info(),
                    rent: self.rent.to_account_info(),
                },
                signer_seeds,
            ),
            DataV2 {
                name: no_name.to_string(),
                symbol: no_symbol,
                uri: no_uri,
                seller_fee_basis_points: 0,
                creators: None,
                collection: None,
                uses: None,
            },
            false,
            true,
            None,
        )?;

        // 撤销mint权限，禁止后续铸造
        token::set_authority(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                token::SetAuthority {
                    current_authority: global_vault.to_account_info(),
                    account_or_mint: no_token.to_account_info(),
                },
                signer_seeds,
            ),
            AuthorityType::MintTokens,
            None,
        )?;

        Ok(())
    }
}
