"use client";
import { Wallet } from "@coinbase/onchainkit/wallet";
import { useState } from "react";
import { verifyLuckyDrawPayment } from "../actions";
import { PaymentRequirements, PaymentPayload } from "x402/types";
import { preparePaymentHeader } from "x402/client";
import { getNetworkId } from "x402/shared";
import { exact } from "x402/schemes";
import { useAccount, useSignTypedData } from "wagmi";
import Link from "next/link";

const PRIZES = ["100 GAME$", "10 USDC", "Black Myth Wukong"];

function LuckyDrawForm({
  paymentRequirements,
  onPrizeWon,
}: {
  paymentRequirements: PaymentRequirements;
  onPrizeWon: (prize: string) => void;
}) {
  const { address, isConnected } = useAccount();
  const [isProcessing, setIsProcessing] = useState(false);
  const { isError, isSuccess, signTypedDataAsync } = useSignTypedData();

  if (!address || !isConnected) {
    return (
      <div className="text-center space-y-4">
        <Wallet />
        <p className="text-gray-400">
          Please connect your wallet to participate in the lucky draw.
        </p>
      </div>
    );
  }

  const unSignedPaymentHeader = preparePaymentHeader(
    address,
    1,
    paymentRequirements
  );

  const eip712Data = {
    types: {
      TransferWithAuthorization: [
        { name: "from", type: "address" },
        { name: "to", type: "address" },
        { name: "value", type: "uint256" },
        { name: "validAfter", type: "uint256" },
        { name: "validBefore", type: "uint256" },
        { name: "nonce", type: "bytes32" },
      ],
    },
    domain: {
      name: paymentRequirements.extra?.name,
      version: paymentRequirements.extra?.version,
      chainId: getNetworkId(paymentRequirements.network),
      verifyingContract: paymentRequirements.asset as `0x${string}`,
    },
    primaryType: "TransferWithAuthorization" as const,
    message: unSignedPaymentHeader.payload.authorization,
  };

  async function handlePayment() {
    setIsProcessing(true);
    try {
      const signature = await signTypedDataAsync(eip712Data);

      const paymentPayload: PaymentPayload = {
        ...unSignedPaymentHeader,
        payload: {
          ...unSignedPaymentHeader.payload,
          signature,
        },
      };

      const payment: string = exact.evm.encodePayment(paymentPayload);

      const verifyPaymentWithPayment = verifyLuckyDrawPayment.bind(null, payment);
      const result = await verifyPaymentWithPayment();
      console.log("result", result);

      if (result.success && result.prize) {
        onPrizeWon(result.prize);
      }
    } catch (error) {
      console.error("Payment error:", error);
    } finally {
      setIsProcessing(false);
    }
  }

  return (
    <div className="space-y-6">
      <Wallet />

      <div className="bg-gradient-to-br from-yellow-900/50 to-orange-900/50 border border-yellow-600 rounded-xl p-6">
        <h3 className="text-xl font-bold mb-4 text-yellow-400">Draw Details</h3>
        <div className="space-y-2 text-gray-300">
          <p>
            <span className="font-semibold text-white">Cost:</span> $1 USDC per draw
          </p>
          <p>
            <span className="font-semibold text-white">Possible Prizes:</span>
          </p>
          <ul className="list-disc list-inside ml-4 space-y-1">
            {PRIZES.map((prize, idx) => (
              <li key={idx} className="text-yellow-200">{prize}</li>
            ))}
          </ul>
        </div>
      </div>

      <button
        disabled={!isConnected || isProcessing}
        onClick={handlePayment}
        className="w-full bg-gradient-to-r from-yellow-500 to-orange-500 hover:from-yellow-600 hover:to-orange-600 disabled:from-gray-600 disabled:to-gray-600 disabled:cursor-not-allowed text-white font-bold py-4 px-6 rounded-lg transition-colors text-xl"
      >
        {isProcessing ? "Processing..." : "🎲 Spin the Wheel - 1 USDC"}
      </button>

      {isProcessing && (
        <p className="text-center text-gray-400">Signing payment and spinning...</p>
      )}
      {isError && <p className="text-center text-red-400">Payment failed</p>}
    </div>
  );
}

export default function LuckyDrawPage() {
  const [prize, setPrize] = useState<string | null>(null);
  const [showConfetti, setShowConfetti] = useState(false);

  const paymentRequirements: PaymentRequirements = {
    scheme: "exact",
    network: "base-sepolia",
    maxAmountRequired: "1000000", // $1 USDC (6 decimals)
    resource: "https://example.com/lucky-draw",
    description: "Lucky Draw Entry - Win prizes!",
    mimeType: "application/json",
    payTo:
      process.env.NEXT_PUBLIC_X402_PAYMENT_ADDRESS ||
      "0x2f2a4eeef6e03854595419adad319740b56a7441",
    maxTimeoutSeconds: 600,
    asset: "0x036CbD53842c5426634e7929541eC2318f3dCF7e", // Base Sepolia USDC
    outputSchema: undefined,
    extra: {
      name: "USDC",
      version: "2",
    },
  };

  const handlePrizeWon = (wonPrize: string) => {
    setPrize(wonPrize);
    setShowConfetti(true);
  };

  if (prize) {
    return (
      <div className="min-h-screen flex items-center justify-center p-4 bg-gradient-to-br from-yellow-900/20 via-orange-900/20 to-red-900/20">
        <div className="max-w-lg w-full space-y-8 text-center">
          <div className="text-8xl mb-4 animate-bounce">🎉</div>
          <h1 className="text-5xl font-bold mb-4 bg-gradient-to-r from-yellow-400 via-orange-400 to-yellow-400 bg-clip-text text-transparent">
            Congratulations!
          </h1>
          <div className="bg-gradient-to-br from-yellow-900/50 to-orange-900/50 border-4 border-yellow-400 rounded-2xl p-8">
            <p className="text-gray-300 text-xl mb-4">You won:</p>
            <p className="text-6xl font-bold text-white mb-6">{prize}</p>
            <p className="text-yellow-200 text-sm">
              Your prize will be credited to your account shortly!
            </p>
          </div>

          <div className="flex flex-col gap-4">
            <button
              onClick={() => setPrize(null)}
              className="bg-gradient-to-r from-yellow-500 to-orange-500 hover:from-yellow-600 hover:to-orange-600 text-white font-bold py-3 px-6 rounded-lg transition-colors"
            >
              Try Again! 🎲
            </button>
            <Link
              href="/"
              className="bg-gray-700 hover:bg-gray-600 text-white font-bold py-3 px-6 rounded-lg transition-colors"
            >
              Back to Home
            </Link>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen flex items-center justify-center p-4 pb-24">
      <div className="max-w-lg w-full space-y-8">
        <div className="text-center">
          <div className="text-6xl mb-4">🎰</div>
          <h1 className="text-5xl font-bold mb-4 bg-gradient-to-r from-yellow-400 via-orange-400 to-red-400 bg-clip-text text-transparent">
            Lucky Draw
          </h1>
          <p className="text-gray-400 text-lg mb-2">
            Pay 1 USDC and win amazing prizes!
          </p>
          <p className="text-gray-500 text-sm">
            Each draw gives you a chance to win GAME$ tokens, USDC, or exclusive games
          </p>
        </div>

        <LuckyDrawForm
          paymentRequirements={paymentRequirements}
          onPrizeWon={handlePrizeWon}
        />

        <div className="text-center">
          <Link
            href="/"
            className="text-blue-400 hover:text-blue-300 underline"
          >
            ← Back to Home
          </Link>
        </div>
      </div>
    </div>
  );
}
