# 402Market

A decentralized prediction market platform powered by [x402 protocol](https://x402.org) for seamless crypto payments.

## Overview

402Market is a Polymarket-style prediction market frontend that allows users to:
- Browse and trade on prediction markets
- Create new prediction markets
- Make payments using USDC via x402 protocol
- Connect wallets with Coinbase OnchainKit

## Getting Started

First, run the development server:

```bash
npm run dev
# or
yarn dev
# or
pnpm dev
# or
bun dev
```

Open [http://localhost:3000](http://localhost:3000) with your browser to see the result.

## Features

- 🎯 **Browse Markets** - View and search prediction markets across multiple categories
- 💰 **Trade Yes/No Tokens** - Buy outcome tokens with flexible amount selection
- ➕ **Create Markets** - Launch your own prediction markets with custom parameters
- 🔐 **x402 Payments** - Secure USDC payments on Base Sepolia via x402 protocol
- 👛 **Wallet Integration** - Connect with Coinbase OnchainKit
- 📱 **Responsive Design** - Works seamlessly on desktop and mobile

## Tech Stack

- **Next.js 16** with App Router
- **React 19** with TypeScript
- **Tailwind CSS v4** for styling
- **x402 Protocol** for payments
- **Coinbase OnchainKit** for wallet integration
- **wagmi & viem** for Ethereum interactions

## Environment Variables

Create a `.env.local` file:

```bash
NEXT_PUBLIC_X402_PAYMENT_ADDRESS=your_payment_address_here
```

## Documentation

See [402MARKET_IMPLEMENTATION.md](./402MARKET_IMPLEMENTATION.md) for detailed implementation documentation.

## Learn More

- [x402 Protocol Documentation](https://x402.org)
- [Coinbase OnchainKit](https://onchainkit.xyz)
- [Next.js Documentation](https://nextjs.org/docs)
