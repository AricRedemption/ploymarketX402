import { cookies } from "next/headers";

export default async function ProtectedPage() {
  const cookieStore = await cookies();
  const transactionHash = cookieStore.get("payment-tx")?.value;
  return (
    <div className="min-h-screen flex items-center justify-center p-4">
      <div className="max-w-2xl w-full space-y-8 text-center">
        <div className="bg-green-900/20 border border-green-500/30 rounded-xl p-8">
          <div className="w-20 h-20 bg-green-600 rounded-full flex items-center justify-center mx-auto mb-6">
            <svg
              className="w-10 h-10 text-white"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M5 13l4 4L19 7"
              />
            </svg>
          </div>

          <h1 className="text-4xl font-bold mb-4">Payment Successful!</h1>
          <p className="text-xl text-gray-300 mb-6">
            You have successfully purchased <span className="text-purple-400 font-bold">100 GAME$</span> tokens
          </p>

          <div className="bg-gray-900/50 border border-gray-800 rounded-lg p-6 mb-6">
            <h2 className="text-2xl font-bold mb-4">Protected Content</h2>
            <p className="text-gray-400 mb-4">
              You now have access to browse and purchase games using your GAME$ tokens.
            </p>
            <ul className="text-left space-y-2 text-gray-400">
              <li>✓ Access to all game stores</li>
              <li>✓ Participate in lucky draws</li>
              <li>✓ Exclusive deals and discounts</li>
              <li>✓ Priority customer support</li>
            </ul>
          </div>

          {transactionHash && (
            <div className="bg-blue-900/20 border border-blue-500/30 rounded-lg p-6 mb-6">
              <h3 className="text-lg font-semibold mb-2">Transaction Details</h3>
              <div className="flex items-center justify-center gap-2 flex-wrap">
                <p className="text-sm text-gray-400">Transaction Hash:</p>
                <a
                  href={`https://sepolia.basescan.org/tx/${transactionHash}`}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-blue-400 hover:text-blue-300 font-mono text-sm break-all underline"
                >
                  {transactionHash}
                </a>
              </div>
            </div>
          )}

          <div className="flex gap-4 justify-center">
            <a
              href="/"
              className="px-6 py-3 bg-gray-800 text-white font-semibold rounded-lg hover:bg-gray-700 transition-all"
            >
              Back to Home
            </a>
            <a
              href="/games"
              className="px-6 py-3 bg-gradient-to-r from-purple-600 to-blue-600 text-white font-semibold rounded-lg hover:from-purple-700 hover:to-blue-700 transition-all"
            >
              Browse Games
            </a>
          </div>
        </div>
      </div>
    </div>
  );
}
