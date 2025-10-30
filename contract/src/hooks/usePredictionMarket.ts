// React Hook for Solana Prediction Market Integration
import { useState, useEffect, useCallback, useMemo } from 'react';
import { Connection, PublicKey, Keypair } from '@solana/web3.js';
import { Program, AnchorProvider } from '@coral-xyz/anchor';
import { PredictionMarketClient, MarketInfo, UserInfo, TokenType, SwapDirection } from './PredictionMarketClient';

// 网络配置
const NETWORK_CONFIG = {
  devnet: {
    rpcUrl: 'https://api.devnet.solana.com',
    programId: '5q1C8N47AYvLu7w6LKngwXhLjrZCZ5izMB8nbziZhYEV'
  },
  mainnet: {
    rpcUrl: 'https://api.mainnet-beta.solana.com',
    programId: '4mKzcNjkA4gtSpJezssQCJRTN1VTKazeujx2rhT6me5K'
  }
};

// Hook接口定义
interface UsePredictionMarketProps {
  network?: 'devnet' | 'mainnet';
  wallet?: Keypair;
}

interface UsePredictionMarketReturn {
  // 客户端状态
  client: PredictionMarketClient | null;
  connection: Connection | null;
  program: Program | null;
  isConnected: boolean;
  
  // 市场数据
  markets: MarketInfo[];
  userMarkets: MarketInfo[];
  userInfo: UserInfo | null;
  
  // 加载状态
  loading: boolean;
  error: string | null;
  
  // 操作方法
  initializeConfig: (config: ConfigParams) => Promise<string>;
  createMarket: (params: CreateMarketParams) => Promise<string>;
  swapTokens: (marketPDA: PublicKey, params: SwapParams) => Promise<string>;
  addLiquidity: (marketPDA: PublicKey, amount: number) => Promise<string>;
  withdrawLiquidity: (marketPDA: PublicKey, amount: number) => Promise<string>;
  resolveMarket: (marketPDA: PublicKey, params: ResolutionParams) => Promise<string>;
  
  // 查询方法
  refreshMarkets: () => Promise<void>;
  refreshUserInfo: (marketPDA: PublicKey) => Promise<void>;
  getSwapPreview: (marketPDA: PublicKey, amount: number, tokenType: TokenType) => Promise<any>;
}

interface CreateMarketParams {
  yesSymbol: string;
  yesUri: string;
  startSlot?: number;
  endingSlot?: number;
}

interface SwapParams {
  amount: number;
  direction: SwapDirection;
  tokenType: TokenType;
  minimumReceiveAmount: number;
}

interface ResolutionParams {
  yesAmount: number;
  noAmount: number;
  tokenType: TokenType;
  isCompleted: boolean;
}

interface ConfigParams {
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
}

