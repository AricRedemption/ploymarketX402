/**
 * X402 Integration for USDC deposits
 * Using X402 protocol with Solana for USDC payments
 */

import { PaymentRequirements } from 'x402/types';

/**
 * Create payment requirements for market token purchase on Solana
 */
export function createMarketPaymentRequirements(
  usdAmount: number,
  marketId: string,
  tokenType: 'yes' | 'no'
): PaymentRequirements {
  // Convert USD to smallest unit (USDC has 6 decimals)
  const amountInSmallestUnit = Math.floor(usdAmount * 1_000_000).toString();

  return {
    scheme: 'exact',
    network: 'solana-devnet', // Using Solana devnet
    maxAmountRequired: amountInSmallestUnit,
    resource: `https://402market.com/market/${marketId}`,
    description: `${tokenType.toUpperCase()} tokens for Market #${marketId}`,
    mimeType: 'application/json',
    payTo:
      process.env.NEXT_PUBLIC_X402_PAYMENT_ADDRESS ||
      '9tXDC3VJhKyNsG4VQdZ1234567890abcdefghij', // Solana wallet address (placeholder)
    maxTimeoutSeconds: 600,
    asset: '4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU', // USDC on Solana devnet
    outputSchema: undefined,
    extra: undefined, // No extra data needed for Solana
  };
}
