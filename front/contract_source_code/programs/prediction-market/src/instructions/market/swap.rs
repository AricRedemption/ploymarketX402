//! 市场指令：代币交换（买/卖 YES 或 NO）

use crate::{
    constants::{CONFIG, GLOBAL, MARKET, USERINFO},
    errors::PredictionMarketError,
    state::{config::*, market::*},
};
use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    associated_token::{self, AssociatedToken},
    token::{self, Mint, Token},
};

/// 账户集合：交易所需账户
#[derive(Accounts)]
pub struct Swap<'info> {
    /// 全局配置
    #[account(
        mut,
        seeds = [CONFIG.as_bytes()],
        bump,
    )]
    global_config: Box<Account<'info, Config>>,

    /// 团队钱包（收取平台费，需与配置一致）
    /// CHECK: should be same with the address in the global_config
    #[account(
        mut,
        constraint = global_config.team_wallet == team_wallet.key() @PredictionMarketError::IncorrectAuthority
    )]
    pub team_wallet: AccountInfo<'info>,

    /// 市场账户
    #[account(
        mut,
        seeds = [MARKET.as_bytes(), &yes_token.key().to_bytes(), &no_token.key().to_bytes()], 
        bump
    )]
    market: Account<'info, Market>,

    /// 全局金库（PDA，存放SOL）
    /// CHECK: global vault pda which stores SOL
    #[account(
        mut,
        seeds = [GLOBAL.as_bytes()],
        bump,
    )]
    pub global_vault: AccountInfo<'info>,

    /// YES/NO 代币mint
    pub yes_token: Box<Account<'info, Mint>>,
    pub no_token: Box<Account<'info, Mint>>,

    /// 全局金库的YES/NO ATA（按需使用）
    /// CHECK: ata of global vault
    #[account(
        mut,
        seeds = [
            global_vault.key().as_ref(),
            anchor_spl::token::spl_token::ID.as_ref(),
            yes_token.key().as_ref(),
        ],
        bump,
        seeds::program = anchor_spl::associated_token::ID
    )]
    global_yes_ata: AccountInfo<'info>,

     /// CHECK: ata of global vault
     #[account(
        mut,
        seeds = [
            global_vault.key().as_ref(),
            anchor_spl::token::spl_token::ID.as_ref(),
            no_token.key().as_ref(),
        ],
        bump,
        seeds::program = anchor_spl::associated_token::ID
    )]
    global_no_ata: AccountInfo<'info>,

    /// 用户的YES/NO ATA（不存在则创建）
    /// CHECK: ata of user
    #[account(
        mut,
        seeds = [
            user.key().as_ref(),
            anchor_spl::token::spl_token::ID.as_ref(),
            yes_token.key().as_ref(),
        ],
        bump,
        seeds::program = anchor_spl::associated_token::ID
    )]
    user_yes_ata: AccountInfo<'info>,

     /// CHECK: ata of user
     #[account(
        mut,
        seeds = [
            user.key().as_ref(),
            anchor_spl::token::spl_token::ID.as_ref(),
            no_token.key().as_ref(),
        ],
        bump,
        seeds::program = anchor_spl::associated_token::ID
    )]
    user_no_ata: AccountInfo<'info>,

    /// 用户信息（按需初始化）
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + std::mem::size_of::<UserInfo>(),
        seeds = [USERINFO.as_bytes(), &user.key().to_bytes(), &market.key().to_bytes()],
        bump
    )]
    pub user_info: Box<Account<'info, UserInfo>>,

    /// 用户签名者
    #[account(mut)]
    pub user: Signer<'info>,
    
    /// 系统/代币/ATA程序
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
    #[account(address = token::ID)]
    pub token_program: Program<'info, Token>,
    #[account(address = associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Swap<'info> { 
    /// 处理交易：校验市场时间/状态，准备用户账户，委托给 `Market::swap`
    pub fn handler(&mut self, amount: u64, direction: u8, token_type: u8 ,minimum_receive_amount: u64, global_vault_bump:u8) -> Result<()> {
        let market = &mut self.market;
        
        // 校验结束时间
        let clock = Clock::get()?;
        if let Some(ending_slot) = market.ending_slot {
            require!(
                ending_slot >= clock.slot,
                PredictionMarketError::InvalidEndTime
            )
        }

        // 不能在完成后再交易
        require!(
            market.is_completed == false,
            PredictionMarketError::CurveAlreadyCompleted
        );

        let user_info_pda = &mut self.user_info;

        // 初始化用户信息（如未初始化）
        if user_info_pda.is_initialized == false {
            msg!("User info does not exist, initializing...");
            user_info_pda.user = self.user.key();
            user_info_pda.yes_balance = 0;
            user_info_pda.no_balance = 0;
            user_info_pda.is_lp = false;
            user_info_pda.is_initialized = true;
            msg!("User info initialized.");
        } else {
            msg!("User info already exists.");
        }

        let source = &mut self.global_vault.to_account_info();
        let team_wallet = &mut self.team_wallet;

        let yes_token = &mut self.yes_token;
        let user_yes_ata = &mut self.user_yes_ata;

        let no_token = &mut self.no_token;
        let user_no_ata = &mut self.user_no_ata;

        msg!(
            "Swap started. amount: {}, direction: {}, token_type: {}, minimum_receive_amount: {}, global_vault_bump: {}",
            amount,
            direction,
            token_type,
            minimum_receive_amount,
            global_vault_bump
        );

        // 确保用户ATA存在
        if token_type == 0 {
            // NO ATA
            if user_no_ata.data_is_empty() {
                anchor_spl::associated_token::create(CpiContext::new(
                    self.associated_token_program.to_account_info(),
                    anchor_spl::associated_token::Create {
                        payer: self.user.to_account_info(),
                        associated_token: user_no_ata.to_account_info(),
                        authority: self.user.to_account_info(),
                        mint: no_token.to_account_info(),
                        system_program: self.system_program.to_account_info(),
                        token_program: self.token_program.to_account_info(),
                    }
                ))?;
            }
        } else {
            // YES ATA
            if user_yes_ata.data_is_empty() {
                anchor_spl::associated_token::create(CpiContext::new(
                    self.associated_token_program.to_account_info(),
                    anchor_spl::associated_token::Create {
                        payer: self.user.to_account_info(),
                        associated_token: user_yes_ata.to_account_info(),
                        authority: self.user.to_account_info(),
                        mint: yes_token.to_account_info(),
                        system_program: self.system_program.to_account_info(),
                        token_program: self.token_program.to_account_info(),
                    }
                ))?;
            }
        }

        // PDA种子
        let signer_seeds: &[&[&[u8]]] = &[&[
            GLOBAL.as_bytes(),
            &[global_vault_bump],
        ]];

        // 交由市场逻辑处理具体交换
        market.swap(
            &*self.global_config,
            yes_token.as_ref(),
            &mut self.global_yes_ata,
            user_yes_ata,
            no_token.as_ref(),
            &mut self.global_no_ata,
            user_no_ata,
            source,
            team_wallet,
            amount,
            direction,
            token_type,
            minimum_receive_amount,
            &self.user,
            signer_seeds,
            user_info_pda,
            &self.token_program,
            &self.system_program,
        )?;

        Ok(())
    }
}