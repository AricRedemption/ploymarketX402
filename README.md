# X402 Prediction Market

A decentralized prediction market platform built on Solana, enabling users to trade on binary outcome events with provably fair pricing and robust security guarantees.

## Overview

X402 Prediction Market is a production-ready prediction market protocol inspired by Polymarket. Users can create markets on any binary outcome event (YES/NO), trade prediction tokens with dynamic pricing, and provide liquidity to earn fees. Built on Solana blockchain with USDC as collateral for fast, low-cost transactions.

**What makes this unique**: This project showcases innovative **x402 payment protocol integration** with **dual-chain support** (Solana + Base), enabling seamless USDC payments for prediction markets, lucky draw games, and token purchases - all with a unified payment experience.

**Program ID**: `EgEc7fuse6eQ3UwqeWGFncDtbTwozWCy4piydbeRaNrU`

## What is a Prediction Market?

A prediction market allows users to bet on the outcome of future events by trading YES/NO tokens. Token prices reflect the crowd's belief about probability:

- If YES tokens trade at $0.70, the market believes there's a 70% chance the event will happen
- If NO tokens trade at $0.30, there's a 30% chance it won't happen
- Prices always sum to $1.00 (100% probability)

When the event concludes, winners redeem their tokens for $1.00 each, while losers get nothing.

## How to Play

### For Traders

**1. Buy Prediction Tokens**
- Choose a market (e.g., "Will Bitcoin reach $100k by December?")
- Buy YES tokens if you think it will happen, or NO tokens if you don't
- Prices adjust based on supply and demand using LMSR pricing algorithm
- Set slippage protection to avoid unexpected price changes

**2. Trade and Arbitrage**
- Buy low, sell high as market sentiment changes
- Use "Complete Set" mechanism: deposit 1 USDC to receive 1 YES + 1 NO token
- Split the set and sell the side you don't want
- Redeem complete sets (1 YES + 1 NO) back to 1 USDC anytime

**3. Claim Winnings**
- Wait for market resolution after event ends
- Winners redeem winning tokens for 1 USDC each
- If you held 100 YES tokens and YES wins, you get 100 USDC

### For Liquidity Providers

**1. Provide Liquidity**
- Deposit YES tokens, NO tokens, and USDC into market pools
- Receive LP (Liquidity Provider) shares representing your ownership
- Earn a portion of every trade's fees proportional to your share

**2. Manage Position**
- Add more liquidity anytime to increase earnings
- Withdraw liquidity and get back your proportional share of pool assets
- Claim accumulated trading fees separately

**3. Fair Fee Distribution**
- Fees are tracked cumulatively to prevent timing manipulation
- LPs who join early don't have unfair advantages over later joiners
- Earn passive income from market trading activity

### For Market Creators

**1. Create New Markets**
- Define a binary outcome event with clear YES/NO resolution criteria
- Set market start time and end time
- Configure initial liquidity depth (affects price stability)
- Optionally restricted by whitelist for quality control

**2. Seed Initial Liquidity**
- Provide initial YES/NO tokens and USDC to bootstrap trading
- Become the first LP and earn fees from all subsequent trades
- Markets with deeper liquidity have lower slippage

## Key Features

### Automated Market Making (AMM)
- **LMSR Pricing**: Logarithmic Market Scoring Rule ensures prices = probabilities
- **Dynamic Pricing**: Prices adjust automatically based on token inventory
- **Slippage Protection**: Set minimum receive amounts to protect against price movements
- **Continuous Trading**: Buy/sell anytime during market lifetime

### Complete Set Mechanism
- **Mint Complete Set**: 1 USDC → 1 YES + 1 NO token (always available)
- **Redeem Complete Set**: 1 YES + 1 NO → 1 USDC (always available)
- **Arbitrage Opportunity**: Keep prices balanced through free minting/redemption
- **100% Collateralized**: Every token pair is fully backed by USDC

### Liquidity Pools
- **Multi-Asset Pools**: Hold YES tokens, NO tokens, and USDC
- **LP Shares**: Proportional ownership tracked via fungible shares
- **Trading Fees**: Configurable fees distributed to LPs
- **Fair Withdrawal**: Claim accumulated fees and withdraw liquidity anytime

### Market Resolution
- **Admin Settlement**: Trusted authority resolves market outcome after event ends
- **Winner Takes All**: Winning token redeems for 1 USDC, losing token becomes worthless
- **Draw Support**: Optional 50/50 split if event is inconclusive
- **Pool Settlement**: LPs can withdraw after resolution without waiting for all claims

