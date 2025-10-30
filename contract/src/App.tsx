// 完整的集成示例 - Next.js应用
import React, { useState, useEffect } from 'react';
import { Connection, PublicKey, Keypair } from '@solana/web3.js';
import { Program, AnchorProvider, BN } from '@coral-xyz/anchor';
import { PredictionMarketClient } from './PredictionMarketClient';
import { PredictionMarketApp } from './components/PredictionMarketComponents';

// 网络配置
const NETWORK_CONFIG = {
  devnet: {
    rpcUrl: 'https://api.devnet.solana.com',
    programId: 'EgEc7fuse6eQ3UwqeWGFncDtbTwozWCy4piydbeRaNrU'
  },
  mainnet: {
    rpcUrl: 'https://api.mainnet-beta.solana.com',
    programId: 'EgEc7fuse6eQ3UwqeWGFncDtbTwozWCy4piydbeRaNrU'
  }
};

// IDL接口定义 (需要从合约生成)
const IDL = {
  "version": "0.1.0",
  "name": "prediction_market",
  "instructions": [
    {
      "name": "configure",
      "accounts": [
        { "name": "globalConfig", "isMut": true, "isSigner": false },
        { "name": "authority", "isMut": false, "isSigner": true }
      ],
      "args": [
        { "name": "newConfig", "type": { "defined": "Config" } }
      ]
    },
    {
      "name": "createMarket",
      "accounts": [
        { "name": "globalConfig", "isMut": true, "isSigner": false },
        { "name": "globalVault", "isMut": true, "isSigner": false },
        { "name": "creator", "isMut": true, "isSigner": true },
        { "name": "yesToken", "isMut": true, "isSigner": true },
        { "name": "noToken", "isMut": true, "isSigner": false },
        { "name": "market", "isMut": true, "isSigner": false },
        { "name": "yesTokenMetadataAccount", "isMut": true, "isSigner": false },
        { "name": "noTokenMetadataAccount", "isMut": true, "isSigner": false },
        { "name": "globalYesTokenAccount", "isMut": true, "isSigner": false },
        { "name": "teamWallet", "isMut": true, "isSigner": false },
        { "name": "systemProgram", "isMut": false, "isSigner": false },
        { "name": "rent", "isMut": false, "isSigner": false },
        { "name": "tokenProgram", "isMut": false, "isSigner": false },
        { "name": "associatedTokenProgram", "isMut": false, "isSigner": false },
        { "name": "mplTokenMetadataProgram", "isMut": false, "isSigner": false }
      ],
      "args": [
        { "name": "params", "type": { "defined": "CreateMarketParams" } }
      ]
    },
    {
      "name": "swap",
      "accounts": [
        { "name": "globalConfig", "isMut": true, "isSigner": false },
        { "name": "teamWallet", "isMut": true, "isSigner": false },
        { "name": "market", "isMut": true, "isSigner": false },
        { "name": "globalVault", "isMut": true, "isSigner": false },
        { "name": "yesToken", "isMut": false, "isSigner": false },
        { "name": "noToken", "isMut": false, "isSigner": false },
        { "name": "globalYesAta", "isMut": true, "isSigner": false },
        { "name": "globalNoAta", "isMut": true, "isSigner": false },
        { "name": "userYesAta", "isMut": true, "isSigner": false },
        { "name": "userNoAta", "isMut": true, "isSigner": false },
        { "name": "userInfo", "isMut": true, "isSigner": false },
        { "name": "user", "isMut": true, "isSigner": true },
        { "name": "systemProgram", "isMut": false, "isSigner": false },
        { "name": "tokenProgram", "isMut": false, "isSigner": false },
        { "name": "associatedTokenProgram", "isMut": false, "isSigner": false }
      ],
      "args": [
        { "name": "amount", "type": "u64" },
        { "name": "direction", "type": "u8" },
        { "name": "tokenType", "type": "u8" },
        { "name": "minimumReceiveAmount", "type": "u64" }
      ]
    },
    {
      "name": "addLiquidity",
      "accounts": [
        { "name": "globalConfig", "isMut": true, "isSigner": false },
        { "name": "teamWallet", "isMut": true, "isSigner": false },
        { "name": "market", "isMut": true, "isSigner": false },
        { "name": "globalVault", "isMut": true, "isSigner": false },
        { "name": "yesToken", "isMut": false, "isSigner": false },
        { "name": "noToken", "isMut": false, "isSigner": false },
        { "name": "userInfo", "isMut": true, "isSigner": false },
        { "name": "user", "isMut": true, "isSigner": true },
        { "name": "systemProgram", "isMut": false, "isSigner": false },
        { "name": "tokenProgram", "isMut": false, "isSigner": false },
        { "name": "associatedTokenProgram", "isMut": false, "isSigner": false }
      ],
      "args": [
        { "name": "amount", "type": "u64" }
      ]
    },
    {
      "name": "withdrawLiquidity",
      "accounts": [
        { "name": "globalConfig", "isMut": true, "isSigner": false },
        { "name": "teamWallet", "isMut": true, "isSigner": false },
        { "name": "market", "isMut": true, "isSigner": false },
        { "name": "globalVault", "isMut": true, "isSigner": false },
        { "name": "yesToken", "isMut": false, "isSigner": false },
        { "name": "noToken", "isMut": false, "isSigner": false },
        { "name": "userInfo", "isMut": true, "isSigner": false },
        { "name": "user", "isMut": true, "isSigner": true },
        { "name": "systemProgram", "isMut": false, "isSigner": false },
        { "name": "tokenProgram", "isMut": false, "isSigner": false },
        { "name": "associatedTokenProgram", "isMut": false, "isSigner": false }
      ],
      "args": [
        { "name": "amount", "type": "u64" }
      ]
    },
    {
      "name": "resolution",
      "accounts": [
        { "name": "globalConfig", "isMut": true, "isSigner": false },
        { "name": "market", "isMut": true, "isSigner": false },
        { "name": "globalVault", "isMut": true, "isSigner": false },
        { "name": "yesToken", "isMut": false, "isSigner": false },
        { "name": "noToken", "isMut": false, "isSigner": false },
        { "name": "userInfo", "isMut": true, "isSigner": false },
        { "name": "user", "isMut": true, "isSigner": false },
        { "name": "authority", "isMut": false, "isSigner": true },
        { "name": "systemProgram", "isMut": false, "isSigner": false },
        { "name": "tokenProgram", "isMut": false, "isSigner": false },
        { "name": "associatedTokenProgram", "isMut": false, "isSigner": false }
      ],
      "args": [
        { "name": "yesAmount", "type": "u64" },
        { "name": "noAmount", "type": "u64" },
        { "name": "tokenType", "type": "u8" },
        { "name": "isCompleted", "type": "bool" }
      ]
    }
  ],
  "accounts": [
    {
      "name": "Config",
      "type": {
        "kind": "struct",
        "fields": [
          { "name": "authority", "type": "publicKey" },
          { "name": "pendingAuthority", "type": "publicKey" },
          { "name": "teamWallet", "type": "publicKey" },
          { "name": "platformBuyFee", "type": "u64" },
          { "name": "platformSellFee", "type": "u64" },
          { "name": "lpBuyFee", "type": "u64" },
          { "name": "lpSellFee", "type": "u64" },
          { "name": "tokenSupplyConfig", "type": "u64" },
          { "name": "tokenDecimalsConfig", "type": "u8" },
          { "name": "initialRealTokenReservesConfig", "type": "u64" },
          { "name": "minSolLiquidity", "type": "u64" },
          { "name": "initialized", "type": "bool" }
        ]
      }
    },
    {
      "name": "Market",
      "type": {
        "kind": "struct",
        "fields": [
          { "name": "yesTokenMint", "type": "publicKey" },
          { "name": "noTokenMint", "type": "publicKey" },
          { "name": "creator", "type": "publicKey" },
          { "name": "initialYesTokenReserves", "type": "u64" },
          { "name": "realYesTokenReserves", "type": "u64" },
          { "name": "realYesSolReserves", "type": "u64" },
          { "name": "tokenYesTotalSupply", "type": "u64" },
          { "name": "initialNoTokenReserves", "type": "u64" },
          { "name": "realNoTokenReserves", "type": "u64" },
          { "name": "realNoSolReserves", "type": "u64" },
          { "name": "tokenNoTotalSupply", "type": "u64" },
          { "name": "isCompleted", "type": "bool" },
          { "name": "startSlot", "type": { "option": "u64" } },
          { "name": "endingSlot", "type": { "option": "u64" } },
          { "name": "lps", "type": { "vec": { "defined": "LpInfo" } } },
          { "name": "totalLpAmount", "type": "u64" }
        ]
      }
    },
    {
      "name": "UserInfo",
      "type": {
        "kind": "struct",
        "fields": [
          { "name": "user", "type": "publicKey" },
          { "name": "yesBalance", "type": "u64" },
          { "name": "noBalance", "type": "u64" },
          { "name": "isLp", "type": "bool" },
          { "name": "isInitialized", "type": "bool" }
        ]
      }
    },
    {
      "name": "LpInfo",
      "type": {
        "kind": "struct",
        "fields": [
          { "name": "user", "type": "publicKey" },
          { "name": "solAmount", "type": "u64" }
        ]
      }
    }
  ],
  "types": [
    {
      "name": "CreateMarketParams",
      "type": {
        "kind": "struct",
        "fields": [
          { "name": "yesSymbol", "type": "string" },
          { "name": "yesUri", "type": "string" },
          { "name": "startSlot", "type": { "option": "u64" } },
          { "name": "endingSlot", "type": { "option": "u64" } }
        ]
      }
    }
  ],
  "errors": [
    { "code": 6000, "name": "IncorrectAuthority", "msg": "Incorrect authority" },
    { "code": 6001, "name": "InvalidEndTime", "msg": "Invalid end time" },
    { "code": 6002, "name": "CurveAlreadyCompleted", "msg": "Curve already completed" },
    { "code": 6003, "name": "InvalidMigrationAuthority", "msg": "Invalid migration authority" },
    { "code": 6004, "name": "WithdrawNotLpError", "msg": "Withdraw not LP error" },
    { "code": 6005, "name": "ValueTooSmall", "msg": "Value too small" },
    { "code": 6006, "name": "ValueTooLarge", "msg": "Value too large" },
    { "code": 6007, "name": "ValueInvalid", "msg": "Value invalid" }
  ]
};

