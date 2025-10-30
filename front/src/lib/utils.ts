/**
 * Utility functions for formatting and constants
 */

// Category constants
export const CATEGORIES = [
  'All',
  'Crypto',
  'Politics',
  'Sports',
  'Entertainment',
  'Technology',
  'Science',
  'Economics'
];

/**
 * Format volume in a human-readable way
 */
export function formatVolume(volume: number): string {
  if (volume >= 1000000) {
    return `$${(volume / 1000000).toFixed(1)}M`;
  }
  if (volume >= 1000) {
    return `$${(volume / 1000).toFixed(0)}K`;
  }
  return `$${volume}`;
}

/**
 * Format probability as percentage
 */
export function formatProbability(price: number): string {
  if (typeof price !== 'number' || isNaN(price)) {
    return '50%'; // Default fallback
  }
  return `${(price * 100).toFixed(0)}%`;
}

/**
 * Get Solscan URL for an address
 * @param address - The Solana address (account, token, transaction)
 * @param type - Type of address: 'account', 'token', 'tx'
 * @param cluster - Network cluster: 'mainnet' or 'devnet'
 */
export function getSolscanUrl(
  address: string,
  type: 'account' | 'token' | 'tx' = 'account',
  cluster: 'mainnet' | 'devnet' = 'devnet'
): string {
  const baseUrl = 'https://solscan.io';
  const clusterParam = cluster === 'devnet' ? '?cluster=devnet' : '';

  switch (type) {
    case 'token':
      return `${baseUrl}/token/${address}${clusterParam}`;
    case 'tx':
      return `${baseUrl}/tx/${address}${clusterParam}`;
    case 'account':
    default:
      return `${baseUrl}/account/${address}${clusterParam}`;
  }
}

/**
 * Shorten an address for display
 */
export function shortenAddress(address: string, chars: number = 4): string {
  if (address.length <= chars * 2 + 3) return address;
  return `${address.slice(0, chars)}...${address.slice(-chars)}`;
}
