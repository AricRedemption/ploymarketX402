// Solana预测市场合约前端集成工具类
import { 
  Program, 
  AnchorProvider, 
  BN,
  web3 
} from '@coral-xyz/anchor';
import { 
  Connection, 
  PublicKey, 
  Keypair,
  SystemProgram,
  SYSVAR_RENT_PUBKEY 
} from '@solana/web3.js';
import { 
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  createAssociatedTokenAccountInstruction 
} from '@solana/spl-token';

// 合约常量
export const PREDICTION_MARKET_PROGRAM_ID = new PublicKey("EgEc7fuse6eQ3UwqeWGFncDtbTwozWCy4piydbeRaNrU");
export const DEVNET_PROGRAM_ID = new PublicKey("EgEc7fuse6eQ3UwqeWGFncDtbTwozWCy4piydbeRaNrU");

// PDA种子常量
export const SEEDS = {
  CONFIG: "config",
  GLOBAL: "global",
  MARKET: "market", 
  USERINFO: "userinfo",
  METADATA: "metadata"
} as const;

// 代币类型枚举
export enum TokenType {
  YES = 0,
  NO = 1
}

// 交易方向枚举
export enum SwapDirection {
  BUY = 0,
  SELL = 1
}

// 接口定义
export interface MarketInfo {
  yesTokenMint: PublicKey;
  noTokenMint: PublicKey;
  creator: PublicKey;
  initialYesTokenReserves: BN;
  realYesTokenReserves: BN;
  realYesSolReserves: BN;
  tokenYesTotalSupply: BN;
  initialNoTokenReserves: BN;
  realNoTokenReserves: BN;
  realNoSolReserves: BN;
  tokenNoTotalSupply: BN;
  isCompleted: boolean;
  startSlot: BN | null;
  endingSlot: BN | null;
  lps: Array<{
    user: PublicKey;
    solAmount: BN;
  }>;
  totalLpAmount: BN;
}

export interface UserInfo {
  user: PublicKey;
  yesBalance: BN;
  noBalance: BN;
  isLp: boolean;
  isInitialized: boolean;
}

export interface CreateMarketParams {
  yesSymbol: string;
  yesUri: string;
  startSlot?: number;
  endingSlot?: number;
}

export interface SwapParams {
  amount: number;
  direction: SwapDirection;
  tokenType: TokenType;
  minimumReceiveAmount: number;
}

export interface LiquidityParams {
  amount: number;
}

// 预测市场客户端类
export class PredictionMarketClient {
  private program: Program<any>;
  private connection: Connection;
  private wallet: web3.Keypair;

  constructor(
    program: Program<any>,
    connection: Connection,
    wallet: web3.Keypair
  ) {
    if (!program) {
      throw new Error('程序实例不能为空');
    }
    this.program = program;
    this.connection = connection;
    this.wallet = wallet;
  }

  // 获取PDA地址
  private getPDA(seeds: Buffer[], programId: PublicKey = this.program.programId): PublicKey {
    const [pda] = PublicKey.findProgramAddressSync(seeds, programId);
    return pda;
  }

  // 获取全局配置PDA
  getGlobalConfigPDA(): PublicKey {
    return this.getPDA([Buffer.from(SEEDS.CONFIG)]);
  }

  // 获取全局金库PDA
  getGlobalVaultPDA(): PublicKey {
    return this.getPDA([Buffer.from(SEEDS.GLOBAL)]);
  }

  // 获取市场PDA
  getMarketPDA(yesTokenMint: PublicKey, noTokenMint: PublicKey): PublicKey {
    return this.getPDA([
      Buffer.from(SEEDS.MARKET),
      yesTokenMint.toBuffer(),
      noTokenMint.toBuffer()
    ]);
  }

  // 获取用户信息PDA
  getUserInfoPDA(marketPDA: PublicKey): PublicKey {
    return this.getPDA([
      Buffer.from(SEEDS.USERINFO),
      this.wallet.publicKey.toBuffer(),
      marketPDA.toBuffer()
    ]);
  }

  // 获取代币元数据PDA
  getTokenMetadataPDA(tokenMint: PublicKey): PublicKey {
    return this.getPDA([
      Buffer.from(SEEDS.METADATA),
      new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").toBuffer(),
      tokenMint.toBuffer()
    ]);
  }

