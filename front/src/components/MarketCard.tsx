'use client';

import Link from 'next/link';
import { Market } from '@/lib/solana/types';
import { formatVolume, formatProbability, getSolscanUrl } from '@/lib/utils';

interface MarketCardProps {
  market: Market;
}

export default function MarketCard({ market }: MarketCardProps) {
  const daysUntilEnd = market.endDate
    ? Math.ceil((market.endDate.getTime() - Date.now()) / (1000 * 60 * 60 * 24))
    : null;

  // Format creation time
  const formatTimeAgo = (date: Date): string => {
    const seconds = Math.floor((Date.now() - date.getTime()) / 1000);

    if (seconds < 60) return 'just now';
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
    if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
    if (seconds < 604800) return `${Math.floor(seconds / 86400)}d ago`;

    // For older dates, show the actual date
    return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
  };

  return (
    <Link href={`/market/${market.id}`}>
      <div className="bg-white dark:bg-neutral-900 rounded-lg border border-neutral-200 dark:border-neutral-800 p-4 hover:shadow-lg transition-all duration-200 cursor-pointer hover:scale-[1.01]">
        {/* Category Badge and Time Info */}
        <div className="flex items-center justify-between mb-3">
          <div className="flex items-center gap-2">
            <span className="px-2 py-1 text-xs font-medium bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-400 rounded">
              {market.category}
            </span>
            <span className="text-xs text-neutral-400 dark:text-neutral-500">
              {formatTimeAgo(market.createdAt)}
            </span>
          </div>
          <span className="text-xs text-neutral-500 dark:text-neutral-400">
            {daysUntilEnd === null ? 'No end date' : daysUntilEnd > 0 ? `${daysUntilEnd}d left` : 'Ended'}
          </span>
        </div>

        {/* Question */}
        <h3 className="text-base font-semibold text-neutral-900 dark:text-neutral-100 mb-2 line-clamp-2 min-h-[48px]">
          {market.question}
        </h3>

        {/* Market Info with Clickable Addresses */}
        <div className="text-xs text-neutral-500 dark:text-neutral-400 mb-3 flex flex-wrap gap-x-2 gap-y-1">
          <span
            onClick={(e) => {
              e.preventDefault();
              e.stopPropagation();
              window.open(getSolscanUrl(market.address.toString(), 'account'), '_blank');
            }}
            className="hover:text-blue-600 dark:hover:text-blue-400 underline cursor-pointer"
          >
            Market: {market.address.toString().slice(0, 4)}...{market.address.toString().slice(-4)}
          </span>
          {market.yesTokenMint.toString() !== '11111111111111111111111111111111' && (
            <>
              <span>•</span>
              <span
                onClick={(e) => {
                  e.preventDefault();
                  e.stopPropagation();
                  window.open(getSolscanUrl(market.yesTokenMint.toString(), 'token'), '_blank');
                }}
                className="hover:text-blue-600 dark:hover:text-blue-400 underline cursor-pointer"
              >
                YES: {market.yesTokenMint.toString().slice(0, 4)}...{market.yesTokenMint.toString().slice(-4)}
              </span>
              <span>•</span>
              <span
                onClick={(e) => {
                  e.preventDefault();
                  e.stopPropagation();
                  window.open(getSolscanUrl(market.noTokenMint.toString(), 'token'), '_blank');
                }}
                className="hover:text-blue-600 dark:hover:text-blue-400 underline cursor-pointer"
              >
                NO: {market.noTokenMint.toString().slice(0, 4)}...{market.noTokenMint.toString().slice(-4)}
              </span>
            </>
          )}
        </div>

        {/* Probability Display */}
        <div className="grid grid-cols-2 gap-2 mb-3">
          {/* Yes */}
          <div className="bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg p-3">
            <div className="text-xs text-neutral-600 dark:text-neutral-400 mb-1">
              Yes
            </div>
            <div className="text-2xl font-bold text-green-600 dark:text-green-400">
              {formatProbability(market.yesPrice)}
            </div>
          </div>

          {/* No */}
          <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-3">
            <div className="text-xs text-neutral-600 dark:text-neutral-400 mb-1">
              No
            </div>
            <div className="text-2xl font-bold text-red-600 dark:text-red-400">
              {formatProbability(market.noPrice)}
            </div>
          </div>
        </div>

        {/* Volume and Liquidity */}
        <div className="flex items-center justify-between text-xs text-neutral-500 dark:text-neutral-400 pt-3 border-t border-neutral-200 dark:border-neutral-800">
          <div>
            <span className="font-medium">Volume:</span> {formatVolume(market.volume)}
          </div>
          <div>
            <span className="font-medium">Liquidity:</span> {formatVolume(market.liquidity)}
          </div>
        </div>
      </div>
    </Link>
  );
}