// 钱包连接Hook
export function useWallet() {
  const [wallet, setWallet] = useState<Keypair | null>(null);
  const [connected, setConnected] = useState(false);

  const connectWallet = () => {
    // 在实际应用中，这里应该连接到用户的钱包
    // 例如使用 Phantom、Solflare 等
    const newWallet = Keypair.generate();
    setWallet(newWallet);
    setConnected(true);
  };

  const disconnectWallet = () => {
    setWallet(null);
    setConnected(false);
  };

  return {
    wallet,
    connected,
    connectWallet,
    disconnectWallet
  };
}

// 程序初始化Hook
export function useProgram(network: 'devnet' | 'mainnet' = 'devnet') {
  const [program, setProgram] = useState<Program | null>(null);
  const [connection, setConnection] = useState<Connection | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const initProgram = async () => {
      try {
        setLoading(true);
        setError(null);

        const config = NETWORK_CONFIG[network];
        const newConnection = new Connection(config.rpcUrl, 'confirmed');
        setConnection(newConnection);

        // 在实际应用中，需要从文件或CDN加载IDL
        // const program = new Program(IDL as any, config.programId, provider);
        // setProgram(program);

      } catch (err) {
        setError(err instanceof Error ? err.message : '初始化程序失败');
      } finally {
        setLoading(false);
      }
    };

    initProgram();
  }, [network]);

  return {
    program,
    connection,
    loading,
    error
  };
}