  // 获取全局代币账户PDA
  getGlobalTokenAccountPDA(tokenMint: PublicKey): PublicKey {
    return this.getPDA([
      this.getGlobalVaultPDA().toBuffer(),
      TOKEN_PROGRAM_ID.toBuffer(),
      tokenMint.toBuffer()
    ], ASSOCIATED_TOKEN_PROGRAM_ID);
  }

  // 获取用户代币账户地址
  async getUserTokenAccount(tokenMint: PublicKey): Promise<PublicKey> {
    return await getAssociatedTokenAddress(tokenMint, this.wallet.publicKey);
  }

  // 初始化全局配置
  async initializeConfig(config: {
    authority: PublicKey;
    pendingAuthority: PublicKey;
    teamWallet: PublicKey;
    platformBuyFee: BN;
    platformSellFee: BN;
    lpBuyFee: BN;
    lpSellFee: BN;
    tokenSupplyConfig: BN;
    tokenDecimalsConfig: number;
    initialRealTokenReservesConfig: BN;
    minSolLiquidity: BN;
    initialized: boolean;
  }): Promise<string> {
    const globalConfigPDA = this.getGlobalConfigPDA();
    const globalVaultPDA = this.getGlobalVaultPDA();

    const tx = await (this.program as any).methods
      .configure(config)
      .accounts({
        payer: this.wallet.publicKey,
        config: globalConfigPDA,
        globalVault: globalVaultPDA,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .rpc();

    return tx;
  }

  // 创建市场
  async createMarket(params: CreateMarketParams): Promise<string> {
    const yesTokenKeypair = Keypair.generate();
    const noTokenKeypair = Keypair.generate();
    
    const globalConfigPDA = this.getGlobalConfigPDA();
    const globalVaultPDA = this.getGlobalVaultPDA();
    const marketPDA = this.getMarketPDA(yesTokenKeypair.publicKey, noTokenKeypair.publicKey);
    
    const yesTokenMetadataPDA = this.getTokenMetadataPDA(yesTokenKeypair.publicKey);
    const noTokenMetadataPDA = this.getTokenMetadataPDA(noTokenKeypair.publicKey);
    
    const globalYesTokenAccountPDA = this.getGlobalTokenAccountPDA(yesTokenKeypair.publicKey);
    const globalNoTokenAccountPDA = this.getGlobalTokenAccountPDA(noTokenKeypair.publicKey);

    // 获取团队钱包地址（需要从全局配置中获取）
    const globalConfig = await (this.program.account as any).config.fetch(globalConfigPDA);
    const teamWallet = globalConfig.teamWallet;

    const tx = await (this.program as any).methods
      .createMarket({
        yesSymbol: params.yesSymbol,
        yesUri: params.yesUri,
        startSlot: params.startSlot || null,
        endingSlot: params.endingSlot || null
      })
      .accounts({
        globalConfig: globalConfigPDA,
        globalVault: globalVaultPDA,
        creator: this.wallet.publicKey,
        yesToken: yesTokenKeypair.publicKey,
        noToken: noTokenKeypair.publicKey,
        market: marketPDA,
        yesTokenMetadataAccount: yesTokenMetadataPDA,
        noTokenMetadataAccount: noTokenMetadataPDA,
        globalYesTokenAccount: globalYesTokenAccountPDA,
        teamWallet: teamWallet,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        mplTokenMetadataProgram: new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")
      })
      .signers([yesTokenKeypair, noTokenKeypair])
      .rpc();

    return tx;
  }

  // 铸造NO代币
  async mintNoToken(
    noTokenKeypair: Keypair,
    noSymbol: string,
    noUri: string
  ): Promise<string> {
    const globalConfigPDA = this.getGlobalConfigPDA();
    const globalVaultPDA = this.getGlobalVaultPDA();
    const noTokenMetadataPDA = this.getTokenMetadataPDA(noTokenKeypair.publicKey);
    const globalNoTokenAccountPDA = this.getGlobalTokenAccountPDA(noTokenKeypair.publicKey);

    const tx = await (this.program as any).methods
      .mintNoToken(noSymbol, noUri)
      .accounts({
        globalConfig: globalConfigPDA,
        globalVault: globalVaultPDA,
        creator: this.wallet.publicKey,
        noToken: noTokenKeypair.publicKey,
        noTokenMetadataAccount: noTokenMetadataPDA,
        globalNoTokenAccount: globalNoTokenAccountPDA,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        mplTokenMetadataProgram: new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")
      })
      .signers([noTokenKeypair])
      .rpc();

    return tx;
  }

  // 交易代币
  async swapTokens(
    marketPDA: PublicKey,
    yesTokenMint: PublicKey,
    noTokenMint: PublicKey,
    params: SwapParams
  ): Promise<string> {
    const globalConfigPDA = this.getGlobalConfigPDA();
    const globalVaultPDA = this.getGlobalVaultPDA();
    const userInfoPDA = this.getUserInfoPDA(marketPDA);
    
    const globalYesTokenAccountPDA = this.getGlobalTokenAccountPDA(yesTokenMint);
    const globalNoTokenAccountPDA = this.getGlobalTokenAccountPDA(noTokenMint);
    
    const userYesTokenAccount = await this.getUserTokenAccount(yesTokenMint);
    const userNoTokenAccount = await this.getUserTokenAccount(noTokenMint);

    // 获取团队钱包地址
    const globalConfig = await (this.program.account as any).config.fetch(globalConfigPDA);
    const teamWallet = globalConfig.teamWallet;

    const tx = await (this.program as any).methods
      .swap(
        new BN(params.amount),
        params.direction,
        params.tokenType,
        new BN(params.minimumReceiveAmount)
      )
      .accounts({
        globalConfig: globalConfigPDA,
        teamWallet: teamWallet,
        market: marketPDA,
        globalVault: globalVaultPDA,
        yesToken: yesTokenMint,
        noToken: noTokenMint,
        globalYesAta: globalYesTokenAccountPDA,
        globalNoAta: globalNoTokenAccountPDA,
        userYesAta: userYesTokenAccount,
        userNoAta: userNoTokenAccount,
        userInfo: userInfoPDA,
        user: this.wallet.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID
      })
      .rpc();

    return tx;
  }

  // 添加流动性
  async addLiquidity(
    marketPDA: PublicKey,
    yesTokenMint: PublicKey,
    noTokenMint: PublicKey,
    params: LiquidityParams
  ): Promise<string> {
    const globalConfigPDA = this.getGlobalConfigPDA();
    const globalVaultPDA = this.getGlobalVaultPDA();
    const userInfoPDA = this.getUserInfoPDA(marketPDA);

    // 获取团队钱包地址
    const globalConfig = await (this.program.account as any).config.fetch(globalConfigPDA);
    const teamWallet = globalConfig.teamWallet;

    const tx = await (this.program as any).methods
      .addLiquidity(new BN(params.amount))
      .accounts({
        globalConfig: globalConfigPDA,
        teamWallet: teamWallet,
        market: marketPDA,
        globalVault: globalVaultPDA,
        yesToken: yesTokenMint,
        noToken: noTokenMint,
        userInfo: userInfoPDA,
        user: this.wallet.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID
      })
      .rpc();

    return tx;
  }

  // 提取流动性
  async withdrawLiquidity(
    marketPDA: PublicKey,
    yesTokenMint: PublicKey,
    noTokenMint: PublicKey,
    params: LiquidityParams
  ): Promise<string> {
    const globalConfigPDA = this.getGlobalConfigPDA();
    const globalVaultPDA = this.getGlobalVaultPDA();
    const userInfoPDA = this.getUserInfoPDA(marketPDA);

    // 获取团队钱包地址
    const globalConfig = await (this.program.account as any).config.fetch(globalConfigPDA);
    const teamWallet = globalConfig.teamWallet;

    const tx = await (this.program as any).methods
      .withdrawLiquidity(new BN(params.amount))
      .accounts({
        globalConfig: globalConfigPDA,
        teamWallet: teamWallet,
        market: marketPDA,
        globalVault: globalVaultPDA,
        yesToken: yesTokenMint,
        noToken: noTokenMint,
        userInfo: userInfoPDA,
        user: this.wallet.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID
      })
      .rpc();

    return tx;
  }

  // 市场结算
  async resolveMarket(
    marketPDA: PublicKey,
    yesTokenMint: PublicKey,
    noTokenMint: PublicKey,
    yesAmount: number,
    noAmount: number,
    tokenType: TokenType,
    isCompleted: boolean
  ): Promise<string> {
    const globalConfigPDA = this.getGlobalConfigPDA();
    const globalVaultPDA = this.getGlobalVaultPDA();
    const userInfoPDA = this.getUserInfoPDA(marketPDA);

    const tx = await (this.program as any).methods
      .resolution(
        new BN(yesAmount),
        new BN(noAmount),
        tokenType,
        isCompleted
      )
      .accounts({
        globalConfig: globalConfigPDA,
        market: marketPDA,
        globalVault: globalVaultPDA,
        yesToken: yesTokenMint,
        noToken: noTokenMint,
        userInfo: userInfoPDA,
        user: this.wallet.publicKey,
        authority: this.wallet.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID
      })
      .rpc();

    return tx;
  }

  // 查询市场信息
  async getMarketInfo(marketPDA: PublicKey): Promise<MarketInfo> {
    const marketAccount = await (this.program.account as any).market.fetch(marketPDA);
    return marketAccount as MarketInfo;
  }

  // 查询用户信息
  async getUserInfo(userInfoPDA: PublicKey): Promise<UserInfo | null> {
    try {
      const userInfoAccount = await (this.program.account as any).userInfo.fetch(userInfoPDA);
      return userInfoAccount as UserInfo;
    } catch (error) {
      console.log("用户信息账户不存在:", error);
      return null;
    }
  }

  // 查询全局配置
  async getGlobalConfig() {
    const globalConfigPDA = this.getGlobalConfigPDA();
    return await (this.program.account as any).config.fetch(globalConfigPDA);
  }

  // 计算交易预览
  async getSwapPreview(
    marketPDA: PublicKey,
    amount: number,
    tokenType: TokenType
  ): Promise<{ buyResult?: any; sellResult?: any }> {
    const marketInfo = await this.getMarketInfo(marketPDA);
    
    // 这里可以实现AMM计算逻辑
    // 当前合约中的实现比较简单，实际项目中需要更复杂的AMM算法
    
    return {
      buyResult: {
        tokenAmount: amount,
        changeAmount: amount,
        currentYesReserves: marketInfo.realYesSolReserves.toNumber(),
        currentNoReserves: marketInfo.realNoSolReserves.toNumber(),
        newYesReserves: tokenType === TokenType.YES ? 
          marketInfo.realYesSolReserves.toNumber() + amount : 
          marketInfo.realYesSolReserves.toNumber(),
        newNoReserves: tokenType === TokenType.NO ? 
          marketInfo.realNoSolReserves.toNumber() + amount : 
          marketInfo.realNoSolReserves.toNumber()
      }
    };
  }
}

// 工具函数
export class PredictionMarketUtils {
  // 格式化代币数量
  static formatTokenAmount(amount: BN, decimals: number = 9): string {
    return (amount.toNumber() / Math.pow(10, decimals)).toFixed(decimals);
  }

  // 解析代币数量
  static parseTokenAmount(amount: string, decimals: number = 9): BN {
    return new BN(Math.floor(parseFloat(amount) * Math.pow(10, decimals)));
  }

  // 计算价格
  static calculatePrice(yesReserves: BN, noReserves: BN): { yesPrice: number; noPrice: number } {
    const totalReserves = yesReserves.add(noReserves);
    if (totalReserves.isZero()) {
      return { yesPrice: 0.5, noPrice: 0.5 };
    }
    
    const yesPrice = yesReserves.toNumber() / totalReserves.toNumber();
    const noPrice = noReserves.toNumber() / totalReserves.toNumber();
    
    return { yesPrice, noPrice };
  }

  // 验证市场是否有效
  static isMarketValid(marketInfo: MarketInfo): boolean {
    const now = Date.now() / 1000;
    
    // 检查市场是否已完成
    if (marketInfo.isCompleted) {
      return false;
    }
    
    // 检查结束时间
    if (marketInfo.endingSlot) {
      // 这里需要将slot转换为时间戳进行比较
      // 简化处理，实际需要根据网络参数计算
      return true;
    }
    
    return true;
  }
}
