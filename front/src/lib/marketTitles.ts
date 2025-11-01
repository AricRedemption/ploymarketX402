/**
 * Market titles configuration
 * Maps market addresses to their actual questions/titles
 *
 * Add your market addresses here to display proper titles on the home page
 * You can also use environment variables for this
 */

interface MarketTitle {
  address: string;
  question: string;
  description?: string;
  category?: string;
}

// Load market titles from environment variable if available
const envMarketTitles = process.env.NEXT_PUBLIC_MARKET_TITLES
  ? JSON.parse(process.env.NEXT_PUBLIC_MARKET_TITLES)
  : [];

export const MARKET_TITLES: MarketTitle[] = [
  // Add your markets here
  // Example:
  // {
  //   address: 'HsVGJFqBBVYUHNM7yJ3vbPSXhM7YRfGME4jLVcKqDxZE',
  //   question: 'Will Bitcoin reach $100k by end of 2025?',
  //   description: 'This market predicts whether Bitcoin will reach $100,000 USD by December 31, 2025',
  //   category: 'Crypto'
  // },
  // {
  //   address: 'GsWGJFqBBVYUHNM7yJ3vbPSXhM7YRfGME4jLVcKqDxZE',
  //   question: 'Will ETH overtake BTC in market cap?',
  //   description: 'Ethereum vs Bitcoin market cap comparison',
  //   category: 'Crypto'
  // },

  // Markets from environment variable (NEXT_PUBLIC_MARKET_TITLES)
  ...envMarketTitles,
];

/**
 * Get market title by address
 */
export function getMarketTitle(address: string): MarketTitle | undefined {
  return MARKET_TITLES.find(m => m.address === address);
}
