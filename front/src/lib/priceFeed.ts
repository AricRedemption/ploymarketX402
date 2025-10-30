/**
 * Price feed utilities for SOL/USD conversion
 */

/**
 * Get current SOL/USD price
 * Using CoinGecko API as a simple price feed
 */
export async function getSolPrice(): Promise<number> {
  try {
    const response = await fetch(
      'https://api.coingecko.com/api/v3/simple/price?ids=solana&vs_currencies=usd'
    );
    const data = await response.json();
    return data.solana.usd;
  } catch (error) {
    console.error('Error fetching SOL price:', error);
    // Fallback price if API fails
    return 150; // ~$150 USD per SOL as fallback
  }
}

/**
 * Convert SOL amount to USD
 */
export async function solToUsd(solAmount: number): Promise<number> {
  const solPrice = await getSolPrice();
  return solAmount * solPrice;
}

/**
 * Convert USD amount to SOL
 */
export async function usdToSol(usdAmount: number): Promise<number> {
  const solPrice = await getSolPrice();
  return usdAmount / solPrice;
}
