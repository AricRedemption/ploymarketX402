# 402Market Frontend Implementation

## Overview
Successfully transformed the Play402 template into a 402Market-style prediction market frontend with x402 protocol integration for payments.

## Features Implemented

### 1. Home Page (/)
- **402Market-style layout** with clean, modern design
- **Market grid display** showing all available prediction markets
- **Category filtering**: All, Crypto, Politics, Technology, Finance, Science, Climate, Sports
- **Search functionality** to find markets by question or description
- **Market statistics**: Total volume, active markets, total liquidity
- **Sticky header** with wallet connection via Coinbase OnchainKit
- **Responsive design** for mobile and desktop

### 2. Market Detail Page (/market/[id])
- **Individual market view** with full question and description
- **Yes/No trading cards** showing current probabilities
- **Market statistics**: Volume, liquidity, traders count
- **Buy modal** with:
  - Quick amount selection buttons ($5, $10, $25, $50)
  - Custom amount input
  - Real-time calculation showing:
    - Price per share
    - Number of shares you'll receive
    - Potential return if you win
  - Color-coded UI (green for Yes, red for No)

### 3. Create Market Page (/create-market)
- **Market creation form** with comprehensive validation:
  - Question field (10-200 characters)
  - Description field with resolution criteria
  - Category selection dropdown
  - End date picker (must be in future)
  - Initial liquidity input (10-10,000 USDC)
- **Form validation** with real-time error messages
- **Wallet connection check** before submission
- **x402 payment flow** for initial liquidity provision
- **Responsive design** for mobile and desktop
- **Navigation button** in header (visible on all pages)

### 4. Payment Flow (x402 Integration)
- **Payment page (/paywall)** handles:
  - Prediction market token purchases (Yes/No tokens)
  - Market creation with initial liquidity
  - Legacy GAME$ token purchases (for backward compatibility)
- **x402 Protocol integration** for USDC payments on Base Sepolia
- **Wallet connection** required before payment
- **Transaction signing** with EIP-712 typed data
- **Payment verification and settlement** via x402 facilitator

### 5. Mock Data
- **8 prediction markets** across various categories:
  - Crypto: Bitcoin $100K, Ethereum ETF
  - Politics: 2024 US Election
  - Technology: AI/AGI benchmarks, GPT-5 release
  - Finance: Stock market crash
  - Science: SpaceX Mars landing
  - Climate: Temperature rise

## Technical Stack

### Frontend
- **Next.js 16** with App Router and Turbopack
- **React 19** with TypeScript
- **Tailwind CSS v4** for styling
- **Coinbase OnchainKit** for wallet integration

### Web3 Integration
- **x402 Protocol** for payment processing
- **wagmi** and **viem** for Ethereum interactions
- **Base Sepolia** testnet for transactions
- **USDC** as payment token

## File Structure

```
src/
├── app/
│   ├── page.tsx                 # Home page with market grid
│   ├── market/[id]/page.tsx    # Market detail with trading
│   ├── create-market/page.tsx  # Market creation form
│   ├── paywall/page.tsx        # x402 payment processing
│   ├── actions.ts              # Server actions for payment
│   ├── layout.tsx              # Root layout
│   └── providers.tsx           # OnchainKit provider
├── components/
│   └── MarketCard.tsx          # Market preview card component
├── types/
│   └── market.ts               # TypeScript types for markets
└── lib/
    └── mockData.ts             # Mock market data and helpers
```

## User Flows

### Trading Flow
1. **Browse Markets**: Users land on homepage and see all prediction markets
2. **Filter/Search**: Users can filter by category or search for specific markets
3. **Select Market**: Click on a market card to view details
4. **Choose Outcome**: Click "Buy Yes" or "Buy No" on the market detail page
5. **Select Amount**:
   - Click quick amount buttons ($5, $10, $25, $50)
   - OR enter custom amount
   - See real-time calculation of shares and potential return
6. **Connect Wallet**: Connect wallet if not already connected
7. **Pay with x402**:
   - Review payment details
   - Sign EIP-712 message
   - Payment is verified and settled on-chain
8. **Receive Tokens**: User receives prediction market tokens (implementation pending)

### Market Creation Flow
1. **Click Create Market**: Click "Create Market" button in header
2. **Connect Wallet**: Connect wallet if not already connected (required)
3. **Fill Form**:
   - Enter market question (10-200 characters)
   - Write detailed description with resolution criteria (min 20 characters)
   - Select category from dropdown
   - Choose end date (must be in future)
   - Set initial liquidity (10-10,000 USDC)
4. **Validate**: Form validates in real-time with helpful error messages
5. **Submit**: Click "Create Market" button
6. **Pay Liquidity**: Redirected to payment page to provide initial liquidity via x402
7. **Market Created**: Market is created on-chain (implementation pending)

## Payment Details

### x402 Configuration
- **Network**: Base Sepolia (testnet)
- **Token**: USDC (0x036CbD53842c5426634e7929541eC2318f3dCF7e)
- **Payment Recipient**: Configurable via NEXT_PUBLIC_X402_PAYMENT_ADDRESS
- **Timeout**: 600 seconds (10 minutes)

### Payment Description Format
- Market purchases: "YES tokens for Market #1" or "NO tokens for Market #1"
- Legacy purchases: "GAME$ Token Purchase"

## Next Steps (Not Implemented Yet)

1. **Smart Contract Integration**
   - Connect to actual prediction market contracts
   - Mint Yes/No tokens on-chain after payment
   - Track user positions and balances

2. **Market Resolution**
   - Oracle integration for market outcomes
   - Automated resolution system
   - Payout distribution to winners

3. **User Dashboard**
   - View current positions
   - Track profit/loss
   - Portfolio value calculation

4. **Market Creation**
   - Allow users to create new markets
   - Market validation and moderation
   - Liquidity provision

5. **Enhanced Features**
   - Order book display
   - Price charts and historical data
   - Social features (comments, sharing)
   - Notifications for market updates

## Environment Variables

Create a `.env.local` file:

```bash
NEXT_PUBLIC_X402_PAYMENT_ADDRESS=0x2f2a4eeef6e03854595419adad319740b56a7441
# Add other configuration as needed
```

## Running the Application

```bash
# Install dependencies
npm install

# Run development server
npm run dev

# Build for production
npm run build

# Start production server
npm start
```

## Testing Payment Flow

1. Make sure you have:
   - A wallet with Base Sepolia ETH for gas
   - Base Sepolia USDC for payments (can get from faucets)

2. Navigate to any market
3. Click "Buy Yes" or "Buy No"
4. Select amount and confirm
5. Connect wallet and sign the payment
6. Payment will be verified and settled via x402

## Notes

- All market data is currently mocked (see `src/lib/mockData.ts`)
- Actual contract integration is pending
- Payment flows through x402 protocol successfully
- Ready for backend API integration when contracts are deployed
