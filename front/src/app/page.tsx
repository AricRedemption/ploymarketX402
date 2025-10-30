'use client';

import { useState, useMemo, useEffect } from 'react';
import Link from 'next/link';
import { useWallet } from '@solana/wallet-adapter-react';
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';
import MarketCard from '@/components/MarketCard';
import { useMarkets } from '@/hooks/useMarkets';
import { CATEGORIES } from '@/lib/utils';

export default function HomePage() {
  const [selectedCategory, setSelectedCategory] = useState('All');
  const [searchQuery, setSearchQuery] = useState('');
  const { markets, loading, error } = useMarkets();
  const wallet = useWallet();
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  const filteredMarkets = useMemo(() => {
    let filtered = markets;

    // Filter by category
    if (selectedCategory !== 'All') {
      filtered = filtered.filter(m => m.category === selectedCategory);
    }

    // Filter by search query
    if (searchQuery) {
      filtered = filtered.filter(market =>
        market.question.toLowerCase().includes(searchQuery.toLowerCase()) ||
        market.description.toLowerCase().includes(searchQuery.toLowerCase())
      );
    }

    return filtered;
  }, [markets, selectedCategory, searchQuery]);

  return (
    <div className="flex flex-col min-h-screen font-sans bg-neutral-50 dark:bg-neutral-950">
      {/* Header */}
      <header className="sticky top-0 z-50 bg-white dark:bg-neutral-900 border-b border-neutral-200 dark:border-neutral-800">
        <div className="max-w-7xl mx-auto px-4 py-4">
          <div className="flex items-center justify-between">
            {/* Logo */}
            <div className="flex items-center gap-8">
              <h1 className="text-2xl font-bold bg-gradient-to-r from-blue-600 to-purple-600 bg-clip-text text-transparent">
                402Market
              </h1>

              {/* Search Bar */}
              <div className="hidden md:block">
                <input
                  type="text"
                  placeholder="Search markets..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="w-80 px-4 py-2 bg-neutral-100 dark:bg-neutral-800 border border-neutral-300 dark:border-neutral-700 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>
            </div>

            {/* Actions */}
            <div className="flex items-center gap-4">
              <Link
                href="/create-market"
                className="hidden sm:block px-4 py-2 bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-700 hover:to-purple-700 text-white font-semibold rounded-lg transition-colors"
              >
                Create Market
              </Link>

              {/* Wallet Connection */}
              {mounted && <WalletMultiButton className="!bg-blue-600 hover:!bg-blue-700 !rounded-lg !h-10" />}
            </div>
          </div>

          {/* Mobile Search & Create */}
          <div className="sm:hidden mt-3 space-y-2">
            <input
              type="text"
              placeholder="Search markets..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full px-4 py-2 bg-neutral-100 dark:bg-neutral-800 border border-neutral-300 dark:border-neutral-700 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
            <Link
              href="/create-market"
              className="block w-full px-4 py-2 bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-700 hover:to-purple-700 text-white font-semibold rounded-lg text-center transition-colors"
            >
              Create Market
            </Link>
          </div>
        </div>
      </header>

      <main className="flex-grow max-w-7xl mx-auto w-full px-4 py-6">
        {/* Hero Section */}
        <div className="mb-8">
          <h2 className="text-3xl md:text-4xl font-bold text-neutral-900 dark:text-neutral-100 mb-2">
            Prediction Markets
          </h2>
          <p className="text-neutral-600 dark:text-neutral-400">
            Trade on your favorite outcomes. Powered by x402 protocol.
          </p>
        </div>

        {/* Category Filter */}
        <div className="mb-6 flex gap-2 overflow-x-auto pb-2">
          {CATEGORIES.map((category) => (
            <button
              key={category}
              onClick={() => setSelectedCategory(category)}
              className={`px-4 py-2 rounded-lg font-medium text-sm whitespace-nowrap transition-colors ${
                selectedCategory === category
                  ? 'bg-blue-600 text-white'
                  : 'bg-white dark:bg-neutral-800 text-neutral-700 dark:text-neutral-300 border border-neutral-200 dark:border-neutral-700 hover:bg-neutral-100 dark:hover:bg-neutral-700'
              }`}
            >
              {category}
            </button>
          ))}
        </div>

        {/* Loading State */}
        {loading && (
          <div className="text-center py-16">
            <div className="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
            <p className="text-neutral-600 dark:text-neutral-400 mt-4">Loading markets...</p>
          </div>
        )}

        {/* Error State */}
        {error && (
          <div className="text-center py-16">
            <p className="text-red-600 dark:text-red-400 mb-2">Error loading markets</p>
            <p className="text-sm text-neutral-500">{error.message}</p>
          </div>
        )}

        {/* Markets Grid */}
        {!loading && !error && filteredMarkets.length > 0 && (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {filteredMarkets.map((market) => (
              <MarketCard key={market.id} market={market} />
            ))}
          </div>
        )}

        {/* No Results */}
        {!loading && !error && filteredMarkets.length === 0 && markets.length > 0 && (
          <div className="text-center py-16">
            <p className="text-neutral-500 dark:text-neutral-400 text-lg">
              No markets found matching your search.
            </p>
          </div>
        )}

        {/* No Markets at All */}
        {!loading && !error && markets.length === 0 && (
          <div className="text-center py-16">
            <p className="text-neutral-500 dark:text-neutral-400 text-lg mb-4">
              No markets created yet. Be the first!
            </p>
            <Link
              href="/create-market"
              className="inline-block px-6 py-3 bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-700 hover:to-purple-700 text-white font-semibold rounded-lg transition-colors"
            >
              Create Market
            </Link>
          </div>
        )}

        {/* Stats Section */}
        {!loading && !error && markets.length > 0 && (
          <div className="mt-12 grid grid-cols-1 md:grid-cols-3 gap-6">
            <div className="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-lg p-6">
              <div className="text-sm text-neutral-600 dark:text-neutral-400 mb-1">
                Total Volume (24h)
              </div>
              <div className="text-3xl font-bold text-neutral-900 dark:text-neutral-100">
                ${(markets.reduce((sum, m) => sum + m.volume, 0) / 1000000).toFixed(1)}M
              </div>
            </div>
            <div className="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-lg p-6">
              <div className="text-sm text-neutral-600 dark:text-neutral-400 mb-1">
                Active Markets
              </div>
              <div className="text-3xl font-bold text-neutral-900 dark:text-neutral-100">
                {markets.filter(m => m.status === 'active').length}
              </div>
            </div>
            <div className="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-lg p-6">
              <div className="text-sm text-neutral-600 dark:text-neutral-400 mb-1">
                Total Liquidity
              </div>
              <div className="text-3xl font-bold text-neutral-900 dark:text-neutral-100">
                ${(markets.reduce((sum, m) => sum + m.liquidity, 0) / 1000000).toFixed(1)}M
              </div>
            </div>
          </div>
        )}
      </main>

      {/* Footer */}
      <footer className="mt-16 border-t border-neutral-200 dark:border-neutral-800 py-8">
        <div className="max-w-7xl mx-auto px-4 text-center text-sm text-neutral-500 dark:text-neutral-400">
          <p>Powered by Solana • Decentralized Prediction Markets</p>
        </div>
      </footer>
    </div>
  );
}
