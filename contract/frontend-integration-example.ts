// 前端对接示例 - Solana 预测市场合约
// 文件: frontend-integration-example.ts

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
  SystemProgram 
} from '@solana/web3.js';
import { 
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  createAssociatedTokenAccountInstruction 
} from '@solana/spl-token';

// 合约配置
export const CONTRACT_CONFIG = {
  programId: new PublicKey("EgEc7fuse6eQ3UwqeWGFncDtbTwozWCy4piydbeRaNrU"),
  network: "devnet",
  rpcUrl: "https://api.devnet.solana.com",
  idlAccount: new PublicKey("4yQt9pFwAtWs7fHNBZFs5nFiDJCJYXPpJ3wPLHkVu7Yv")
};

// PDA种子
export const PDA_SEEDS = {
  CONFIG: "config",
  GLOBAL: "global",
  MARKET: "market", 
  USERINFO: "userinfo",
  METADATA: "metadata"
};

// 代币名称
export const TOKEN_NAMES = {
  YES: "agree",
  NO: "disagree"
};

/**
 * 预测市场客户端类
 */
export class PredictionMarketClient {
  private program: Program;
  private connection: Connection;
  private provider: AnchorProvider;

  constructor(wallet: any) {
    this.connection = new Connection(CONTRACT_CONFIG.rpcUrl);
    this.provider = new AnchorProvider(this.connection, wallet, {});
    
    // 注意：需要导入IDL文件
    // this.program = new Program(idl, CONTRACT_CONFIG.programId, this.provider);
  }

  /**
   * 获取PDA地址
   */
  async getPDA(seeds: string[], programId?: PublicKey): Promise<[PublicKey, number]> {
    return PublicKey.findProgramAddressSync(
      seeds.map(seed => Buffer.from(seed)),
      programId || CONTRACT_CONFIG.programId
    );
  }

  /**
   * 获取全局配置PDA
   */
  async getConfigPDA(): Promise<[PublicKey, number]> {
    return this.getPDA([PDA_SEEDS.CONFIG]);
  }

  /**
   * 获取全局金库PDA
   */
  async getGlobalVaultPDA(): Promise<[PublicKey, number]> {
    return this.getPDA([PDA_SEEDS.GLOBAL]);
  }

  /**
   * 获取市场PDA
   */
  async getMarketPDA(yesToken: PublicKey, noToken: PublicKey): Promise<[PublicKey, number]> {
    return this.getPDA([
      PDA_SEEDS.MARKET,
      yesToken.toBase58(),
      noToken.toBase58()
    ]);
  }

  /**
   * 获取用户信息PDA
   */
  async getUserInfoPDA(user: PublicKey, market: PublicKey): Promise<[PublicKey, number]> {
    return this.getPDA([
      PDA_SEEDS.USERINFO,
      user.toBase58(),
      market.toBase58()
    ]);
  }

  /**
   * 配置全局设置（仅管理员）
   */
  async configure(config: any): Promise<string> {
    const [configPda] = await this.getConfigPDA();
    const [globalVaultPda] = await this.getGlobalVaultPDA();

    const tx = await this.program.methods
      .configure(config)
      .accounts({
        payer: this.provider.wallet.publicKey,
        config: configPda,
        globalVault: globalVaultPda,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: new PublicKey("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL")
      })
      .rpc();

    return tx;
  }

  /**
   * 创建预测市场
   */
  async createMarket(params: {
    yesSymbol: string;
    yesUri: string;
    startSlot?: number;
    endingSlot?: number;
  }): Promise<string> {
    const [configPda] = await this.getConfigPDA();
    const [globalVaultPda] = await this.getGlobalVaultPDA();
    
    // 生成YES代币mint
    const yesTokenMint = Keypair.generate();
    const noTokenMint = Keypair.generate();
    
    const [marketPda] = await this.getMarketPDA(yesTokenMint.publicKey, noTokenMint.publicKey);

    const tx = await this.program.methods
      .createMarket({
        yesSymbol: params.yesSymbol,
        yesUri: params.yesUri,
        startSlot: params.startSlot || null,
        endingSlot: params.endingSlot || null
      })
      .accounts({
        globalConfig: configPda,
        globalVault: globalVaultPda,
        creator: this.provider.wallet.publicKey,
        yesToken: yesTokenMint.publicKey,
        noToken: noTokenMint.publicKey,
        market: marketPda,
        // ... 其他必要账户
      })
      .signers([yesTokenMint, noTokenMint])
      .rpc();

    return tx;
  }

  /**
   * 铸造NO代币
   */
  async mintNoToken(params: {
    noSymbol: string;
    noUri: string;
  }): Promise<string> {
    const [configPda] = await this.getConfigPDA();
    const [globalVaultPda] = await this.getGlobalVaultPDA();
    
    const noTokenMint = Keypair.generate();

    const tx = await this.program.methods
      .mintNoToken(params.noSymbol, params.noUri)
      .accounts({
        globalConfig: configPda,
        globalVault: globalVaultPda,
        creator: this.provider.wallet.publicKey,
        noToken: noTokenMint.publicKey,
        // ... 其他必要账户
      })
      .signers([noTokenMint])
      .rpc();

    return tx;
  }

