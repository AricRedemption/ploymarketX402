"use client";

import { useParams, useRouter } from "next/navigation";
import { useState, useEffect } from "react";
import { formatVolume, formatProbability, getSolscanUrl } from "@/lib/utils";
import { useWallet } from "@solana/wallet-adapter-react";
import { WalletMultiButton } from "@solana/wallet-adapter-react-ui";
import { useMarket } from "@/hooks/useMarkets";
import { useSwap } from "@/hooks/useSwap";
import { useTokenBalance } from "@/hooks/useTokenBalance";
import { useLiquidity } from "@/hooks/useLiquidity";
import { solToUsd } from "@/lib/priceFeed";
import Link from "next/link";

export default function MarketDetailPage() {
  const params = useParams();
  const router = useRouter();
  const wallet = useWallet();
  const {
    market,
    loading: marketLoading,
    error: marketError,
  } = useMarket(params.id as string);
  const { buyTokens, loading: swapLoading, error: swapError } = useSwap();
  const {
    addLiquidity,
    withdrawLiquidity,
    loading: liquidityLoading,
  } = useLiquidity();
  const {
    yesBalance,
    noBalance,
    loading: balanceLoading,
  } = useTokenBalance(
    market?.yesTokenMint || null,
    market?.noTokenMint || null
  );
  const [mounted, setMounted] = useState(false);

  const [selectedOutcome, setSelectedOutcome] = useState<"yes" | "no" | null>(
    null
  );
  const [solAmount, setSolAmount] = useState("");
  const [showBuyModal, setShowBuyModal] = useState(false);
  const [txSignature, setTxSignature] = useState<string | null>(null);

  // Liquidity management state
  const [showLiquidityModal, setShowLiquidityModal] = useState(false);
  const [liquidityAction, setLiquidityAction] = useState<"add" | "withdraw">(
    "add"
  );
  const [liquidityAmount, setLiquidityAmount] = useState("");
  const [usdAmount, setUsdAmount] = useState<number | null>(null);
  const [useX402, setUseX402] = useState(true); // Toggle between X402 and direct contract
  const [isProcessingPayment, setIsProcessingPayment] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  // Calculate USD amount when SOL amount changes
  useEffect(() => {
    async function calculateUsd() {
      if (solAmount && parseFloat(solAmount) > 0) {
        const usd = await solToUsd(parseFloat(solAmount));
        setUsdAmount(usd);
      } else {
        setUsdAmount(null);
      }
    }
    calculateUsd();
  }, [solAmount]);

  if (marketLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-neutral-50 dark:bg-neutral-950">
        <div className="text-center">
          <div className="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
          <p className="text-neutral-600 dark:text-neutral-400 mt-4">
            Loading market...
          </p>
        </div>
      </div>
    );
  }

  if (marketError || !market) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-neutral-50 dark:bg-neutral-950">
        <div className="text-center">
          <h1 className="text-2xl font-bold text-neutral-900 dark:text-neutral-100 mb-4">
            {marketError ? "Error loading market" : "Market not found"}
          </h1>
          {marketError && (
            <p className="text-sm text-neutral-500 mb-4">
              {marketError.message}
            </p>
          )}
          <Link
            href="/"
            className="text-blue-600 hover:text-blue-700 dark:text-blue-400 dark:hover:text-blue-300"
          >
            ← Back to markets
          </Link>
        </div>
      </div>
    );
  }

  const daysUntilEnd = market.endDate
    ? Math.ceil((market.endDate.getTime() - Date.now()) / (1000 * 60 * 60 * 24))
    : null;

  const handleBuyClick = (outcome: "yes" | "no") => {
    setSelectedOutcome(outcome);
    setShowBuyModal(true);
  };

  const handleConfirmPurchase = async () => {
    const amount = parseFloat(solAmount);
    if (amount > 0 && selectedOutcome && wallet.publicKey) {
      try {
        if (useX402 && usdAmount) {
          // Redirect to paywall page with market parameters
          console.log("💰 Redirecting to X402 payment page");

          const paywallUrl = `/paywall?amount=${usdAmount.toFixed(
            2
          )}&marketId=${market.id}&outcome=${selectedOutcome}`;
          window.location.href = paywallUrl;
          return;
        } else {
          // Original contract-based swap (currently not working)
          const teamWallet = market.creator.toString();

          const signature = await buyTokens(
            market.id,
            market.yesTokenMint.toString(),
            market.noTokenMint.toString(),
            selectedOutcome,
            amount,
            teamWallet
          );

          setTxSignature(signature);
        }

        // Close modal after delay
        setTimeout(() => {
          setShowBuyModal(false);
          setSolAmount("");
          setSelectedOutcome(null);
          setTxSignature(null);
        }, 5000);
      } catch (err) {
        console.error("Purchase failed:", err);
        setIsProcessingPayment(false);
        alert(
          `Purchase failed: ${
            err instanceof Error ? err.message : "Unknown error"
          }`
        );
      }
    }
  };

  const handleLiquidityAction = async () => {
    const amount = parseFloat(liquidityAmount);
    if (amount > 0 && wallet.publicKey) {
      try {
        const teamWallet = market.creator.toString();

        let signature;
        if (liquidityAction === "add") {
          signature = await addLiquidity(
            market.id,
            market.yesTokenMint.toString(),
            market.noTokenMint.toString(),
            amount,
            teamWallet
          );
        } else {
          signature = await withdrawLiquidity(
            market.id,
            market.yesTokenMint.toString(),
            market.noTokenMint.toString(),
            amount,
            teamWallet
          );
        }

        setTxSignature(signature);
        // Close modal and show success
        setTimeout(() => {
          setShowLiquidityModal(false);
          setLiquidityAmount("");
          setTxSignature(null);
          // Reload the page to refresh market data
          window.location.reload();
        }, 3000);
      } catch (err) {
        console.error("Liquidity action failed:", err);
      }
    }
  };

  const isMarketCreator =
    wallet.publicKey &&
    market &&
    wallet.publicKey.toString() === market.creator.toString();

  const calculateShares = () => {
    const amount = parseFloat(solAmount) || 0;
    const price = selectedOutcome === "yes" ? market.yesPrice : market.noPrice;
    return (amount / price).toFixed(2);
  };

  const potentialReturn = () => {
    const shares = parseFloat(calculateShares());
    return shares.toFixed(2);
  };

  return (
    <div className="min-h-screen bg-neutral-50 dark:bg-neutral-950">
      {/* Header */}
      <header className="sticky top-0 z-50 bg-white dark:bg-neutral-900 border-b border-neutral-200 dark:border-neutral-800">
        <div className="max-w-7xl mx-auto px-4 py-4">
          <div className="flex items-center justify-between">
            <Link href="/">
              <h1 className="text-2xl font-bold bg-gradient-to-r from-blue-600 to-purple-600 bg-clip-text text-transparent cursor-pointer">
                402Market
              </h1>
            </Link>

            {mounted && (
              <WalletMultiButton className="!bg-blue-600 hover:!bg-blue-700 !rounded-lg !h-10" />
            )}
          </div>
        </div>
      </header>

      <main className="max-w-5xl mx-auto px-4 py-8">
        {/* Back Button */}
        <Link
          href="/"
          className="inline-flex items-center text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-100 mb-6"
        >
          ← Back to markets
        </Link>

        {/* Market Header */}
        <div className="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-lg p-6 mb-6">
          <div className="flex items-start justify-between mb-4">
            <div className="flex-grow">
              <div className="flex items-center gap-3 mb-3">
                <span className="px-3 py-1 text-sm font-medium bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-400 rounded">
                  {market.category}
                </span>
                <span className="text-sm text-neutral-500 dark:text-neutral-400">
                  {daysUntilEnd !== null && daysUntilEnd > 0
                    ? `Closes in ${daysUntilEnd} days`
                    : market.status === "closed"
                    ? "Closed"
                    : "Active"}
                </span>
              </div>
              <h1 className="text-3xl font-bold text-neutral-900 dark:text-neutral-100 mb-3">
                {market.question}
              </h1>
              <p className="text-neutral-600 dark:text-neutral-400">
                {market.description}
              </p>
            </div>
          </div>

          {/* Market Stats */}
          <div className="grid grid-cols-3 gap-4 pt-4 border-t border-neutral-200 dark:border-neutral-800">
            <div>
              <div className="text-sm text-neutral-500 dark:text-neutral-400 mb-1">
                Volume
              </div>
              <div className="text-xl font-bold text-neutral-900 dark:text-neutral-100">
                {formatVolume(market.volume)}
              </div>
            </div>
            <div>
              <div className="text-sm text-neutral-500 dark:text-neutral-400 mb-1">
                Liquidity
              </div>
              <div className="text-xl font-bold text-neutral-900 dark:text-neutral-100">
                {formatVolume(market.liquidity)}
              </div>
            </div>
            <div>
              <div className="text-sm text-neutral-500 dark:text-neutral-400 mb-1">
                Traders
              </div>
              <div className="text-xl font-bold text-neutral-900 dark:text-neutral-100">
                {Math.floor(market.volume / 1000)}
              </div>
            </div>
          </div>
        </div>

        {/* User Holdings */}
        {wallet.publicKey && !balanceLoading && (
          <div className="mb-6 bg-gradient-to-r from-purple-50 to-blue-50 dark:from-purple-900/20 dark:to-blue-900/20 border border-purple-200 dark:border-purple-800 rounded-lg p-4">
            <div className="flex items-center justify-between mb-3">
              <h3 className="text-lg font-bold text-neutral-900 dark:text-neutral-100">
                Your Holdings
              </h3>
              {yesBalance === 0 && noBalance === 0 && (
                <span className="text-sm text-neutral-500 dark:text-neutral-400">
                  No tokens yet
                </span>
              )}
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="bg-white/50 dark:bg-neutral-800/50 rounded-lg p-3">
                <div className="text-sm text-neutral-600 dark:text-neutral-400 mb-1">
                  YES Tokens
                </div>
                <div className="text-2xl font-bold text-green-600 dark:text-green-400">
                  {yesBalance.toFixed(2)}
                </div>
                <div className="text-xs text-neutral-500 dark:text-neutral-400 mt-1">
                  Value: ~${(yesBalance * market.yesPrice).toFixed(2)}
                </div>
              </div>
              <div className="bg-white/50 dark:bg-neutral-800/50 rounded-lg p-3">
                <div className="text-sm text-neutral-600 dark:text-neutral-400 mb-1">
                  NO Tokens
                </div>
                <div className="text-2xl font-bold text-red-600 dark:text-red-400">
                  {noBalance.toFixed(2)}
                </div>
                <div className="text-xs text-neutral-500 dark:text-neutral-400 mt-1">
                  Value: ~${(noBalance * market.noPrice).toFixed(2)}
                </div>
              </div>
            </div>
            <div className="mt-3 pt-3 border-t border-purple-200 dark:border-purple-800">
              <div className="flex justify-between text-sm">
                <span className="text-neutral-600 dark:text-neutral-400">
                  Total Value:
                </span>
                <span className="font-bold text-neutral-900 dark:text-neutral-100">
                  ~$
                  {(
                    yesBalance * market.yesPrice +
                    noBalance * market.noPrice
                  ).toFixed(2)}
                </span>
              </div>
            </div>
          </div>
        )}

        {/* Market Creator Controls - Not Implemented Warning */}
        {/* {isMarketCreator && (
          <div className="mb-6 bg-red-50 dark:bg-red-900/20 border-2 border-red-400 dark:border-red-700 rounded-lg p-4">
            <div className="flex items-start gap-3">
              <div className="text-2xl">⚠️</div>
              <div className="flex-1">
                <h3 className="text-lg font-bold text-red-900 dark:text-red-200 mb-2">
                  Smart Contract Not Fully Implemented
                </h3>
                <p className="text-sm text-red-800 dark:text-red-300 mb-3">
                  The following functions in the Solana program are currently{" "}
                  <strong>placeholder implementations</strong> that do nothing:
                </p>
                <ul className="text-sm text-red-800 dark:text-red-300 mb-3 ml-4 list-disc space-y-1">
                  <li>
                    <code className="bg-red-100 dark:bg-red-900 px-1 rounded">
                      add_liquidity
                    </code>{" "}
                    - Cannot add SOL to market reserves
                  </li>
                  <li>
                    <code className="bg-red-100 dark:bg-red-900 px-1 rounded">
                      withdraw_liquidity
                    </code>{" "}
                    - Cannot withdraw SOL from reserves
                  </li>
                  <li>
                    <code className="bg-red-100 dark:bg-red-900 px-1 rounded">
                      resolution
                    </code>{" "}
                    - Cannot resolve market or distribute winnings
                  </li>
                </ul>
                <p className="text-sm text-red-800 dark:text-red-300 mb-3">
                  Current Liquidity: <strong>{formatVolume(market.liquidity)}</strong> | Status:{" "}
                  <strong>
                    {market.status.charAt(0).toUpperCase() + market.status.slice(1)}
                  </strong>
                </p>
                <div className="text-xs text-red-700 dark:text-red-400 bg-red-100 dark:bg-red-900/50 p-3 rounded">
                  <strong>For developers:</strong> See{" "}
                  <code className="bg-red-200 dark:bg-red-950 px-1 rounded">
                    contract_source_code/programs/prediction-market/src/state/market.rs:392-438
                  </code>
                  <br />
                  All these functions just return{" "}
                  <code className="bg-red-200 dark:bg-red-950 px-1 rounded">
                    Ok(())
                  </code>{" "}
                  without any actual implementation. The smart contract needs to be
                  completed before these features will work.
                </div>
              </div>
            </div>
          </div>
        )} */}

        {/* Liquidity Info */}
        {market.liquidity === 0 && !isMarketCreator && (
          <div className="mb-6 bg-blue-50 dark:bg-blue-900/20 border-2 border-blue-400 dark:border-blue-700 rounded-lg p-4">
            <div className="flex items-start gap-3">
              <div className="text-2xl">ℹ️</div>
              <div className="flex-1">
                <h3 className="text-lg font-bold text-blue-900 dark:text-blue-200 mb-2">
                  No Liquidity - Using X402 for Payments
                </h3>
                <p className="text-sm text-blue-800 dark:text-blue-300">
                  This market has no liquidity (0 SOL in reserves). You can
                  still purchase tokens by depositing USDC via X402. Direct
                  contract swaps won&apos;t work until liquidity is added.
                </p>
              </div>
            </div>
          </div>
        )}

        {/* Trading Interface */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {/* Yes Card */}
          <div className="bg-gradient-to-br from-green-50 to-emerald-50 dark:from-green-900/20 dark:to-emerald-900/20 border-2 border-green-300 dark:border-green-700 rounded-lg p-6">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-2xl font-bold text-green-700 dark:text-green-400">
                Yes
              </h3>
              <div className="text-right">
                <div className="text-4xl font-bold text-green-600 dark:text-green-400">
                  {formatProbability(market.yesPrice)}
                </div>
                <div className="text-sm text-green-600/70 dark:text-green-400/70">
                  ${market.yesPrice.toFixed(2)} per share
                </div>
              </div>
            </div>
            <button
              onClick={() => handleBuyClick("yes")}
              disabled={swapLoading}
              className="w-full font-bold py-3 px-6 rounded-lg transition-colors bg-green-600 hover:bg-green-700 dark:bg-green-700 dark:hover:bg-green-600 text-white disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Buy Yes
            </button>
          </div>

          {/* No Card */}
          <div className="bg-gradient-to-br from-red-50 to-rose-50 dark:from-red-900/20 dark:to-rose-900/20 border-2 border-red-300 dark:border-red-700 rounded-lg p-6">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-2xl font-bold text-red-700 dark:text-red-400">
                No
              </h3>
              <div className="text-right">
                <div className="text-4xl font-bold text-red-600 dark:text-red-400">
                  {formatProbability(market.noPrice)}
                </div>
                <div className="text-sm text-red-600/70 dark:text-red-400/70">
                  ${market.noPrice.toFixed(2)} per share
                </div>
              </div>
            </div>
            <button
              onClick={() => handleBuyClick("no")}
              disabled={swapLoading}
              className="w-full font-bold py-3 px-6 rounded-lg transition-colors bg-red-600 hover:bg-red-700 dark:bg-red-700 dark:hover:bg-red-600 text-white disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Buy No
            </button>
          </div>
        </div>

        {/* About Section */}
        <div className="mt-6 bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-lg p-6">
          <h3 className="text-xl font-bold text-neutral-900 dark:text-neutral-100 mb-3">
            About this market
          </h3>
          <div className="space-y-2 text-neutral-600 dark:text-neutral-400">
            <p>
              <strong>Category:</strong> {market.category}
            </p>
            <p>
              <strong>Market Address:</strong>{" "}
              <a
                href={getSolscanUrl(market.address.toString(), "account")}
                target="_blank"
                rel="noopener noreferrer"
                className="text-blue-600 dark:text-blue-400 hover:underline"
              >
                {market.address.toString().slice(0, 8)}...
                {market.address.toString().slice(-8)}
              </a>
            </p>
            {market.yesTokenMint.toString() !==
              "11111111111111111111111111111111" && (
              <>
                <p>
                  <strong>YES Token:</strong>{" "}
                  <a
                    href={getSolscanUrl(
                      market.yesTokenMint.toString(),
                      "token"
                    )}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-blue-600 dark:text-blue-400 hover:underline"
                  >
                    {market.yesTokenMint.toString().slice(0, 8)}...
                    {market.yesTokenMint.toString().slice(-8)}
                  </a>
                </p>
                <p>
                  <strong>NO Token:</strong>{" "}
                  <a
                    href={getSolscanUrl(market.noTokenMint.toString(), "token")}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-blue-600 dark:text-blue-400 hover:underline"
                  >
                    {market.noTokenMint.toString().slice(0, 8)}...
                    {market.noTokenMint.toString().slice(-8)}
                  </a>
                </p>
              </>
            )}
            <p>
              <strong>Creator:</strong>{" "}
              <a
                href={getSolscanUrl(market.creator.toString(), "account")}
                target="_blank"
                rel="noopener noreferrer"
                className="text-blue-600 dark:text-blue-400 hover:underline"
              >
                {market.creator.toString().slice(0, 8)}...
                {market.creator.toString().slice(-8)}
              </a>
            </p>
            <p>
              <strong>End Date:</strong>{" "}
              {market.endDate
                ? market.endDate.toLocaleDateString("en-US", {
                    year: "numeric",
                    month: "long",
                    day: "numeric",
                  })
                : "No end date"}
            </p>
            <p>
              <strong>Status:</strong>{" "}
              {market.status.charAt(0).toUpperCase() + market.status.slice(1)}
            </p>
          </div>
        </div>
      </main>

      {/* Buy Modal */}
      {showBuyModal && selectedOutcome && (
        <div className="fixed inset-0 bg-black/70 flex items-center justify-center z-50 p-4">
          <div className="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-2xl p-8 max-w-md w-full">
            <h2 className="text-3xl font-bold mb-6 text-neutral-900 dark:text-neutral-100">
              Buy {selectedOutcome === "yes" ? "Yes" : "No"} Tokens
            </h2>

            <div className="space-y-6">
              {/* Market Info */}
              <div className="bg-neutral-100 dark:bg-neutral-800 rounded-lg p-4">
                <p className="text-sm text-neutral-600 dark:text-neutral-400 mb-1">
                  Market
                </p>
                <p className="font-semibold text-neutral-900 dark:text-neutral-100 line-clamp-2">
                  {market.question}
                </p>
              </div>

              {/* Amount Input */}
              <div>
                <label className="block text-neutral-700 dark:text-neutral-300 mb-2 font-semibold">
                  Enter SOL Amount
                </label>

                {/* Quick Amount Buttons */}
                <div className="grid grid-cols-4 gap-2 mb-3">
                  {[0.1, 0.5, 1, 2].map((amount) => (
                    <button
                      key={amount}
                      type="button"
                      onClick={() => setSolAmount(amount.toString())}
                      className={`py-2 px-3 rounded-lg font-medium text-sm transition-colors ${
                        solAmount === amount.toString()
                          ? selectedOutcome === "yes"
                            ? "bg-green-600 text-white"
                            : "bg-red-600 text-white"
                          : "bg-neutral-200 dark:bg-neutral-700 text-neutral-700 dark:text-neutral-300 hover:bg-neutral-300 dark:hover:bg-neutral-600"
                      }`}
                    >
                      {amount} SOL
                    </button>
                  ))}
                </div>

                <input
                  type="number"
                  min="0.01"
                  step="0.01"
                  value={solAmount}
                  onChange={(e) => setSolAmount(e.target.value)}
                  placeholder="0.00"
                  className="w-full bg-neutral-100 dark:bg-neutral-800 border border-neutral-300 dark:border-neutral-700 rounded-lg px-4 py-3 text-neutral-900 dark:text-neutral-100 text-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
                <p className="text-neutral-500 dark:text-neutral-400 text-sm mt-1">
                  Minimum: 0.01 SOL
                </p>
              </div>

              {/* X402 Info */}
              {useX402 && (
                <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-300 dark:border-blue-700 rounded-lg p-3">
                  <div className="flex items-center gap-2 mb-2">
                    <span className="text-lg">🔗</span>
                    <span className="text-sm font-semibold text-blue-900 dark:text-blue-200">
                      Using X402 Protocol
                    </span>
                  </div>
                  <p className="text-xs text-blue-800 dark:text-blue-300">
                    You&apos;ll be redirected to complete payment using USDC via
                    X402.
                  </p>
                </div>
              )}

              {/* Calculation */}
              {solAmount && parseFloat(solAmount) > 0 && (
                <div
                  className={`rounded-xl p-4 ${
                    selectedOutcome === "yes"
                      ? "bg-green-100 dark:bg-green-900/30 border border-green-300 dark:border-green-700"
                      : "bg-red-100 dark:bg-red-900/30 border border-red-300 dark:border-red-700"
                  }`}
                >
                  <div className="space-y-2 text-sm">
                    <div className="flex items-center justify-between">
                      <span className="text-neutral-700 dark:text-neutral-300">
                        SOL Amount:
                      </span>
                      <span className="font-bold text-neutral-900 dark:text-neutral-100">
                        {parseFloat(solAmount).toFixed(4)} SOL
                      </span>
                    </div>
                    {usdAmount && (
                      <div className="flex items-center justify-between">
                        <span className="text-neutral-700 dark:text-neutral-300">
                          USD Equivalent:
                        </span>
                        <span className="font-bold text-neutral-900 dark:text-neutral-100">
                          ${usdAmount.toFixed(2)} USDC
                        </span>
                      </div>
                    )}
                    <div className="flex items-center justify-between">
                      <span className="text-neutral-700 dark:text-neutral-300">
                        Shares you&apos;ll get:
                      </span>
                      <span className="font-bold text-neutral-900 dark:text-neutral-100">
                        {calculateShares()}
                      </span>
                    </div>
                    <div className="flex items-center justify-between pt-2 border-t border-current/20">
                      <span className="text-neutral-700 dark:text-neutral-300">
                        Potential return:
                      </span>
                      <span className="text-lg font-bold text-neutral-900 dark:text-neutral-100">
                        {potentialReturn()} shares
                      </span>
                    </div>
                  </div>
                </div>
              )}

              {/* Transaction Status */}
              {(swapLoading || isProcessingPayment) && (
                <div className="text-center py-4">
                  <div className="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
                  <p className="text-sm text-neutral-600 dark:text-neutral-400 mt-2">
                    {useX402
                      ? "Processing X402 payment..."
                      : "Processing transaction..."}
                  </p>
                </div>
              )}

              {txSignature && (
                <div className="bg-green-100 dark:bg-green-900/30 border border-green-300 dark:border-green-700 rounded-lg p-4">
                  <p className="text-green-700 dark:text-green-400 font-semibold mb-2">
                    {useX402
                      ? "Payment successful!"
                      : "Transaction successful!"}
                  </p>
                  <a
                    href={`https://explorer.solana.com/tx/${txSignature}?cluster=devnet`}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-sm text-blue-600 dark:text-blue-400 hover:underline break-all"
                  >
                    View on Solana Explorer
                  </a>
                </div>
              )}

              {swapError && (
                <div className="bg-red-100 dark:bg-red-900/30 border border-red-300 dark:border-red-700 rounded-lg p-4">
                  <p className="text-red-700 dark:text-red-400 text-sm">
                    {swapError.message}
                  </p>
                </div>
              )}

              {/* Buttons */}
              <div className="flex gap-3">
                <button
                  onClick={() => {
                    setShowBuyModal(false);
                    setSolAmount("");
                    setSelectedOutcome(null);
                    setTxSignature(null);
                  }}
                  disabled={swapLoading}
                  className="flex-1 bg-neutral-200 dark:bg-neutral-700 hover:bg-neutral-300 dark:hover:bg-neutral-600 text-neutral-900 dark:text-neutral-100 font-bold py-3 px-6 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  Cancel
                </button>
                <button
                  onClick={handleConfirmPurchase}
                  disabled={
                    !wallet.publicKey ||
                    !solAmount ||
                    parseFloat(solAmount) < 0.01 ||
                    swapLoading ||
                    isProcessingPayment ||
                    !!txSignature
                  }
                  className={`flex-1 font-bold py-3 px-6 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed ${
                    selectedOutcome === "yes"
                      ? "bg-green-600 hover:bg-green-700 text-white"
                      : "bg-red-600 hover:bg-red-700 text-white"
                  }`}
                >
                  {!wallet.publicKey
                    ? "Connect Wallet"
                    : swapLoading || isProcessingPayment
                    ? "Processing..."
                    : txSignature
                    ? "Success!"
                    : useX402
                    ? `Continue to Payment ($${
                        usdAmount?.toFixed(2) || "0.00"
                      })`
                    : "Buy Tokens"}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Liquidity Modal */}
      {showLiquidityModal && (
        <div className="fixed inset-0 bg-black/70 flex items-center justify-center z-50 p-4">
          <div className="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-2xl p-8 max-w-md w-full">
            <h2 className="text-3xl font-bold mb-6 text-neutral-900 dark:text-neutral-100">
              {liquidityAction === "add" ? "Add" : "Withdraw"} Liquidity
            </h2>

            <div className="space-y-6">
              {/* Market Info */}
              <div className="bg-neutral-100 dark:bg-neutral-800 rounded-lg p-4">
                <p className="text-sm text-neutral-600 dark:text-neutral-400 mb-1">
                  Market
                </p>
                <p className="font-semibold text-neutral-900 dark:text-neutral-100 line-clamp-2">
                  {market.question}
                </p>
                <p className="text-sm text-neutral-600 dark:text-neutral-400 mt-2">
                  Current Liquidity: {formatVolume(market.liquidity)}
                </p>
              </div>

              {/* Amount Input */}
              <div>
                <label className="block text-neutral-700 dark:text-neutral-300 mb-2 font-semibold">
                  Enter SOL Amount
                </label>

                {/* Quick Amount Buttons */}
                <div className="grid grid-cols-4 gap-2 mb-3">
                  {[0.5, 1, 5, 10].map((amount) => (
                    <button
                      key={amount}
                      type="button"
                      onClick={() => setLiquidityAmount(amount.toString())}
                      className={`py-2 px-3 rounded-lg font-medium text-sm transition-colors ${
                        liquidityAmount === amount.toString()
                          ? liquidityAction === "add"
                            ? "bg-green-600 text-white"
                            : "bg-orange-600 text-white"
                          : "bg-neutral-200 dark:bg-neutral-700 text-neutral-700 dark:text-neutral-300 hover:bg-neutral-300 dark:hover:bg-neutral-600"
                      }`}
                    >
                      {amount} SOL
                    </button>
                  ))}
                </div>

                <input
                  type="number"
                  min="0.1"
                  step="0.1"
                  value={liquidityAmount}
                  onChange={(e) => setLiquidityAmount(e.target.value)}
                  placeholder="0.00"
                  className="w-full bg-neutral-100 dark:bg-neutral-800 border border-neutral-300 dark:border-neutral-700 rounded-lg px-4 py-3 text-neutral-900 dark:text-neutral-100 text-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
                <p className="text-neutral-500 dark:text-neutral-400 text-sm mt-1">
                  Minimum: 0.1 SOL
                </p>
              </div>

              {/* Info */}
              {liquidityAmount && parseFloat(liquidityAmount) > 0 && (
                <div
                  className={`rounded-xl p-4 ${
                    liquidityAction === "add"
                      ? "bg-green-100 dark:bg-green-900/30 border border-green-300 dark:border-green-700"
                      : "bg-orange-100 dark:bg-orange-900/30 border border-orange-300 dark:border-orange-700"
                  }`}
                >
                  <p className="text-sm text-neutral-700 dark:text-neutral-300">
                    {liquidityAction === "add" ? (
                      <>
                        You will {liquidityAction}{" "}
                        <strong>
                          {parseFloat(liquidityAmount).toFixed(2)} SOL
                        </strong>{" "}
                        to this market. This will enable trading and you may
                        earn fees from swaps.
                      </>
                    ) : (
                      <>
                        You will {liquidityAction}{" "}
                        <strong>
                          {parseFloat(liquidityAmount).toFixed(2)} SOL
                        </strong>{" "}
                        from this market. This may reduce trading availability.
                      </>
                    )}
                  </p>
                </div>
              )}

              {/* Transaction Status */}
              {txSignature && (
                <div className="bg-green-100 dark:bg-green-900/30 border border-green-300 dark:border-green-700 rounded-lg p-4">
                  <p className="text-sm text-green-800 dark:text-green-300 mb-2">
                    Transaction successful!
                  </p>
                  <a
                    href={`https://solscan.io/tx/${txSignature}?cluster=devnet`}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-xs text-blue-600 dark:text-blue-400 hover:underline break-all"
                  >
                    View on Solscan
                  </a>
                </div>
              )}

              {/* Buttons */}
              <div className="flex gap-3">
                <button
                  onClick={() => {
                    setShowLiquidityModal(false);
                    setLiquidityAmount("");
                    setTxSignature(null);
                  }}
                  disabled={liquidityLoading}
                  className="flex-1 bg-neutral-200 dark:bg-neutral-700 hover:bg-neutral-300 dark:hover:bg-neutral-600 text-neutral-900 dark:text-neutral-100 font-bold py-3 px-6 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  Cancel
                </button>
                <button
                  onClick={handleLiquidityAction}
                  disabled={
                    !wallet.publicKey ||
                    !liquidityAmount ||
                    parseFloat(liquidityAmount) < 0.1 ||
                    liquidityLoading ||
                    !!txSignature
                  }
                  className={`flex-1 font-bold py-3 px-6 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed ${
                    liquidityAction === "add"
                      ? "bg-green-600 hover:bg-green-700 text-white"
                      : "bg-orange-600 hover:bg-orange-700 text-white"
                  }`}
                >
                  {!wallet.publicKey
                    ? "Connect Wallet"
                    : liquidityLoading
                    ? "Processing..."
                    : txSignature
                    ? "Success!"
                    : liquidityAction === "add"
                    ? "Add Liquidity"
                    : "Withdraw Liquidity"}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
