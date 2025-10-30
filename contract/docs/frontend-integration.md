# Polymarket X402 前端对接文档

**版本**: v1.1.1  
**更新时间**: 2025-10-30  
**合约状态**: ✅ 生产就绪

---

## 📋 目录

1. [快速开始](#快速开始)
2. [环境配置](#环境配置)
3. [核心概念](#核心概念)
4. [客户端 API](#客户端-api)
5. [React Hooks](#react-hooks)
6. [完整流程示例](#完整流程示例)
7. [错误处理](#错误处理)
8. [最佳实践](#最佳实践)
9. [常见问题](#常见问题)

---

## 🚀 快速开始

### 安装依赖

```bash
npm install @coral-xyz/anchor @solana/web3.js @solana/spl-token
# 或
yarn add @coral-xyz/anchor @solana/web3.js @solana/spl-token
```

### 基本使用

```typescript
import { Connection, PublicKey, Keypair } from '@solana/web3.js';
import { AnchorProvider, Program } from '@coral-xyz/anchor';
import { PredictionMarketClient } from './PredictionMarketClient';

// 1. 连接到 Solana
const connection = new Connection('https://api.devnet.solana.com', 'confirmed');

// 2. 加载钱包
const wallet = Keypair.fromSecretKey(/* your secret key */);

// 3. 创建 Provider
const provider = new AnchorProvider(connection, wallet, {});

// 4. 加载程序 IDL
const idl = require('./target/idl/prediction_market.json');
const programId = new PublicKey('EgEc7fuse6eQ3UwqeWGFncDtbTwozWCy4piydbeRaNrU');
const program = new Program(idl, programId, provider);

// 5. 创建客户端
const client = new PredictionMarketClient(program, connection, wallet);

// 6. 开始使用
const marketInfo = await client.getMarketInfo(marketPDA);
console.log('Market info:', marketInfo);
```

---

## ⚙️ 环境配置

### 网络配置

```typescript
// Devnet 配置
const DEVNET_CONFIG = {
  rpcUrl: 'https://api.devnet.solana.com',
  programId: 'EgEc7fuse6eQ3UwqeWGFncDtbTwozWCy4piydbeRaNrU',
  commitment: 'confirmed'
};

// Mainnet 配置 (待部署)
const MAINNET_CONFIG = {
  rpcUrl: 'https://api.mainnet-beta.solana.com',
  programId: 'YOUR_MAINNET_PROGRAM_ID',
  commitment: 'confirmed'
};
```

### USDC 配置

本合约使用 USDC 作为抵押品代币：

```typescript
// USDC Mint 地址
const USDC_MINT = {
  devnet: new PublicKey('EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v'), // Devnet USDC
  mainnet: new PublicKey('EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v') // Mainnet USDC
};

// USDC 精度
const USDC_DECIMALS = 6; // 1 USDC = 10^6 最小单位
```

---

## 💡 核心概念

### 双账本系统

合约采用双账本架构：

1. **Settlement Ledger (结算账本)**
   - 管理条件代币的 1:1 抵押品锁定
   - 用于 `mint_complete_set` / `redeem_complete_set` / `claim_rewards`
   - 字段：`total_collateral_locked`, `total_yes_minted`, `total_no_minted`

2. **AMM Pool Ledger (池子账本)**
   - 管理流动性池的储备金和交易
   - 用于 `add_liquidity` / `withdraw_liquidity` / `swap`
   - 字段：`pool_collateral_reserve`, `pool_yes_reserve`, `pool_no_reserve`

### 条件代币机制

这是 Polymarket 的核心玩法：

```
用户存入 1 USDC → 获得 1 YES + 1 NO
用户销毁 1 YES + 1 NO → 赎回 1 USDC
```

**精度要求**：YES/NO 代币精度必须与 USDC 精度一致（6位）

### LMSR 定价

合约使用 Logarithmic Market Scoring Rule (LMSR) 算法进行价格发现：

- **成本函数**: `C(q) = b * ln(e^(q_yes/b) + e^(q_no/b))`
- **边际价格**: `P(YES) = e^(q_yes/b) / (e^(q_yes/b) + e^(q_no/b))`
- **流动性参数**: `b` 决定市场深度，值越大滑点越小

---

## 🔧 客户端 API

### PredictionMarketClient 类

#### 构造函数

```typescript
constructor(
  program: Program<any>,
  connection: Connection,
  wallet: Keypair
)
```

#### PDA 获取方法

```typescript
// 获取全局配置 PDA
getGlobalConfigPDA(): PublicKey

// 获取全局金库 PDA
getGlobalVaultPDA(): PublicKey

// 获取市场 PDA
getMarketPDA(yesTokenMint: PublicKey, noTokenMint: PublicKey): PublicKey

// 获取用户信息 PDA
getUserInfoPDA(marketPDA: PublicKey): PublicKey

// 获取代币元数据 PDA
getTokenMetadataPDA(tokenMint: PublicKey): PublicKey

// 获取全局代币账户 PDA
getGlobalTokenAccountPDA(tokenMint: PublicKey): PublicKey

// 获取用户代币账户地址
async getUserTokenAccount(tokenMint: PublicKey): Promise<PublicKey>
```

#### 核心指令

##### 1. 初始化全局配置

**管理员专用** - 首次部署时调用

```typescript
async initializeConfig(config: {
  authority: PublicKey;
  pendingAuthority: PublicKey;
  teamWallet: PublicKey;
  platformBuyFee: BN;       // 平台买入手续费（基点，如100=1%）
  platformSellFee: BN;      // 平台卖出手续费
  lpBuyFee: BN;             // LP买入手续费
  lpSellFee: BN;            // LP卖出手续费
  tokenSupplyConfig: BN;    // 代币供应量配置
  tokenDecimalsConfig: number; // 代币精度（必须为6，匹配USDC）
  initialRealTokenReservesConfig: BN;
  minSolLiquidity: BN;      // 最小流动性要求
  initialized: boolean;
}): Promise<string>
```

**示例**：
```typescript
const tx = await client.initializeConfig({
  authority: adminPublicKey,
  pendingAuthority: adminPublicKey,
  teamWallet: teamWalletPublicKey,
  platformBuyFee: new BN(100),  // 1%
  platformSellFee: new BN(100), // 1%
  lpBuyFee: new BN(50),         // 0.5%
  lpSellFee: new BN(50),        // 0.5%
  tokenSupplyConfig: new BN(1_000_000_000_000), // 1M USDC (6位精度)
  tokenDecimalsConfig: 6,       // 必须为6（USDC精度）
  initialRealTokenReservesConfig: new BN(1_000_000_000), // 1000 USDC
  minSolLiquidity: new BN(1_000_000_000), // 1000 USDC
  initialized: true
});
```

##### 2. 创建市场

```typescript
async createMarket(params: {
  yesSymbol: string;    // YES代币符号
  yesUri: string;       // YES代币元数据URI
  startSlot?: number;   // 市场开始槽位（可选）
  endingSlot?: number;  // 市场结束槽位（可选）
}): Promise<string>
```

**示例**：
```typescript
const tx = await client.createMarket({
  yesSymbol: 'BTC100K',
  yesUri: 'https://example.com/metadata/btc100k.json',
  startSlot: undefined,  // 立即开始
  endingSlot: currentSlot + 1_512_000  // ~1周后结束
});
```

##### 3. 铸造完整集合

用户存入 USDC，获得等量的 YES + NO 代币

```typescript
async mintCompleteSet(
  marketPDA: PublicKey,
  yesTokenMint: PublicKey,
  noTokenMint: PublicKey,
  usdcAmount: number  // USDC 数量（6位精度）
): Promise<string>
```

**示例**：
```typescript
// 存入 100 USDC，获得 100 YES + 100 NO
const tx = await client.mintCompleteSet(
  marketPDA,
  yesTokenMint,
  noTokenMint,
  100_000_000  // 100 USDC (100 * 10^6)
);
```

##### 4. 赎回完整集合

销毁等量的 YES + NO 代币，赎回 USDC

```typescript
async redeemCompleteSet(
  marketPDA: PublicKey,
  yesTokenMint: PublicKey,
  noTokenMint: PublicKey,
  amount: number  // 赎回数量
): Promise<string>
```

**示例**：
```typescript
// 销毁 50 YES + 50 NO，赎回 50 USDC
const tx = await client.redeemCompleteSet(
  marketPDA,
  yesTokenMint,
  noTokenMint,
  50_000_000  // 50 * 10^6
);
```

**注意**：只能在市场未完成时使用，市场完成后请使用 `claim_rewards`

##### 5. 交易代币 (Swap)

在 AMM 池中买卖 YES/NO 代币

```typescript
async swapTokens(
  marketPDA: PublicKey,
  yesTokenMint: PublicKey,
  noTokenMint: PublicKey,
  params: {
    amount: number;              // 交易数量
    direction: SwapDirection;    // 0=买入, 1=卖出
    tokenType: TokenType;        // 0=NO, 1=YES
    minimumReceiveAmount: number;// 最小接收数量（滑点保护）
    deadline?: number;           // 交易截止时间戳（可选，0=不检查）
  }
): Promise<string>
```

**示例 - 买入 YES 代币**：
```typescript
const tx = await client.swapTokens(
  marketPDA,
  yesTokenMint,
  noTokenMint,
  {
    amount: 10_000_000,           // 用 10 USDC 购买
    direction: SwapDirection.BUY, // 买入
    tokenType: TokenType.YES,     // YES代币
    minimumReceiveAmount: 9_000_000, // 至少获得 9 YES（10%滑点容忍）
    deadline: Math.floor(Date.now() / 1000) + 60 // 1分钟内有效
  }
);
```

**示例 - 卖出 NO 代币**：
```typescript
const tx = await client.swapTokens(
  marketPDA,
  yesTokenMint,
  noTokenMint,
  {
    amount: 5_000_000,             // 卖出 5 NO
    direction: SwapDirection.SELL, // 卖出
    tokenType: TokenType.NO,       // NO代币
    minimumReceiveAmount: 4_500_000, // 至少获得 4.5 USDC
    deadline: 0 // 不检查截止时间
  }
);
```

##### 6. 添加流动性

向 AMM 池添加 USDC + YES + NO 代币，获得 LP 份额

```typescript
async addLiquidity(
  marketPDA: PublicKey,
  yesTokenMint: PublicKey,
  noTokenMint: PublicKey,
  params: {
    usdcAmount: number;  // USDC 数量
    yesAmount: number;   // YES 代币数量
    noAmount: number;    // NO 代币数量
  }
): Promise<string>
```

**示例**：
```typescript
const tx = await client.addLiquidity(
  marketPDA,
  yesTokenMint,
  noTokenMint,
  {
    usdcAmount: 1000_000_000,  // 1000 USDC
    yesAmount: 500_000_000,    // 500 YES
    noAmount: 500_000_000      // 500 NO
  }
);
```

##### 7. 提取流动性

赎回 LP 份额，获得按比例的 USDC + YES + NO 代币

```typescript
async withdrawLiquidity(
  marketPDA: PublicKey,
  yesTokenMint: PublicKey,
  noTokenMint: PublicKey,
  params: {
    lpSharesToBurn: number;  // 要赎回的 LP 份额数量
  }
): Promise<string>
```

**示例**：
```typescript
const tx = await client.withdrawLiquidity(
  marketPDA,
  yesTokenMint,
  noTokenMint,
  {
    lpSharesToBurn: 100_000_000  // 赎回 100 LP 份额
  }
);
```

##### 8. 市场结算

**管理员专用** - 市场结束后结算结果

```typescript
async resolveMarket(
  marketPDA: PublicKey,
  yesTokenMint: PublicKey,
  noTokenMint: PublicKey,
  yesAmount: number,      // YES代币的赎回比例（基点）
  noAmount: number,       // NO代币的赎回比例（基点）
  tokenType: TokenType,   // 获胜方代币类型
  isCompleted: boolean    // 标记市场为已完成
): Promise<string>
```

**示例 - YES 全胜**：
```typescript
const tx = await client.resolveMarket(
  marketPDA,
  yesTokenMint,
  noTokenMint,
  10000,  // YES = 100% (10000基点 = 100%)
  0,      // NO = 0%
  TokenType.YES,
  true
);
```

**示例 - 平局**：
```typescript
const tx = await client.resolveMarket(
  marketPDA,
  yesTokenMint,
  noTokenMint,
  5000,  // YES = 50%
  5000,  // NO = 50%
  2,     // 平局（不使用 TokenType.YES/NO）
  true
);
```

##### 9. 领取奖励

市场结算后，用户根据持仓领取奖励

```typescript
async claimRewards(
  marketPDA: PublicKey,
  yesTokenMint: PublicKey,
  noTokenMint: PublicKey
): Promise<string>
```

**示例**：
```typescript
// 假设用户持有 100 YES，市场结算 YES 全胜
// 用户将获得 100 USDC
const tx = await client.claimRewards(
  marketPDA,
  yesTokenMint,
  noTokenMint
);
```

#### 查询方法

```typescript
// 查询市场信息
async getMarketInfo(marketPDA: PublicKey): Promise<MarketInfo>

// 查询用户信息
async getUserInfo(userInfoPDA: PublicKey): Promise<UserInfo | null>

// 查询全局配置
async getGlobalConfig(): Promise<Config>

// 计算交易预览
async getSwapPreview(
  marketPDA: PublicKey,
  amount: number,
  tokenType: TokenType
): Promise<{ buyResult?: any; sellResult?: any }>
```

---

## ⚛️ React Hooks

### usePredictionMarket

主要的 React Hook，提供完整的市场操作功能

```typescript
const {
  // 客户端状态
  client,
  connection,
  program,
  isConnected,
  
  // 市场数据
  markets,
  userMarkets,
  userInfo,
  
  // 加载状态
  loading,
  error,
  
  // 操作方法
  initializeConfig,
  createMarket,
  swapTokens,
  addLiquidity,
  withdrawLiquidity,
  resolveMarket,
  
  // 查询方法
  refreshMarkets,
  refreshUserInfo,
  getSwapPreview
} = usePredictionMarket({
  network: 'devnet',
  wallet: keypair
});
```

**完整示例**：
```typescript
import { usePredictionMarket, TokenType, SwapDirection } from './hooks/usePredictionMarket';

function MarketTradingUI() {
  const { 
    client, 
    isConnected, 
    swapTokens, 
    loading, 
    error 
  } = usePredictionMarket({
    network: 'devnet',
    wallet: myWallet
  });
  
  const handleBuy = async () => {
    try {
      const tx = await swapTokens(marketPDA, {
        amount: 10_000_000,
        direction: SwapDirection.BUY,
        tokenType: TokenType.YES,
        minimumReceiveAmount: 9_000_000
      });
      console.log('买入成功:', tx);
    } catch (err) {
      console.error('买入失败:', err);
    }
  };
  
  return (
    <div>
      <button onClick={handleBuy} disabled={loading || !isConnected}>
        {loading ? '处理中...' : '买入 YES'}
      </button>
      {error && <p style={{color: 'red'}}>{error}</p>}
    </div>
  );
}
```

### useMarketInfo

获取单个市场的详细信息

```typescript
const { 
  marketInfo, 
  loading, 
  error, 
  refresh 
} = useMarketInfo(marketPDA);

useEffect(() => {
  if (marketInfo) {
    console.log('YES 储备:', marketInfo.pool_yes_reserve);
    console.log('NO 储备:', marketInfo.pool_no_reserve);
  }
}, [marketInfo]);
```

### useSwapPreview

实时计算交易预览（滑点、价格影响等）

```typescript
const { 
  preview, 
  loading, 
  error 
} = useSwapPreview(
  marketPDA,
  10_000_000,  // 10 USDC
  TokenType.YES
);

if (preview) {
  console.log('预计获得:', preview.tokenAmount);
  console.log('价格影响:', preview.priceImpact);
}
```

---

## 📝 完整流程示例

### 场景 1：用户参与预测市场（买入 YES）

```typescript
import { Connection, Keypair, PublicKey } from '@solana/web3.js';
import { PredictionMarketClient, TokenType, SwapDirection } from './PredictionMarketClient';

async function participateInMarket() {
  // 1. 初始化客户端
  const connection = new Connection('https://api.devnet.solana.com');
  const wallet = Keypair.fromSecretKey(/* ... */);
  const client = new PredictionMarketClient(program, connection, wallet);
  
  // 2. 获取市场信息
  const marketPDA = new PublicKey('YOUR_MARKET_PDA');
  const marketInfo = await client.getMarketInfo(marketPDA);
  
  console.log('市场信息:', {
    yesReserve: marketInfo.pool_yes_reserve,
    noReserve: marketInfo.pool_no_reserve,
    isCompleted: marketInfo.is_completed
  });
  
  // 3. 方案 A：先铸造完整集合（获得 YES + NO）
  const mintTx = await client.mintCompleteSet(
    marketPDA,
    marketInfo.yesTokenMint,
    marketInfo.noTokenMint,
    100_000_000  // 100 USDC → 100 YES + 100 NO
  );
  console.log('铸造交易:', mintTx);
  
  // 4. 卖掉 NO 代币（如果看好 YES）
  const sellNoTx = await client.swapTokens(
    marketPDA,
    marketInfo.yesTokenMint,
    marketInfo.noTokenMint,
    {
      amount: 100_000_000,           // 卖出 100 NO
      direction: SwapDirection.SELL,
      tokenType: TokenType.NO,
      minimumReceiveAmount: 40_000_000  // 至少获得 40 USDC
    }
  );
  console.log('卖出 NO 交易:', sellNoTx);
  
  // 现在用户持有 100 YES（成本 ~60 USDC）
  
  // 5. 方案 B：直接买入 YES（不铸造）
  const buyYesTx = await client.swapTokens(
    marketPDA,
    marketInfo.yesTokenMint,
    marketInfo.noTokenMint,
    {
      amount: 60_000_000,           // 用 60 USDC 购买
      direction: SwapDirection.BUY,
      tokenType: TokenType.YES,
      minimumReceiveAmount: 80_000_000  // 至少获得 80 YES
    }
  );
  console.log('买入 YES 交易:', buyYesTx);
}
```

### 场景 2：LP 提供流动性赚取手续费

```typescript
async function provideLiquidity() {
  const client = new PredictionMarketClient(program, connection, wallet);
  const marketPDA = new PublicKey('YOUR_MARKET_PDA');
  const marketInfo = await client.getMarketInfo(marketPDA);
  
  // 1. 铸造完整集合（获得 YES + NO）
  await client.mintCompleteSet(
    marketPDA,
    marketInfo.yesTokenMint,
    marketInfo.noTokenMint,
    1000_000_000  // 1000 USDC → 1000 YES + 1000 NO
  );
  
  // 2. 添加流动性
  const addLpTx = await client.addLiquidity(
    marketPDA,
    marketInfo.yesTokenMint,
    marketInfo.noTokenMint,
    {
      usdcAmount: 1000_000_000,  // 1000 USDC
      yesAmount: 500_000_000,    // 500 YES
      noAmount: 500_000_000      // 500 NO
    }
  );
  console.log('添加流动性成功:', addLpTx);
  
  // 3. 等待累积手续费...
  
  // 4. 领取 LP 手续费
  const claimFeesTx = await client.claimLpFees(
    marketPDA,
    marketInfo.yesTokenMint,
    marketInfo.noTokenMint
  );
  console.log('领取手续费成功:', claimFeesTx);
  
  // 5. 提取流动性
  const withdrawTx = await client.withdrawLiquidity(
    marketPDA,
    marketInfo.yesTokenMint,
    marketInfo.noTokenMint,
    {
      lpSharesToBurn: 100_000_000  // 提取部分 LP 份额
    }
  );
  console.log('提取流动性成功:', withdrawTx);
}
```

### 场景 3：市场结算后领取奖励

```typescript
async function claimAfterSettlement() {
  const client = new PredictionMarketClient(program, connection, wallet);
  const marketPDA = new PublicKey('YOUR_MARKET_PDA');
  const marketInfo = await client.getMarketInfo(marketPDA);
  
  // 1. 检查市场是否已结算
  if (!marketInfo.is_completed) {
    throw new Error('市场尚未结算');
  }
  
  // 2. 查看结算结果
  console.log('结算结果:', {
    yesRatio: marketInfo.resolution_yes_ratio,  // 基点（10000 = 100%）
    noRatio: marketInfo.resolution_no_ratio,
    winner: marketInfo.winner_token_type
  });
  
  // 3. 查看用户持仓
  const userInfoPDA = client.getUserInfoPDA(marketPDA);
  const userYesAta = await client.getUserTokenAccount(marketInfo.yesTokenMint);
  const userNoAta = await client.getUserTokenAccount(marketInfo.noTokenMint);
  
  const yesBalance = (await connection.getTokenAccountBalance(userYesAta)).value.uiAmount;
  const noBalance = (await connection.getTokenAccountBalance(userNoAta)).value.uiAmount;
  
  console.log('用户持仓:', {
    yes: yesBalance,
    no: noBalance
  });
  
  // 4. 领取奖励
  const claimTx = await client.claimRewards(
    marketPDA,
    marketInfo.yesTokenMint,
    marketInfo.noTokenMint
  );
  console.log('领取奖励成功:', claimTx);
  
  // 5. 计算实际收益
  // 假设 YES 全胜（10000 基点）
  // 用户持有 100 YES → 获得 100 USDC
  // 用户持有 50 NO → 获得 0 USDC
}
```

---

## 🚨 错误处理

### 常见错误码

```typescript
enum PredictionMarketError {
  InvalidAmount = 6000,           // 金额无效
  InsufficientBalance = 6001,     // 余额不足
  InsufficientLiquidity = 6002,   // 流动性不足
  SlippageExceeded = 6003,        // 滑点超限
  MarketNotStarted = 6004,        // 市场未开始
  MarketEnded = 6005,             // 市场已结束
  CurveAlreadyCompleted = 6006,   // 市场已完成
  ContractPaused = 6007,          // 合约已暂停
  InvalidAuthority = 6008,        // 权限无效
  MathOverflow = 6009,            // 数学溢出
  InvalidParameter = 6010,        // 参数无效
  DeadlineExceeded = 6011,        // 交易超时
  // ... 更多错误码请参考 errors.rs
}
```

### 错误处理示例

```typescript
try {
  const tx = await client.swapTokens(marketPDA, params);
  console.log('交易成功:', tx);
} catch (error) {
  if (error.code === 6003) {
    // 滑点超限
    alert('价格变化过大,请调整滑点容忍度');
  } else if (error.code === 6001) {
    // 余额不足
    alert('USDC 余额不足');
  } else if (error.code === 6005) {
    // 市场已结束
    alert('市场已结束,无法交易');
  } else if (error.code === 6011) {
    // 交易超时
    alert('交易已过期,请重新提交');
  } else {
    // 其他错误
    console.error('交易失败:', error);
    alert(`错误: ${error.message}`);
  }
}
```

### 交易确认最佳实践

```typescript
async function sendTransactionWithConfirmation(
  client: PredictionMarketClient,
  txPromise: Promise<string>
) {
  try {
    // 1. 发送交易
    const signature = await txPromise;
    console.log('交易已发送:', signature);
    
    // 2. 等待确认
    const connection = client.connection;
    const confirmation = await connection.confirmTransaction(
      signature,
      'confirmed'  // 或 'finalized' 以获得最终确认
    );
    
    if (confirmation.value.err) {
      throw new Error(`交易失败: ${confirmation.value.err}`);
    }
    
    console.log('交易已确认:', signature);
    return signature;
    
  } catch (error) {
    console.error('交易错误:', error);
    throw error;
  }
}

// 使用示例
await sendTransactionWithConfirmation(
  client,
  client.swapTokens(marketPDA, params)
);
```

---

## ✅ 最佳实践

### 1. 精度处理

**重要**：所有金额必须使用 6 位精度（匹配 USDC）

```typescript
// ❌ 错误：使用浮点数
const amount = 10.5;  // 不精确

// ✅ 正确：使用最小单位（lamports）
const amount = 10_500_000;  // 10.5 USDC = 10.5 * 10^6

// 工具函数
function toUsdcLamports(usdcAmount: number): number {
  return Math.floor(usdcAmount * 1_000_000);
}

function fromUsdcLamports(lamports: number): number {
  return lamports / 1_000_000;
}

// 使用
const userInput = 10.5;  // 用户输入 10.5 USDC
const lamports = toUsdcLamports(userInput);  // 10_500_000
const tx = await client.swapTokens(marketPDA, {
  amount: lamports,
  ...
});
```

### 2. 滑点保护

```typescript
// 计算最小接收数量（容忍 1% 滑点）
function calculateMinimumReceive(
  expectedAmount: number,
  slippageTolerance: number = 0.01  // 1%
): number {
  return Math.floor(expectedAmount * (1 - slippageTolerance));
}

// 使用
const expectedYes = 100_000_000;  // 预期获得 100 YES
const minReceive = calculateMinimumReceive(expectedYes, 0.01);  // 99 YES

await client.swapTokens(marketPDA, {
  amount: 60_000_000,
  direction: SwapDirection.BUY,
  tokenType: TokenType.YES,
  minimumReceiveAmount: minReceive  // 滑点保护
});
```

### 3. 交易截止时间

```typescript
// 设置 1 分钟有效期
const deadline = Math.floor(Date.now() / 1000) + 60;

await client.swapTokens(marketPDA, {
  amount: 10_000_000,
  direction: SwapDirection.BUY,
  tokenType: TokenType.YES,
  minimumReceiveAmount: 9_000_000,
  deadline: deadline  // Unix 时间戳
});
```

### 4. Gas 费优化

```typescript
// 批量操作：先铸造，再卖出（2个交易）
// vs 直接买入（1个交易）

// 方案 A：铸造 + 卖出（成本更低，但需要2笔交易）
await client.mintCompleteSet(marketPDA, mint, mint, 100_000_000);
await client.swapTokens(marketPDA, mint, mint, {
  amount: 100_000_000,
  direction: SwapDirection.SELL,
  tokenType: TokenType.NO,
  minimumReceiveAmount: 40_000_000
});

// 方案 B：直接买入（更快，但可能成本更高）
await client.swapTokens(marketPDA, mint, mint, {
  amount: 60_000_000,
  direction: SwapDirection.BUY,
  tokenType: TokenType.YES,
  minimumReceiveAmount: 90_000_000
});

// 选择依据：比较 Gas 费 + 价格影响
```

### 5. 市场状态检查

```typescript
async function canTrade(
  client: PredictionMarketClient,
  marketPDA: PublicKey
): Promise<boolean> {
  const marketInfo = await client.getMarketInfo(marketPDA);
  const currentSlot = await client.connection.getSlot();
  
  // 检查市场是否完成
  if (marketInfo.is_completed) {
    return false;
  }
  
  // 检查市场是否开始
  if (marketInfo.start_slot && currentSlot < marketInfo.start_slot) {
    return false;
  }
  
  // 检查市场是否结束
  if (marketInfo.ending_slot && currentSlot >= marketInfo.ending_slot) {
    return false;
  }
  
  return true;
}

// 使用
if (await canTrade(client, marketPDA)) {
  await client.swapTokens(...);
} else {
  alert('市场当前不可交易');
}
```

### 6. ATA 初始化

用户首次参与市场时需要初始化 ATA（Associated Token Account）：

```typescript
import { getAssociatedTokenAddress, createAssociatedTokenAccountInstruction } from '@solana/spl-token';

async function ensureUserAta(
  connection: Connection,
  user: PublicKey,
  tokenMint: PublicKey,
  payer: Keypair
): Promise<PublicKey> {
  const ata = await getAssociatedTokenAddress(tokenMint, user);
  
  // 检查 ATA 是否存在
  const accountInfo = await connection.getAccountInfo(ata);
  if (!accountInfo) {
    // 创建 ATA
    const ix = createAssociatedTokenAccountInstruction(
      payer.publicKey,  // 支付者
      ata,              // ATA 地址
      user,             // 所有者
      tokenMint         // 代币 mint
    );
    
    const tx = new Transaction().add(ix);
    await connection.sendTransaction(tx, [payer]);
    console.log('创建 ATA:', ata.toBase58());
  }
  
  return ata;
}
```

---

## ❓ 常见问题

### Q1: 为什么代币精度必须是 6？

**A**: 本合约使用 USDC 作为抵押品（6位精度），YES/NO 代币必须与抵押品精度一致以确保 1:1 套保机制正确运作。如果使用 9 位精度，1 USDC（10^6）铸造的代币数量会是 1000000，而不是预期的 1（导致 1000 倍错误）。

### Q2: mint_complete_set 和直接 swap 买入的区别？

**A**:
- **mint_complete_set**: 1 USDC → 1 YES + 1 NO（无滑点，1:1兑换）
- **swap**: 使用 LMSR 定价，价格根据池子储备动态变化（有滑点）

**套利策略**: 当 YES 价格 > 0.5 USDC 时，可以 mint 获得 YES + NO，然后卖出 YES 获利。

### Q3: 市场完成后如何操作？

**A**:
1. **不能再 swap** - 市场已关闭交易
2. **不能 redeem_complete_set** - 应该用 claim_rewards 领取奖励
3. **必须 claim_rewards** - 根据结算比例领取 USDC
4. **LP 提取** - 必须先调用 settle_pool，然后 withdraw_liquidity

### Q4: 如何计算当前 YES/NO 价格？

**A**:
```typescript
async function getCurrentPrices(
  client: PredictionMarketClient,
  marketPDA: PublicKey
): Promise<{ yesPrice: number; noPrice: number }> {
  const marketInfo = await client.getMarketInfo(marketPDA);
  
  // LMSR 边际价格公式
  // P(YES) = e^(q_yes/b) / (e^(q_yes/b) + e^(q_no/b))
  
  const b = marketInfo.lmsr_b;
  const qYes = marketInfo.lmsr_q_yes;
  const qNo = marketInfo.lmsr_q_no;
  
  const expYes = Math.exp(qYes / b);
  const expNo = Math.exp(qNo / b);
  const sum = expYes + expNo;
  
  return {
    yesPrice: expYes / sum,
    noPrice: expNo / sum
  };
}

// 使用
const prices = await getCurrentPrices(client, marketPDA);
console.log(`YES: ${(prices.yesPrice * 100).toFixed(2)}%`);
console.log(`NO: ${(prices.noPrice * 100).toFixed(2)}%`);
```

### Q5: 如何处理交易失败？

**A**: 参考上文 [错误处理](#错误处理) 章节，主要策略：
1. 捕获特定错误码
2. 提供友好的错误提示
3. 允许用户调整参数重试
4. 记录错误日志供调试

### Q6: LP 手续费如何分配？

**A**: 合约使用 `fee_per_share_cumulative` 机制公平分配：
- 每次 swap 时，LP 手续费累加到 `accumulated_lp_fees`
- 同时更新 `fee_per_share_cumulative += lp_fee / total_lp_shares`
- LP 领取时，根据其份额和上次领取时的 `fee_per_share` 计算未领取费用
- 防止了后来的 LP "搭便车"领取早期手续费

### Q7: 如何监听市场事件？

**A**:
```typescript
// 订阅程序日志
const programId = new PublicKey('EgEc7fuse6eQ3UwqeWGFncDtbTwozWCy4piydbeRaNrU');

connection.onLogs(
  programId,
  (logs) => {
    console.log('收到日志:', logs);
    
    // 解析事件
    if (logs.logs.some(log => log.includes('SwapEvent'))) {
      console.log('检测到交易事件');
      // 刷新市场数据
    }
  },
  'confirmed'
);

// 订阅账户变化
connection.onAccountChange(
  marketPDA,
  (accountInfo) => {
    console.log('市场账户已更新');
    // 重新解析市场数据
  },
  'confirmed'
);
```

### Q8: 支持哪些钱包？

**A**: 合约支持所有兼容 Solana 标准的钱包：
- Phantom
- Solflare
- Backpack
- Ledger
- 等

前端集成示例：
```typescript
import { useWallet } from '@solana/wallet-adapter-react';

function MyComponent() {
  const { publicKey, signTransaction } = useWallet();
  
  // 使用 wallet adapter 代替 Keypair
  const provider = new AnchorProvider(
    connection,
    wallet,  // wallet adapter 实例
    {}
  );
  
  // ... 其他逻辑
}
```

---

## 📚 附录

### 数据结构定义

#### MarketInfo
```typescript
interface MarketInfo {
  // 代币 Mint
  yesTokenMint: PublicKey;
  noTokenMint: PublicKey;
  creator: PublicKey;
  
  // Settlement Ledger（结算账本）
  total_collateral_locked: number;  // 锁定的 USDC 抵押品总量
  total_yes_minted: number;         // 铸造的 YES 总量
  total_no_minted: number;          // 铸造的 NO 总量
  
  // AMM Pool Ledger（池子账本）
  pool_collateral_reserve: number;  // 池子中的 USDC 储备
  pool_yes_reserve: number;         // 池子中的 YES 储备
  pool_no_reserve: number;          // 池子中的 NO 储备
  total_lp_shares: number;          // LP 总份额
  
  // LMSR 参数
  lmsr_b: number;                   // 流动性参数
  lmsr_q_yes: number;               // YES 净持仓量
  lmsr_q_no: number;                // NO 净持仓量
  
  // 市场状态
  is_completed: boolean;
  start_slot: number | null;
  ending_slot: number | null;
  
  // 结算参数
  resolution_yes_ratio: number;     // YES 赎回比例（基点）
  resolution_no_ratio: number;      // NO 赎回比例（基点）
  winner_token_type: number;        // 获胜方（0=NO, 1=YES, 2=平局）
  
  // LP 费用
  accumulated_lp_fees: number;
  fee_per_share_cumulative: bigint; // u128，10^18 精度
}
```

#### UserInfo
```typescript
interface UserInfo {
  user: PublicKey;
  is_lp: boolean;
  is_initialized: boolean;
  // 注意：余额由 SPL Token ATA 追踪，不在此结构中
}
```

#### Config
```typescript
interface Config {
  authority: PublicKey;
  pending_authority: PublicKey;
  team_wallet: PublicKey;
  usdc_mint: PublicKey;             // USDC Mint 地址
  platform_buy_fee: number;         // 平台买入费（基点）
  platform_sell_fee: number;        // 平台卖出费（基点）
  lp_buy_fee: number;               // LP 买入费（基点）
  lp_sell_fee: number;              // LP 卖出费（基点）
  token_supply_config: number;
  token_decimals_config: number;    // 必须为 6
  initial_real_token_reserves_config: number;
  min_sol_liquidity: number;
  usdc_vault_min_balance: number;   // USDC 金库最小余额
  is_paused: boolean;
  whitelist_enabled: boolean;       // 是否启用白名单
  initialized: boolean;
}
```

### 枚举定义

```typescript
enum TokenType {
  NO = 0,
  YES = 1
}

enum SwapDirection {
  BUY = 0,
  SELL = 1
}
```

### PDA 种子常量

```typescript
const SEEDS = {
  CONFIG: 'config',
  GLOBAL: 'global',
  MARKET: 'market',
  USERINFO: 'userinfo',
  LPPOSITION: 'lp_position',
  WHITELIST: 'prediction_market_creator_whitelist',
  METADATA: 'metadata'
};
```

### 程序 ID

```typescript
// Devnet
const PROGRAM_ID = new PublicKey('EgEc7fuse6eQ3UwqeWGFncDtbTwozWCy4piydbeRaNrU');

// Mainnet (待部署)
const MAINNET_PROGRAM_ID = new PublicKey('YOUR_MAINNET_PROGRAM_ID');
```

---

## 📞 技术支持

如有问题，请通过以下方式联系：

- **GitHub Issues**: [项目仓库](https://github.com/your-repo)
- **Discord**: [社区频道](https://discord.gg/your-invite)
- **邮箱**: support@example.com

---

**最后更新**: 2025-10-30  
**文档版本**: v1.0.0  
**合约版本**: v1.1.1

