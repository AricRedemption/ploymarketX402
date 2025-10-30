"use client";
import { WalletMultiButton } from "@solana/wallet-adapter-react-ui";
import { useWallet, useConnection } from "@solana/wallet-adapter-react";
import { useState, useEffect, Suspense } from "react";
import { useSearchParams, useRouter } from "next/navigation";
import { verifyMarketTokenPayment } from "../actions";
import { PaymentRequirements } from "x402/types";
import { createSolanaPaymentPayload } from "@/lib/x402-solana";
import { encodePayment } from "x402/schemes";
import { Buffer } from "buffer";

function PaymentForm({
  paymentRequirements,
  marketId,
  outcome,
}: {
  paymentRequirements: PaymentRequirements;
  marketId: string | null;
  outcome: 'yes' | 'no' | null;
}) {
  const wallet = useWallet();
  const { connection } = useConnection();
  const router = useRouter();
  const [isProcessing, setIsProcessing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  if (!wallet.publicKey || !wallet.connected) {
    return (
      <div className="text-center space-y-4">
        <WalletMultiButton />
        <p className="text-neutral-600 dark:text-neutral-400">
          Please connect your Solana wallet to proceed with payment.
        </p>
      </div>
    );
  }

  async function handlePayment() {
    setIsProcessing(true);
    setError(null);

    try {
      console.log('Creating Solana payment transaction...');

      // Create the payment payload (signed transaction)
      const paymentPayload = await createSolanaPaymentPayload(
        wallet,
        paymentRequirements,
        connection
      );

      console.log('Encoding payment...');
      // Encode the payment (scheme-agnostic encoding)
      const payment = encodePayment(paymentPayload);

      console.log('Verifying payment with server...');
      // Verify payment server-side
      const result = await verifyMarketTokenPayment(
        payment,
        paymentRequirements.maxAmountRequired,
        marketId || 'general',
        outcome || 'yes'
      );

      if (!result.success) {
        throw new Error(result.error || 'Payment verification failed');
      }

      console.log('✅ Payment validated! Sending transaction to Solana...');

      // Decode the signed transaction and send it to Solana
      const signedTransactionBase64 = paymentPayload.payload.transaction;
      const signedTransactionBuffer = Buffer.from(signedTransactionBase64, 'base64');

      // Send the transaction to Solana network
      const signature = await connection.sendRawTransaction(signedTransactionBuffer, {
        skipPreflight: false,
        preflightCommitment: 'confirmed',
      });

      console.log('Transaction sent! Signature:', signature);
      console.log('Confirming transaction...');

      // Wait for confirmation
      const confirmation = await connection.confirmTransaction(signature, 'confirmed');

      if (confirmation.value.err) {
        throw new Error('Transaction failed: ' + JSON.stringify(confirmation.value.err));
      }

      console.log('✅ Transaction confirmed!', signature);
      setSuccess(true);

      // Redirect back to market after success
      setTimeout(() => {
        if (marketId) {
          router.push(`/market/${marketId}?payment=success`);
        } else {
          router.push('/?payment=success');
        }
      }, 2000);
    } catch (err: any) {
      console.error('Payment error:', err);
      setError(err.message || 'Payment failed. Please try again.');
    } finally {
      setIsProcessing(false);
    }
  }

  const usdcAmount = (
    parseInt(paymentRequirements.maxAmountRequired) / 1_000_000
  ).toFixed(2);

  const gameTokens = (parseFloat(usdcAmount) * 100).toFixed(0);
  const isMarketPurchase = paymentRequirements.description.includes('tokens for Market');

  return (
    <div className="space-y-6">
      <div className="flex justify-center">
        <WalletMultiButton />
      </div>

      <div className="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-xl p-6">
        <h3 className="text-xl font-bold mb-4 text-neutral-900 dark:text-neutral-100">
          Payment Details
        </h3>
        <div className="space-y-2 text-neutral-600 dark:text-neutral-400">
          <p>
            <span className="font-semibold text-neutral-900 dark:text-neutral-100">
              Amount:
            </span>{" "}
            ${usdcAmount} USDC (on Solana devnet)
          </p>
          {!isMarketPurchase && (
            <p>
              <span className="font-semibold text-neutral-900 dark:text-neutral-100">
                You will receive:
              </span>{" "}
              <span className="text-purple-600 dark:text-purple-400 font-bold">
                {gameTokens} GAME$
              </span>
            </p>
          )}
          <p>
            <span className="font-semibold text-neutral-900 dark:text-neutral-100">
              To:
            </span>{" "}
            <span className="font-mono text-sm break-all">
              {paymentRequirements.payTo}
            </span>
          </p>
          <p>
            <span className="font-semibold text-neutral-900 dark:text-neutral-100">
              For:
            </span>{" "}
            {paymentRequirements.description}
          </p>
        </div>
      </div>

      <button
        disabled={!wallet.connected || isProcessing || success}
        onClick={handlePayment}
        className="w-full bg-blue-600 hover:bg-blue-700 disabled:bg-neutral-600 disabled:cursor-not-allowed text-white font-bold py-4 px-6 rounded-lg transition-colors"
      >
        {isProcessing ? "Processing Payment..." : success ? "Payment Successful!" : "Pay with X402"}
      </button>

      {isProcessing && (
        <p className="text-center text-neutral-600 dark:text-neutral-400">
          Please confirm the transaction in your wallet...
        </p>
      )}

      {success && (
        <div className="bg-green-100 dark:bg-green-900/30 border border-green-300 dark:border-green-700 rounded-lg p-4">
          <p className="text-center text-green-700 dark:text-green-400 font-semibold">
            Payment successful! Redirecting...
          </p>
        </div>
      )}

      {error && (
        <div className="bg-red-100 dark:bg-red-900/30 border border-red-300 dark:border-red-700 rounded-lg p-4">
          <p className="text-center text-red-700 dark:text-red-400">
            {error}
          </p>
        </div>
      )}
    </div>
  );
}

function PaywallContent() {
  const searchParams = useSearchParams();
  const amountParam = searchParams.get('amount');
  const marketId = searchParams.get('marketId');
  const outcome = searchParams.get('outcome') as 'yes' | 'no' | null;
  const [paymentRequirements, setPaymentRequirements] = useState<PaymentRequirements | null>(null);

  useEffect(() => {
    const usdcAmount = amountParam ? parseFloat(amountParam) : 1;
    const amountInSmallestUnit = Math.floor(usdcAmount * 1_000_000).toString(); // Convert to 6 decimals

    // Create description based on market or general purchase
    let description = "GAME$ Token Purchase";
    if (marketId && outcome) {
      description = `${outcome.toUpperCase()} tokens for Market #${marketId}`;
    }

    const requirements: PaymentRequirements = {
      scheme: "exact",
      network: "solana-devnet", // Using Solana devnet
      maxAmountRequired: amountInSmallestUnit,
      resource: `https://402market.com${marketId ? `/market/${marketId}` : ''}`,
      description,
      mimeType: "application/json",
      payTo:
        process.env.NEXT_PUBLIC_X402_PAYMENT_ADDRESS ||
        "9tXDC3VJhKyNsG4VQdZ1234567890abcdefghij", // Solana wallet address (placeholder)
      maxTimeoutSeconds: 600,
      asset: "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU", // USDC on Solana devnet
      outputSchema: undefined,
      extra: undefined, // No extra data for Solana
    };

    setPaymentRequirements(requirements);
  }, [amountParam, marketId, outcome]);

  if (!paymentRequirements) {
    return (
      <div className="min-h-screen flex items-center justify-center p-4 bg-neutral-50 dark:bg-neutral-950">
        <div className="text-neutral-500 dark:text-neutral-400">Loading...</div>
      </div>
    );
  }

  const gameTokens = amountParam ? (parseFloat(amountParam) * 100).toFixed(0) : '100';

  return (
    <div className="min-h-screen flex items-center justify-center p-4 bg-neutral-50 dark:bg-neutral-950">
      <div className="max-w-lg w-full space-y-8">
        <div className="text-center">
          {marketId && outcome ? (
            <>
              <h1 className="text-4xl font-bold mb-4 text-neutral-900 dark:text-neutral-100">
                Buy {outcome === 'yes' ? 'Yes' : 'No'} Tokens
              </h1>
              <p className="text-neutral-600 dark:text-neutral-400 mb-4">
                Purchase prediction market tokens using X402 protocol on Solana
              </p>
              <div className={`rounded-xl p-4 inline-block border-2 ${
                outcome === 'yes'
                  ? 'bg-green-100 dark:bg-green-900/30 border-green-500'
                  : 'bg-red-100 dark:bg-red-900/30 border-red-500'
              }`}>
                <p className="text-sm text-neutral-700 dark:text-neutral-300">You are purchasing</p>
                <p className={`text-3xl font-bold ${
                  outcome === 'yes' ? 'text-green-700 dark:text-green-400' : 'text-red-700 dark:text-red-400'
                }`}>
                  ${amountParam} USDC worth of {outcome?.toUpperCase()} tokens
                </p>
              </div>
            </>
          ) : (
            <>
              <h1 className="text-4xl font-bold mb-4 text-neutral-900 dark:text-neutral-100">
                Buy GAME$ Tokens
              </h1>
              <p className="text-neutral-600 dark:text-neutral-400 mb-4">
                Purchase tokens using X402 protocol on Solana
              </p>
              <div className="bg-gradient-to-r from-purple-900/30 to-blue-900/30 border border-purple-500/50 rounded-xl p-4 inline-block">
                <p className="text-sm text-neutral-600 dark:text-neutral-400">You are purchasing</p>
                <p className="text-3xl font-bold bg-gradient-to-r from-purple-400 to-blue-400 bg-clip-text text-transparent">
                  {gameTokens} GAME$
                </p>
              </div>
            </>
          )}
        </div>

        <PaymentForm
          paymentRequirements={paymentRequirements}
          marketId={marketId}
          outcome={outcome}
        />
      </div>
    </div>
  );
}

export default function Paywall() {
  return (
    <Suspense
      fallback={
        <div className="min-h-screen flex items-center justify-center p-4 bg-neutral-50 dark:bg-neutral-950">
          <div className="text-neutral-500 dark:text-neutral-400">Loading...</div>
        </div>
      }
    >
      <PaywallContent />
    </Suspense>
  );
}
