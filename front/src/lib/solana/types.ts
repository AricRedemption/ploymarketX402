import { PublicKey } from '@solana/web3.js';
import BN from 'bn.js';

// Market account structure from contract
export interface MarketAccount {
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
  lps: LpInfo[];
  totalLpAmount: BN;
}

// LP Info structure
export interface LpInfo {
  user: PublicKey;
  solAmount: BN;
}

// User Info account structure
export interface UserInfoAccount {
  user: PublicKey;
  yesBalance: BN;
  noBalance: BN;
  isLp: boolean;
  isInitialized: boolean;
}

// Global Config account structure
export interface GlobalConfigAccount {
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

// Create Market Parameters
export interface CreateMarketParams {
  yesSymbol: string;
  yesUri: string;
  startSlot: BN | null;
  endingSlot: BN | null;
}

// Trade direction and token type enums
export enum TradeDirection {
  Buy = 0,
  Sell = 1,
}

export enum TokenType {
  Yes = 0,
  No = 1,
}

// Frontend-friendly market interface
export interface Market {
  id: string;
  address: PublicKey;
  question: string;
  description: string;
  category: string;
  createdAt: Date; // creation timestamp
  endDate: Date | null;
  volume: number; // calculated from reserves
  liquidity: number; // total SOL in both pools
  yesPrice: number; // 0-1 probability
  noPrice: number; // 0-1 probability
  yesTokenMint: PublicKey;
  noTokenMint: PublicKey;
  creator: PublicKey;
  status: 'active' | 'closed' | 'resolved';
  resolvedOutcome?: 'yes' | 'no';
  tags: string[];
  // Raw contract data
  contractData: MarketAccount;
}

// Trade event
export interface TradeEvent {
  user: PublicKey;
  tokenYes: PublicKey;
  tokenNo: PublicKey;
  marketInfo: PublicKey;
  solAmount: BN;
  tokenAmount: BN;
  feeLamports: BN;
  isBuy: boolean;
  isYesNo: boolean;
  realSolReserves: BN;
  realTokenYesReserves: BN;
  realTokenNoReserves: BN;
  timestamp: BN;
}

// Helper function to calculate price from reserves (AMM formula)
export function calculatePrice(
  tokenReserves: BN,
  solReserves: BN,
  totalSupply: BN
): number {
  if (tokenReserves.isZero() || solReserves.isZero()) {
    return 0.5; // Default 50% if no liquidity
  }

  // Price = SOL_reserves / (SOL_reserves + TOKEN_reserves converted to SOL equivalent)
  // Using constant product AMM: x * y = k
  // Price ≈ y / (x + y) where y is SOL and x is token

  const solValue = solReserves.toNumber();
  const tokenValue = tokenReserves.toNumber();

  // Simplified price calculation
  // Real formula would be: solReserves / (solReserves + (tokenReserves * currentPrice))
  // For now, approximate as: solReserves / totalSupply
  const price = solValue / (totalSupply.toNumber() / 1e9);

  // Clamp between 0.01 and 0.99
  return Math.max(0.01, Math.min(0.99, price));
}

// Calculate how many tokens you get for SOL (buy)
export function calculateBuyAmount(
  solAmount: BN,
  tokenReserves: BN,
  solReserves: BN
): BN {
  // AMM formula: Δtoken = tokenReserves * (1 - solReserves / (solReserves + solAmount))
  const newSolReserves = solReserves.add(solAmount);
  const ratio = solReserves.mul(new BN(1e9)).div(newSolReserves);
  const tokensOut = tokenReserves.sub(tokenReserves.mul(ratio).div(new BN(1e9)));
  return tokensOut;
}

// Calculate how much SOL you get for tokens (sell)
export function calculateSellAmount(
  tokenAmount: BN,
  tokenReserves: BN,
  solReserves: BN
): BN {
  // AMM formula: Δsol = solReserves * (1 - tokenReserves / (tokenReserves + tokenAmount))
  const newTokenReserves = tokenReserves.add(tokenAmount);
  const ratio = tokenReserves.mul(new BN(1e9)).div(newTokenReserves);
  const solOut = solReserves.sub(solReserves.mul(ratio).div(new BN(1e9)));
  return solOut;
}