// 主要的React Hook
export function usePredictionMarket({ 
  network = 'devnet', 
  wallet 
}: UsePredictionMarketProps): UsePredictionMarketReturn {
  
  // 状态管理
  const [client, setClient] = useState<PredictionMarketClient | null>(null);
  const [connection, setConnection] = useState<Connection | null>(null);
  const [program, setProgram] = useState<Program | null>(null);
  const [isConnected, setIsConnected] = useState(false);
  const [markets, setMarkets] = useState<MarketInfo[]>([]);
  const [userMarkets, setUserMarkets] = useState<MarketInfo[]>([]);
  const [userInfo, setUserInfo] = useState<UserInfo | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // 初始化连接
  useEffect(() => {
    const initConnection = async () => {
      try {
        setLoading(true);
        setError(null);

        const config = NETWORK_CONFIG[network];
        const newConnection = new Connection(config.rpcUrl, 'confirmed');
        setConnection(newConnection);

        if (wallet) {
          const provider = new AnchorProvider(newConnection, wallet, {});
          
          // 加载IDL文件
          const idl = require('../../target/idl/prediction_market.json');
          const program = new Program(idl as any, config.programId, provider);
          setProgram(program);
          
          const newClient = new PredictionMarketClient(
            program,
            newConnection,
            wallet
          );
          setClient(newClient);
          setIsConnected(true);
        }

      } catch (err) {
        setError(err instanceof Error ? err.message : '连接失败');
        setIsConnected(false);
      } finally {
        setLoading(false);
      }
    };

    initConnection();
  }, [network, wallet]);

  // 创建市场
  const createMarket = useCallback(async (params: CreateMarketParams): Promise<string> => {
    if (!client) throw new Error('客户端未初始化');
    
    try {
      setLoading(true);
      setError(null);
      
      const tx = await client.createMarket(params);
      await refreshMarkets(); // 刷新市场列表
      
      return tx;
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : '创建市场失败';
      setError(errorMsg);
      throw err;
    } finally {
      setLoading(false);
    }
  }, [client]);

  // 初始化配置
  const initializeConfig = useCallback(async (config: ConfigParams): Promise<string> => {
    if (!client) throw new Error('客户端未初始化');
    
    try {
      setLoading(true);
      setError(null);
      
      const tx = await client.initializeConfig(config);
      
      return tx;
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : '初始化配置失败';
      setError(errorMsg);
      throw err;
    } finally {
      setLoading(false);
    }
  }, [client]);

  // 交易代币
  const swapTokens = useCallback(async (
    marketPDA: PublicKey, 
    params: SwapParams
  ): Promise<string> => {
    if (!client) throw new Error('客户端未初始化');
    
    try {
      setLoading(true);
      setError(null);
      
      // 获取市场信息以获取代币mint地址
      const marketInfo = await client.getMarketInfo(marketPDA);
      
      const tx = await client.swapTokens(
        marketPDA,
        marketInfo.yesTokenMint,
        marketInfo.noTokenMint,
        params
      );
      
      await refreshUserInfo(marketPDA); // 刷新用户信息
      
      return tx;
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : '交易失败';
      setError(errorMsg);
      throw err;
    } finally {
      setLoading(false);
    }
  }, [client]);

  // 添加流动性
  const addLiquidity = useCallback(async (
    marketPDA: PublicKey, 
    amount: number
  ): Promise<string> => {
    if (!client) throw new Error('客户端未初始化');
    
    try {
      setLoading(true);
      setError(null);
      
      const marketInfo = await client.getMarketInfo(marketPDA);
      
      const tx = await client.addLiquidity(
        marketPDA,
        marketInfo.yesTokenMint,
        marketInfo.noTokenMint,
        { amount }
      );
      
      await refreshUserInfo(marketPDA);
      
      return tx;
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : '添加流动性失败';
      setError(errorMsg);
      throw err;
    } finally {
      setLoading(false);
    }
  }, [client]);

  // 提取流动性
  const withdrawLiquidity = useCallback(async (
    marketPDA: PublicKey, 
    amount: number
  ): Promise<string> => {
    if (!client) throw new Error('客户端未初始化');
    
    try {
      setLoading(true);
      setError(null);
      
      const marketInfo = await client.getMarketInfo(marketPDA);
      
      const tx = await client.withdrawLiquidity(
        marketPDA,
        marketInfo.yesTokenMint,
        marketInfo.noTokenMint,
        { amount }
      );
      
      await refreshUserInfo(marketPDA);
      
      return tx;
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : '提取流动性失败';
      setError(errorMsg);
      throw err;
    } finally {
      setLoading(false);
    }
  }, [client]);

  // 市场结算
  const resolveMarket = useCallback(async (
    marketPDA: PublicKey, 
    params: ResolutionParams
  ): Promise<string> => {
    if (!client) throw new Error('客户端未初始化');
    
    try {
      setLoading(true);
      setError(null);
      
      const marketInfo = await client.getMarketInfo(marketPDA);
      
      const tx = await client.resolveMarket(
        marketPDA,
        marketInfo.yesTokenMint,
        marketInfo.noTokenMint,
        params.yesAmount,
        params.noAmount,
        params.tokenType,
        params.isCompleted
      );
      
      await refreshMarkets();
      
      return tx;
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : '市场结算失败';
      setError(errorMsg);
      throw err;
    } finally {
      setLoading(false);
    }
  }, [client]);

  // 刷新市场列表
  const refreshMarkets = useCallback(async () => {
    if (!client) return;
    
    try {
      // 这里需要实现获取所有市场的逻辑
      // 由于合约没有提供获取所有市场的方法，可能需要使用事件监听或其他方式
      console.log('刷新市场列表');
    } catch (err) {
      console.error('刷新市场列表失败:', err);
    }
  }, [client]);

  // 刷新用户信息
  const refreshUserInfo = useCallback(async (marketPDA: PublicKey) => {
    if (!client) return;
    
    try {
      const userInfoPDA = client.getUserInfoPDA(marketPDA);
      const info = await client.getUserInfo(userInfoPDA);
      setUserInfo(info);
    } catch (err) {
      console.error('刷新用户信息失败:', err);
    }
  }, [client]);

  // 获取交易预览
  const getSwapPreview = useCallback(async (
    marketPDA: PublicKey, 
    amount: number, 
    tokenType: TokenType
  ) => {
    if (!client) return null;
    
    try {
      return await client.getSwapPreview(marketPDA, amount, tokenType);
    } catch (err) {
      console.error('获取交易预览失败:', err);
      return null;
    }
  }, [client]);

  return {
    client,
    connection,
    program,
    isConnected,
    markets,
    userMarkets,
    userInfo,
    loading,
    error,
    initializeConfig,
    createMarket,
    swapTokens,
    addLiquidity,
    withdrawLiquidity,
    resolveMarket,
    refreshMarkets,
    refreshUserInfo,
    getSwapPreview
  };
}