  /**
   * 交易代币
   */
  async swap(params: {
    amount: number;
    direction: number; // 0=买入, 1=卖出
    tokenType: number; // 0=YES, 1=NO
    minimumReceiveAmount: number;
    yesToken: PublicKey;
    noToken: PublicKey;
  }): Promise<string> {
    const [configPda] = await this.getConfigPDA();
    const [globalVaultPda] = await this.getGlobalVaultPDA();
    const [marketPda] = await this.getMarketPDA(params.yesToken, params.noToken);
    const [userInfoPda] = await this.getUserInfoPDA(
      this.provider.wallet.publicKey, 
      marketPda
    );

    // 获取用户ATA地址
    const userYesAta = await getAssociatedTokenAddress(
      params.yesToken,
      this.provider.wallet.publicKey
    );
    const userNoAta = await getAssociatedTokenAddress(
      params.noToken,
      this.provider.wallet.publicKey
    );

    const tx = await this.program.methods
      .swap(
        new BN(params.amount),
        params.direction,
        params.tokenType,
        new BN(params.minimumReceiveAmount)
      )
      .accounts({
        globalConfig: configPda,
        teamWallet: new PublicKey("7zLK1hCPF5NDY2NZeGL1fwezq4owMpThfjgtTxF34xJr"), // 团队钱包
        market: marketPda,
        globalVault: globalVaultPda,
        yesToken: params.yesToken,
        noToken: params.noToken,
        userInfo: userInfoPda,
        user: this.provider.wallet.publicKey,
        // ... 其他必要账户
      })
      .rpc();

    return tx;
  }

  /**
   * 添加流动性
   */
  async addLiquidity(params: {
    amount: number;
    yesToken: PublicKey;
    noToken: PublicKey;
  }): Promise<string> {
    const [configPda] = await this.getConfigPDA();
    const [globalVaultPda] = await this.getGlobalVaultPDA();
    const [marketPda] = await this.getMarketPDA(params.yesToken, params.noToken);
    const [userInfoPda] = await this.getUserInfoPDA(
      this.provider.wallet.publicKey,
      marketPda
    );

    const tx = await this.program.methods
      .addLiquidity(new BN(params.amount))
      .accounts({
        globalConfig: configPda,
        teamWallet: new PublicKey("7zLK1hCPF5NDY2NZeGL1fwezq4owMpThfjgtTxF34xJr"),
        market: marketPda,
        globalVault: globalVaultPda,
        yesToken: params.yesToken,
        noToken: params.noToken,
        userInfo: userInfoPda,
        user: this.provider.wallet.publicKey,
        // ... 其他必要账户
      })
      .rpc();

    return tx;
  }

  /**
   * 提取流动性
   */
  async withdrawLiquidity(params: {
    amount: number;
    yesToken: PublicKey;
    noToken: PublicKey;
  }): Promise<string> {
    const [configPda] = await this.getConfigPDA();
    const [globalVaultPda] = await this.getGlobalVaultPDA();
    const [marketPda] = await this.getMarketPDA(params.yesToken, params.noToken);
    const [userInfoPda] = await this.getUserInfoPDA(
      this.provider.wallet.publicKey,
      marketPda
    );

    const tx = await this.program.methods
      .withdrawLiquidity(new BN(params.amount))
      .accounts({
        globalConfig: configPda,
        teamWallet: new PublicKey("7zLK1hCPF5NDY2NZeGL1fwezq4owMpThfjgtTxF34xJr"),
        market: marketPda,
        globalVault: globalVaultPda,
        yesToken: params.yesToken,
        noToken: params.noToken,
        userInfo: userInfoPda,
        user: this.provider.wallet.publicKey,
        // ... 其他必要账户
      })
      .rpc();

    return tx;
  }

  /**
   * 市场结算（仅管理员）
   */
  async resolution(params: {
    yesAmount: number;
    noAmount: number;
    tokenType: number;
    isCompleted: boolean;
    yesToken: PublicKey;
    noToken: PublicKey;
  }): Promise<string> {
    const [configPda] = await this.getConfigPDA();
    const [globalVaultPda] = await this.getGlobalVaultPDA();
    const [marketPda] = await this.getMarketPDA(params.yesToken, params.noToken);
    const [userInfoPda] = await this.getUserInfoPDA(
      this.provider.wallet.publicKey,
      marketPda
    );

    const tx = await this.program.methods
      .resolution(
        new BN(params.yesAmount),
        new BN(params.noAmount),
        params.tokenType,
        params.isCompleted
      )
      .accounts({
        globalConfig: configPda,
        market: marketPda,
        globalVault: globalVaultPda,
        yesToken: params.yesToken,
        noToken: params.noToken,
        userInfo: userInfoPda,
        user: this.provider.wallet.publicKey,
        authority: this.provider.wallet.publicKey,
        // ... 其他必要账户
      })
      .rpc();

    return tx;
  }

  /**
   * 监听事件
   */
  async listenToEvents(callback: (event: any) => void) {
    this.program.addEventListener('CreateEvent', callback);
    this.program.addEventListener('TradeEvent', callback);
    this.program.addEventListener('CompleteEvent', callback);
    this.program.addEventListener('WithdrawEvent', callback);
    this.program.addEventListener('GlobalUpdateEvent', callback);
  }
}

// 使用示例
export async function example() {
  // 初始化钱包（这里需要实际的钱包实现）
  const wallet = {}; // 实际的钱包对象
  
  const client = new PredictionMarketClient(wallet);
  
  try {
    // 创建市场
    const createTx = await client.createMarket({
      yesSymbol: "YES",
      yesUri: "https://example.com/metadata"
    });
    console.log("创建市场交易:", createTx);
    
    // 交易代币
    const swapTx = await client.swap({
      amount: 1000000, // 1 SOL (lamports)
      direction: 0, // 买入
      tokenType: 0, // YES代币
      minimumReceiveAmount: 1000000,
      yesToken: new PublicKey("YES_TOKEN_MINT"),
      noToken: new PublicKey("NO_TOKEN_MINT")
    });
    console.log("交易交易:", swapTx);
    
  } catch (error) {
    console.error("交易失败:", error);
  }
}