## Platform Features

### Main Dashboard
- **Market Explorer**: Browse all active prediction markets with real-time pricing
- **Category Filters**: Filter markets by Finance, Sports, Politics, Technology, Entertainment, and more
- **Search Functionality**: Find specific markets by keywords in questions or descriptions
- **Live Statistics**: Track 24-hour volume, active markets count, and total liquidity
- **Responsive Design**: Fully optimized for desktop and mobile devices

### Lucky Draw Game (Base Sepolia)

An innovative gamification feature powered by x402 on Base network:

- **Entry Fee**: Pay 1 USDC using x402 payment protocol
- **Prize Pool**: Win 100 GAME$ tokens, 10 USDC, or Black Myth Wukong game
- **EVM Wallet Integration**: Uses Coinbase OnchainKit for seamless Base connection
- **Instant Results**: Immediate prize determination with celebration animations
- **Provably Fair**: On-chain payment verification before prize distribution

### Payment Gateway (Solana)

Universal payment interface for market participation:

- **Market Token Purchase**: Buy YES or NO tokens for any prediction market
- **General Token Sales**: Purchase GAME$ utility tokens for platform usage
- **Dynamic Pricing**: Real-time USDC amount calculation based on token quantity
- **Dual Wallet Support**: Solana Wallet Adapter for SPL token transactions
- **Transaction Tracking**: View payment details and confirmation status
- **Automatic Redirects**: Return to market page after successful payment

### x402 Payment Benefits

- **Gasless Transactions**: Minimize transaction costs for users
- **Cross-Chain Flexibility**: Support both Solana and EVM ecosystems
- **Standardized Interface**: Consistent payment experience across features
- **Server-Side Verification**: Enhanced security with backend payment validation
- **Future-Proof**: Easily extensible to additional chains and tokens

## Security Features

### Collateral Safety
- **1:1 Guarantee**: Every YES+NO token pair is backed by exactly 1 USDC
- **No Fractional Reserve**: Impossible to over-issue tokens
- **Segregated Accounting**: Dual ledger system separates settlement from trading
- **Audit Trail**: All deposits and withdrawals are transparently tracked on-chain

### Access Control
- **Multi-Tier Authority**: Admin, creators, and LPs have different permission levels
- **Two-Step Transfer**: Authority handover requires nomination and acceptance
- **Emergency Pause**: Admin can halt all trading in crisis situations
- **Whitelist Mode**: Optional creator access control for curated markets

### Trading Protection
- **Slippage Limits**: Users specify minimum acceptable receive amounts
- **Time Windows**: Trading only allowed between market start and end times
- **Reentrancy Guards**: Prevents recursive call exploits
- **Token Validation**: Markets verify mint authorities to prevent fake tokens

### Smart Contract Security
- **Deterministic Math**: Fixed-point arithmetic ensures consistent calculations
- **PDA Design**: Program Derived Addresses prevent account substitution attacks
- **Authority Checks**: Every admin operation validates signer permissions
- **Token Reuse Prevention**: Markets reject tokens with existing supply

## Why LMSR Pricing?

Traditional AMMs like Uniswap use constant product (x·y=k), but prediction markets need special properties:

**LMSR Advantages:**
- Prices directly represent probabilities (0-100%)
- Fixed liquidity depth parameter for predictable slippage
- Designed specifically for binary outcomes
- Better price discovery for rare events

**How It Works:**
- Market maintains YES and NO token reserves
- Prices calculated from logarithmic cost function
- Buying YES tokens increases YES price, decreases NO price
- Prices always sum to exactly 100%

## Fee Structure

**Trading Fees:**
- Platform Fee: Configurable percentage sent to protocol treasury
- LP Fee: Configurable percentage distributed to liquidity providers
- Separate rates for buys and sells (allows asymmetric pricing)
- All fees deducted from USDC amounts (not tokens)

**Example (1% platform + 0.5% LP fees):**
- User sends 1000 USDC to buy YES tokens
- 10 USDC goes to platform treasury
- 5 USDC goes to LP fee pool
- 985 USDC used for LMSR pricing to calculate tokens received

## Technical Highlights

### Solana Smart Contract
- Built with Anchor framework (Rust)
- Deployed on Solana Devnet (mainnet-ready)
- USDC collateral (6 decimal precision)
- Gas-optimized compute units