// 市场信息Hook
export function useMarketInfo(marketPDA: PublicKey | null) {
  const [marketInfo, setMarketInfo] = useState<MarketInfo | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchMarketInfo = useCallback(async () => {
    if (!marketPDA) return;
    
    try {
      setLoading(true);
      setError(null);
      
      // 这里需要客户端实例
      // const info = await client.getMarketInfo(marketPDA);
      // setMarketInfo(info);
    } catch (err) {
      setError(err instanceof Error ? err.message : '获取市场信息失败');
    } finally {
      setLoading(false);
    }
  }, [marketPDA]);

  useEffect(() => {
    fetchMarketInfo();
  }, [fetchMarketInfo]);

  return {
    marketInfo,
    loading,
    error,
    refresh: fetchMarketInfo
  };
}

// 用户信息Hook
export function useUserInfo(marketPDA: PublicKey | null) {
  const [userInfo, setUserInfo] = useState<UserInfo | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchUserInfo = useCallback(async () => {
    if (!marketPDA) return;
    
    try {
      setLoading(true);
      setError(null);
      
      // 这里需要客户端实例
      // const info = await client.getUserInfo(userInfoPDA);
      // setUserInfo(info);
    } catch (err) {
      setError(err instanceof Error ? err.message : '获取用户信息失败');
    } finally {
      setLoading(false);
    }
  }, [marketPDA]);

  useEffect(() => {
    fetchUserInfo();
  }, [fetchUserInfo]);

  return {
    userInfo,
    loading,
    error,
    refresh: fetchUserInfo
  };
}

// 交易预览Hook
export function useSwapPreview(
  marketPDA: PublicKey | null,
  amount: number,
  tokenType: TokenType
) {
  const [preview, setPreview] = useState<any>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const calculatePreview = useCallback(async () => {
    if (!marketPDA || amount <= 0) return;
    
    try {
      setLoading(true);
      setError(null);
      
      // 这里需要客户端实例
      // const result = await client.getSwapPreview(marketPDA, amount, tokenType);
      // setPreview(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : '计算交易预览失败');
    } finally {
      setLoading(false);
    }
  }, [marketPDA, amount, tokenType]);

  useEffect(() => {
    calculatePreview();
  }, [calculatePreview]);

  return {
    preview,
    loading,
    error,
    refresh: calculatePreview
  };
}

// 工具Hook
export function usePredictionMarketUtils() {
  const formatTokenAmount = useCallback((amount: number, decimals: number = 9): string => {
    return (amount / Math.pow(10, decimals)).toFixed(decimals);
  }, []);

  const parseTokenAmount = useCallback((amount: string, decimals: number = 9): number => {
    return Math.floor(parseFloat(amount) * Math.pow(10, decimals));
  }, []);

  const calculatePrice = useCallback((yesReserves: number, noReserves: number) => {
    const totalReserves = yesReserves + noReserves;
    if (totalReserves === 0) {
      return { yesPrice: 0.5, noPrice: 0.5 };
    }
    
    const yesPrice = yesReserves / totalReserves;
    const noPrice = noReserves / totalReserves;
    
    return { yesPrice, noPrice };
  }, []);

  const isMarketValid = useCallback((marketInfo: MarketInfo | null): boolean => {
    if (!marketInfo) return false;
    
    // 检查市场是否已完成
    if (marketInfo.isCompleted) {
      return false;
    }
    
    // 检查结束时间
    if (marketInfo.endingSlot) {
      // 这里需要将slot转换为时间戳进行比较
      return true;
    }
    
    return true;
  }, []);

  return {
    formatTokenAmount,
    parseTokenAmount,
    calculatePrice,
    isMarketValid
  };
}