// 主应用组件
export default function App() {
  const { wallet, connected, connectWallet, disconnectWallet } = useWallet();
  const { program, connection, loading: programLoading, error: programError } = useProgram('devnet');

  if (programLoading) {
    return (
      <div className="loading-container">
        <div className="loading-spinner"></div>
        <p>正在初始化程序...</p>
      </div>
    );
  }

  if (programError) {
    return (
      <div className="error-container">
        <h2>初始化失败</h2>
        <p>{programError}</p>
        <button onClick={() => window.location.reload()}>
          重新加载
        </button>
      </div>
    );
  }

  return (
    <div className="app">
      <header className="app-header">
        <h1>Solana 预测市场</h1>
        <div className="wallet-section">
          {connected && wallet ? (
            <div className="wallet-info">
              <span className="wallet-address">
                {wallet.publicKey.toString().slice(0, 8)}...
                {wallet.publicKey.toString().slice(-8)}
              </span>
              <button onClick={disconnectWallet} className="btn btn-secondary">
                断开连接
              </button>
            </div>
          ) : (
            <button onClick={connectWallet} className="btn btn-primary">
              连接钱包
            </button>
          )}
        </div>
      </header>

      <main className="app-main">
        {connected ? (
          <PredictionMarketApp />
        ) : (
          <div className="connect-prompt">
            <h2>欢迎使用 Solana 预测市场</h2>
            <p>请先连接您的钱包以开始使用</p>
            <button onClick={connectWallet} className="btn btn-primary btn-large">
              连接钱包
            </button>
          </div>
        )}
      </main>
    </div>
  );
}

