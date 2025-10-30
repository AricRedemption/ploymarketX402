import { PublicKey, Connection, clusterApiUrl } from '@solana/web3.js';

// Program ID from deployed contract
export const PROGRAM_ID = new PublicKey('EgEc7fuse6eQ3UwqeWGFncDtbTwozWCy4piydbeRaNrU');

// RPC endpoint - can be configured via environment variable
export const RPC_ENDPOINT = process.env.NEXT_PUBLIC_SOLANA_RPC_ENDPOINT || clusterApiUrl('devnet');

// Create connection
export const connection = new Connection(RPC_ENDPOINT, 'confirmed');

// Global config PDA seeds
export const CONFIG_SEED = Buffer.from('config');
export const GLOBAL_VAULT_SEED = Buffer.from('global');
export const MARKET_SEED = Buffer.from('market');
export const USER_INFO_SEED = Buffer.from('userinfo');

// Get PDA for global config
export function getGlobalConfigPDA(): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [CONFIG_SEED],
    PROGRAM_ID
  );
}

// Get PDA for global vault
export function getGlobalVaultPDA(): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [GLOBAL_VAULT_SEED],
    PROGRAM_ID
  );
}

// Get PDA for market
export function getMarketPDA(yesToken: PublicKey, noToken: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [MARKET_SEED, yesToken.toBuffer(), noToken.toBuffer()],
    PROGRAM_ID
  );
}

// Get PDA for user info
export function getUserInfoPDA(user: PublicKey, market: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [USER_INFO_SEED, user.toBuffer(), market.toBuffer()],
    PROGRAM_ID
  );
}

// Contract deployed data from contract_deployed_data.md
export const DEPLOYED_CONFIG = {
  programId: 'EgEc7fuse6eQ3UwqeWGFncDtbTwozWCy4piydbeRaNrU',
  // Team wallet from deployed config
  teamWallet: new PublicKey('CRD9Pe7ou8yidVnCn3a1rUejSczCA1xsSwajfqS5Xfwb'),
  // Example market from test deployment
  exampleMarket: {
    marketAddress: 'RbKhjLGrHYh9kW7vNkwcHmkBJZTT7yKKtiXdpNYm1tz',
    yesTokenAddress: '25C8k3BoofV8XG42rqrxNPsmZbnuZBdaQF6afdLqxf48',
    noTokenAddress: 'AypNhMLiNYWKL8Tr9WaWkwKcQ5Hq8xvR8ykqCCSfFqEW',
  }
};

// Token decimals from contract
export const TOKEN_DECIMALS = 9;

// Convert SOL amount to lamports
export function solToLamports(sol: number): number {
  return Math.floor(sol * 1e9);
}

// Convert lamports to SOL
export function lamportsToSol(lamports: number): number {
  return lamports / 1e9;
}

// Convert token amount to smallest unit
export function toTokenAmount(amount: number, decimals: number = TOKEN_DECIMALS): number {
  return Math.floor(amount * Math.pow(10, decimals));
}

// Convert smallest unit to token amount
export function fromTokenAmount(amount: number, decimals: number = TOKEN_DECIMALS): number {
  return amount / Math.pow(10, decimals);
}