### Frontend Application
- Next.js 15 with React
- Tailwind CSS v4 for styling
- Solana Wallet Adapter integration
- Coinbase OnchainKit for wallet UX

### x402 Payment Protocol Integration

This project showcases **innovative x402 payment protocol** with unique dual-chain architecture:

**Dual-Chain Payment Support:**
- **Solana Devnet**: Primary chain for prediction market trading (low gas fees, high speed)
- **Base Sepolia**: Secondary chain for gaming features like Lucky Draw (EVM compatibility)
- Unified payment interface across both chains via x402 protocol

**Solana Payment Flow:**
- SPL Token transfers (USDC with 6 decimals)
- Automatic Associated Token Account creation for recipients
- Signed transaction encoding for x402 verification
- Used for: Market token purchases, liquidity provision, general token sales

**Base (EVM) Payment Flow:**
- EIP-712 typed data signing for USDC authorization
- `TransferWithAuthorization` for gasless payments
- Coinbase OnchainKit wallet integration
- Used for: Lucky Draw participation, premium features

**Key Features:**
- Standardized `PaymentRequirements` interface for both chains
- Server-side payment verification before transaction submission
- Seamless user experience regardless of blockchain
- Extensible to additional chains and tokens

### Innovative Architecture
- **Dual Ledger System**: Separates collateral safety from trading efficiency
- **Fair LP Distribution**: Cumulative fee tracking prevents timing attacks
- **Fixed-Point Math**: Deterministic calculations across all platforms
- **Binary Search Optimization**: Efficient token amount calculations
- **Cross-Chain Payment Gateway**: x402-powered payments on Solana and Base

## Use Cases

**Financial Markets:**
- Cryptocurrency price predictions
- Stock market movements
- Economic indicators (inflation, GDP)

**Sports & Entertainment:**
- Game outcomes and championships
- Award show winners
- Box office performance

**Politics & Governance:**
- Election results
- Policy passage predictions
- Approval ratings

**Science & Technology:**
- Product launch dates
- Scientific discoveries
- Technology adoption rates

**Community & Social:**
- Local event outcomes
- Social media trends
- Community decisions

## Roadmap

- Mainnet deployment with smart contract audit
- Multi-outcome markets (beyond binary YES/NO)
- Decentralized oracle integration for automated resolution
- Mobile application with push notifications
- Cross-chain liquidity aggregation
- Advanced order types (limit orders, stop-loss)
- NFT-based market creation rights and governance

## Technical Stack

- **Blockchain**: Solana (Devnet)
- **Smart Contract**: Anchor Framework (Rust)
- **Frontend**: Next.js 15 + TypeScript
- **Styling**: Tailwind CSS v4
- **Wallet**: Solana Wallet Adapter
- **Payment**: x402 Protocol
- **Collateral**: USDC (SPL Token)

## Getting Started

Visit our frontend application to start trading prediction markets:

1. Connect your Solana wallet (or Coinbase wallet for Lucky Draw on Base)
2. Fund your wallet with USDC
3. Browse available markets or create your own
4. Buy YES/NO tokens based on your predictions using x402 payment gateway
5. Trade or provide liquidity to earn fees
6. Try the Lucky Draw feature for a chance to win prizes
7. Claim winnings after market resolution

## Why x402 Integration Matters

This project demonstrates the power of the **x402 payment protocol** in enabling seamless cross-chain USDC payments for decentralized applications:

**For Users:**

- Single payment interface across Solana and Base chains
- Reduced transaction friction with gasless payments
- Consistent experience regardless of blockchain
- Enhanced security with server-side verification

**For Developers:**

- Standardized payment requirements interface
- Easy integration with multiple chains
- Flexible and extensible architecture
- Production-ready payment verification flows

**For the Ecosystem:**

- Bridges Solana DeFi with EVM ecosystems
- Enables innovative cross-chain features (prediction markets + gaming)
- Demonstrates real-world utility of payment protocols
- Paves the way for multi-chain DeFi applications

**Unique Implementation:**

This is one of the first projects to combine:

- Solana prediction markets with LMSR pricing
- x402 payment protocol for dual-chain support
- Unified UX for complex DeFi operations
- Gaming features (Lucky Draw) alongside financial products

## License

MIT License

## Acknowledgments

- Built with Anchor Framework
- Inspired by Polymarket
- LMSR pricing based on research by Robin Hanson
- Integrated with x402 Payment Protocol

---

**Built for Hackathon - Demonstrating advanced Solana DeFi with production-grade security and innovative market mechanics**