// 使用示例
export function ExampleUsage() {
  const { wallet, connected } = useWallet();
  const { program, connection } = useProgram('devnet');

  const [client, setClient] = useState<PredictionMarketClient | null>(null);

  useEffect(() => {
    if (connected && wallet && program && connection) {
      const newClient = new PredictionMarketClient(program, connection, wallet);
      setClient(newClient);
    }
  }, [connected, wallet, program, connection]);

  const handleCreateMarket = async () => {
    if (!client) return;

    try {
      const tx = await client.createMarket({
        yesSymbol: 'YES',
        yesUri: 'https://example.com/metadata.json',
        startSlot: undefined,
        endingSlot: undefined
      });
      
      console.log('市场创建成功:', tx);
    } catch (error) {
      console.error('创建市场失败:', error);
    }
  };

  const handleSwap = async () => {
    if (!client) return;

    try {
      // 这里需要实际的marketPDA
      const marketPDA = new PublicKey('...'); // 实际的市场地址
      const yesTokenMint = new PublicKey('...'); // 实际的YES代币地址
      const noTokenMint = new PublicKey('...'); // 实际的NO代币地址

      const tx = await client.swapTokens(marketPDA, yesTokenMint, noTokenMint, {
        amount: 1000000000, // 1 SOL (in lamports)
        direction: SwapDirection.BUY,
        tokenType: TokenType.YES,
        minimumReceiveAmount: 0
      });

      console.log('交易成功:', tx);
    } catch (error) {
      console.error('交易失败:', error);
    }
  };

  return (
    <div className="example-usage">
      <h2>使用示例</h2>
      
      <div className="example-actions">
        <button 
          onClick={handleCreateMarket}
          disabled={!client}
          className="btn btn-primary"
        >
          创建市场
        </button>
        
        <button 
          onClick={handleSwap}
          disabled={!client}
          className="btn btn-secondary"
        >
          执行交易
        </button>
      </div>
    </div>
  );
}

// 样式
const globalStyles = `
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen',
    'Ubuntu', 'Cantarell', 'Fira Sans', 'Droid Sans', 'Helvetica Neue',
    sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  background-color: #f5f5f5;
}

.app {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}

.app-header {
  background-color: white;
  padding: 1rem 2rem;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.app-main {
  flex: 1;
  padding: 2rem;
}

.loading-container,
.error-container,
.connect-prompt {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 400px;
  text-align: center;
}

.loading-spinner {
  width: 40px;
  height: 40px;
  border: 4px solid #f3f3f3;
  border-top: 4px solid #007bff;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: 1rem;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.wallet-section {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.wallet-info {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.wallet-address {
  font-family: monospace;
  background-color: #f8f9fa;
  padding: 0.5rem;
  border-radius: 4px;
  font-size: 0.9rem;
}

.btn {
  padding: 0.75rem 1.5rem;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 1rem;
  font-weight: 500;
  transition: all 0.2s;
  text-decoration: none;
  display: inline-block;
}

.btn-primary {
  background-color: #007bff;
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background-color: #0056b3;
}

.btn-secondary {
  background-color: #6c757d;
  color: white;
}

.btn-secondary:hover:not(:disabled) {
  background-color: #545b62;
}

.btn-large {
  padding: 1rem 2rem;
  font-size: 1.1rem;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.example-usage {
  max-width: 800px;
  margin: 0 auto;
  padding: 2rem;
  background-color: white;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0,0,0,0.1);
}

.example-actions {
  display: flex;
  gap: 1rem;
  margin-top: 1rem;
}
`;

// 导出样式
export { globalStyles };
