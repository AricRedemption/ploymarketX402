import {
  Connection,
  PublicKey,
  Transaction,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  Keypair,
} from '@solana/web3.js';
import { Program, AnchorProvider, BN } from '@coral-xyz/anchor';
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  createAssociatedTokenAccountInstruction,
  getAccount,
} from '@solana/spl-token';
import {
  PROGRAM_ID,
  getGlobalConfigPDA,
  getGlobalVaultPDA,
  getMarketPDA,
  getUserInfoPDA,
  connection,
  fromTokenAmount,
  lamportsToSol,
} from './config';
import {
  MarketAccount,
  GlobalConfigAccount,
  UserInfoAccount,
  CreateMarketParams,
  TradeDirection,
  TokenType,
  Market,
  calculatePrice,
} from './types';
import idlJson from '../idl.json';

// Metadata program ID
const METADATA_PROGRAM_ID = new PublicKey('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s');

/**
 * Get the program instance
 */
export function getProgram(provider: AnchorProvider): Program {
  try {
    // Cast the IDL properly for Anchor
    // Use 'as any' to bypass strict type checking - the IDL is valid but Anchor's type system is strict
    const idl = idlJson as any;

    // Program constructor: new Program(idl, provider)
    // In newer Anchor versions (0.30+), the program ID is extracted from the IDL's "address" field
    return new Program(idl, provider);
  } catch (error) {
    console.error('Error creating Anchor program:', error);
    console.log('Falling back to minimal program instance');

    // Fallback: return a minimal program instance
    // This won't work for actual transactions but allows the app to load
    return new Program(idlJson as any, provider);
  }
}

/**
 * Fetch global configuration from blockchain
 */
export async function fetchGlobalConfig(): Promise<GlobalConfigAccount | null> {
  try {
    const [configPDA] = getGlobalConfigPDA();
    const accountInfo = await connection.getAccountInfo(configPDA);

    if (!accountInfo) {
      console.log('Global config not found');
      return null;
    }

    // Parse the account data based on the Config struct
    // This is a simplified version - you'd need to properly deserialize based on Anchor's format
    const data = accountInfo.data;

    // For now, return null and implement proper deserialization when needed
    // You would use program.account.config.fetch(configPDA) with an anchor provider
    return null;
  } catch (error) {
    console.error('Error fetching global config:', error);
    return null;
  }
}

/**
 * Fetch a specific market from blockchain
 */
export async function fetchMarket(
  marketAddress: PublicKey
): Promise<MarketAccount | null> {
  try {
    const accountInfo = await connection.getAccountInfo(marketAddress);

    if (!accountInfo) {
      console.log('Market not found:', marketAddress.toString());
      return null;
    }

    // Return null for now - proper deserialization needs Anchor program instance
    // You would use program.account.market.fetch(marketAddress) with an anchor provider
    return null;
  } catch (error) {
    console.error('Error fetching market:', error);
    return null;
  }
}

/**
 * Fetch all markets from blockchain
 * Note: This function requires a provider with Anchor program instance for proper deserialization
 * Use the useMarkets hook instead which handles this correctly
 */
export async function fetchAllMarkets(): Promise<Market[]> {
  try {
    // Get all market accounts using getProgramAccounts
    const accounts = await connection.getProgramAccounts(PROGRAM_ID, {
      filters: [
        {
          // Filter for Market account discriminator
          memcmp: {
            offset: 0,
            bytes: 'qb7V1wA45pY', // Base58 of Market discriminator from IDL
          },
        },
      ],
    });

    console.log(`Found ${accounts.length} markets on-chain`);

    if (accounts.length === 0) {
      console.log('No markets found on blockchain. Create a market to get started.');
      return [];
    }

    // We cannot properly deserialize without the Anchor program instance
    // Return empty array and let the useMarkets hook handle proper fetching
    console.warn('fetchAllMarkets: Cannot deserialize without Anchor program. Use useMarkets hook instead.');
    return [];
  } catch (error) {
    console.error('Error fetching all markets:', error);
    return [];
  }
}

/**
 * Fetch user info for a specific market
 */
export async function fetchUserInfo(
  userPubkey: PublicKey,
  marketPubkey: PublicKey
): Promise<UserInfoAccount | null> {
  try {
    const [userInfoPDA] = getUserInfoPDA(userPubkey, marketPubkey);
    const accountInfo = await connection.getAccountInfo(userInfoPDA);

    if (!accountInfo) {
      return null;
    }

    // Return null for now - proper deserialization needs Anchor program instance
    return null;
  } catch (error) {
    console.error('Error fetching user info:', error);
    return null;
  }
}

/**
 * Get metadata PDA for a token mint
 */
export function getMetadataPDA(mint: PublicKey): PublicKey {
  const [metadataPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from('metadata'), METADATA_PROGRAM_ID.toBuffer(), mint.toBuffer()],
    METADATA_PROGRAM_ID
  );
  return metadataPDA;
}

/**
 * Get creation time for an account by fetching the block time of its creation transaction
 */
export async function getAccountCreationTime(address: PublicKey): Promise<Date> {
  try {
    // Get signatures for this account (oldest first)
    const signatures = await connection.getSignaturesForAddress(address, {
      limit: 1,
    }, 'confirmed');

    if (signatures.length === 0) {
      // No signatures found, return current time as fallback
      return new Date();
    }

    // Get the oldest signature (creation transaction)
    const oldestSignature = signatures[signatures.length - 1];

    if (oldestSignature.blockTime) {
      // blockTime is in seconds, convert to milliseconds
      return new Date(oldestSignature.blockTime * 1000);
    }

    // Fallback to current time if no blockTime
    return new Date();
  } catch (error) {
    console.error('Error fetching account creation time:', error);
    return new Date();
  }
}

/**
 * Fetch token metadata name and symbol from Metaplex
 */
export async function fetchTokenMetadata(mint: PublicKey): Promise<{ name: string; symbol: string } | null> {
  try {
    const metadataPDA = getMetadataPDA(mint);
    const accountInfo = await connection.getAccountInfo(metadataPDA);

    if (!accountInfo) {
      return null;
    }

    // Parse Metaplex metadata (simplified)
    // Full parsing would require @metaplex-foundation/mpl-token-metadata
    // For now, return null - implement full parsing when needed
    // The structure is: key(1) + update_authority(32) + mint(32) + name(variable) + symbol(variable) + ...

    return null; // Return null for now, will implement full parsing if needed
  } catch (error) {
    console.error('Error fetching token metadata:', error);
    return null;
  }
}

/**
 * Create a transaction to mint NO token
 * This must be called before creating a market
 */
export async function createMintNoTokenInstruction(
  provider: AnchorProvider,
  noTokenKeypair: Keypair,
  noSymbol: string,
  noUri: string
): Promise<Transaction> {
  const program = getProgram(provider);
  const [globalConfig] = getGlobalConfigPDA();
  const [globalVault] = getGlobalVaultPDA();

  const noTokenMetadata = getMetadataPDA(noTokenKeypair.publicKey);
  const globalNoTokenAccount = await getAssociatedTokenAddress(
    noTokenKeypair.publicKey,
    globalVault,
    true
  );

  const tx = await program.methods
    .mintNoToken(noSymbol, noUri)
    .accounts({
      globalConfig,
      globalVault,
      creator: provider.wallet.publicKey,
      noToken: noTokenKeypair.publicKey,
      noTokenMetadataAccount: noTokenMetadata,
      globalNoTokenAccount,
      systemProgram: SystemProgram.programId,
      rent: SYSVAR_RENT_PUBKEY,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      mplTokenMetadataProgram: METADATA_PROGRAM_ID,
    })
    .transaction();

  return tx;
}

/**
 * Mint NO token using RPC (handles signing and sending automatically)
 */
export async function mintNoTokenRpc(
  provider: AnchorProvider,
  noTokenKeypair: Keypair,
  noSymbol: string,
  noUri: string
): Promise<string> {
  const program = getProgram(provider);
  const [globalConfig] = getGlobalConfigPDA();
  const [globalVault] = getGlobalVaultPDA();

  const noTokenMetadata = getMetadataPDA(noTokenKeypair.publicKey);
  const globalNoTokenAccount = await getAssociatedTokenAddress(
    noTokenKeypair.publicKey,
    globalVault,
    true
  );

  // Log accounts for debugging
  console.log('Mint NO Token accounts:', {
    globalConfig: globalConfig.toString(),
    globalVault: globalVault.toString(),
    creator: provider.wallet.publicKey.toString(),
    noToken: noTokenKeypair.publicKey.toString(),
  });

  try {
    const tx = await program.methods
      .mintNoToken(noSymbol, noUri)
      .accounts({
        globalConfig,
        globalVault,
        creator: provider.wallet.publicKey,
        noToken: noTokenKeypair.publicKey,
        noTokenMetadataAccount: noTokenMetadata,
        globalNoTokenAccount,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        mplTokenMetadataProgram: METADATA_PROGRAM_ID,
      })
      .transaction();

    // Set recent blockhash and fee payer
    const { blockhash } = await provider.connection.getLatestBlockhash();
    tx.recentBlockhash = blockhash;
    tx.feePayer = provider.wallet.publicKey;

    // Sign with the noToken keypair first
    tx.partialSign(noTokenKeypair);

    // Sign with wallet (this will prompt user)
    const signedTx = await provider.wallet.signTransaction(tx);

    // Send the fully signed transaction
    const signature = await provider.connection.sendRawTransaction(signedTx.serialize(), {
      skipPreflight: true,
      preflightCommitment: 'confirmed',
    });

    // Wait for confirmation
    await provider.connection.confirmTransaction(signature, 'confirmed');

    return signature;
  } catch (error) {
    console.error('mintNoTokenRpc error:', error);
    throw error;
  }
}

/**
 * Create a transaction to create a new market
 */
export async function createMarketInstruction(
  provider: AnchorProvider,
  yesTokenKeypair: Keypair,
  noTokenMint: PublicKey,
  params: CreateMarketParams,
  teamWallet: PublicKey
): Promise<Transaction> {
  const program = getProgram(provider);
  const [globalConfig] = getGlobalConfigPDA();
  const [globalVault] = getGlobalVaultPDA();
  const [market] = getMarketPDA(yesTokenKeypair.publicKey, noTokenMint);

  const yesTokenMetadata = getMetadataPDA(yesTokenKeypair.publicKey);
  const noTokenMetadata = getMetadataPDA(noTokenMint);

  const globalYesTokenAccount = await getAssociatedTokenAddress(
    yesTokenKeypair.publicKey,
    globalVault,
    true
  );

  const tx = await program.methods
    .createMarket(params)
    .accounts({
      globalConfig,
      globalVault,
      creator: provider.wallet.publicKey,
      yesToken: yesTokenKeypair.publicKey,
      noToken: noTokenMint,
      market,
      yesTokenMetadataAccount: yesTokenMetadata,
      noTokenMetadataAccount: noTokenMetadata,
      globalYesTokenAccount,
      systemProgram: SystemProgram.programId,
      rent: SYSVAR_RENT_PUBKEY,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      mplTokenMetadataProgram: METADATA_PROGRAM_ID,
      teamWallet,
    })
    .transaction();

  return tx;
}

/**
 * Create market using RPC (handles signing and sending automatically)
 */
export async function createMarketRpc(
  provider: AnchorProvider,
  yesTokenKeypair: Keypair,
  noTokenMint: PublicKey,
  params: CreateMarketParams,
  teamWallet: PublicKey
): Promise<string> {
  const program = getProgram(provider);
  const [globalConfig] = getGlobalConfigPDA();
  const [globalVault] = getGlobalVaultPDA();
  const [market] = getMarketPDA(yesTokenKeypair.publicKey, noTokenMint);

  const yesTokenMetadata = getMetadataPDA(yesTokenKeypair.publicKey);
  const noTokenMetadata = getMetadataPDA(noTokenMint);

  const globalYesTokenAccount = await getAssociatedTokenAddress(
    yesTokenKeypair.publicKey,
    globalVault,
    true
  );

  try {
    const tx = await program.methods
      .createMarket(params)
      .accounts({
        globalConfig,
        globalVault,
        creator: provider.wallet.publicKey,
        yesToken: yesTokenKeypair.publicKey,
        noToken: noTokenMint,
        market,
        yesTokenMetadataAccount: yesTokenMetadata,
        noTokenMetadataAccount: noTokenMetadata,
        globalYesTokenAccount,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        mplTokenMetadataProgram: METADATA_PROGRAM_ID,
        teamWallet,
      })
      .transaction();

    // Set recent blockhash and fee payer
    const { blockhash } = await provider.connection.getLatestBlockhash();
    tx.recentBlockhash = blockhash;
    tx.feePayer = provider.wallet.publicKey;

    // Sign with the yesToken keypair first
    tx.partialSign(yesTokenKeypair);

    // Sign with wallet (this will prompt user)
    const signedTx = await provider.wallet.signTransaction(tx);

    // Send the fully signed transaction
    const signature = await provider.connection.sendRawTransaction(signedTx.serialize(), {
      skipPreflight: true,
      preflightCommitment: 'confirmed',
    });

    // Wait for confirmation
    await provider.connection.confirmTransaction(signature, 'confirmed');

    return signature;
  } catch (error) {
    console.error('createMarketRpc error:', error);
    throw error;
  }
}

/**
 * Create a transaction to swap (buy/sell) tokens
 */
export async function createSwapInstruction(
  provider: AnchorProvider,
  marketPubkey: PublicKey,
  yesTokenMint: PublicKey,
  noTokenMint: PublicKey,
  amount: BN,
  direction: TradeDirection,
  tokenType: TokenType,
  minimumReceiveAmount: BN,
  teamWallet: PublicKey
): Promise<Transaction> {
  const program = getProgram(provider);
  const [globalConfig] = getGlobalConfigPDA();
  const [globalVault] = getGlobalVaultPDA();
  const [userInfo] = getUserInfoPDA(provider.wallet.publicKey, marketPubkey);

  const globalYesAta = await getAssociatedTokenAddress(
    yesTokenMint,
    globalVault,
    true
  );
  const globalNoAta = await getAssociatedTokenAddress(
    noTokenMint,
    globalVault,
    true
  );
  const userYesAta = await getAssociatedTokenAddress(
    yesTokenMint,
    provider.wallet.publicKey
  );
  const userNoAta = await getAssociatedTokenAddress(
    noTokenMint,
    provider.wallet.publicKey
  );

  // Create a transaction
  const tx = new Transaction();

  // Check if user's YES token account exists, if not, create it
  try {
    await getAccount(provider.connection, userYesAta);
    console.log('✅ User YES token account already exists:', userYesAta.toString());
  } catch {
    console.log('📝 Creating user YES token account:', userYesAta.toString());
    tx.add(
      createAssociatedTokenAccountInstruction(
        provider.wallet.publicKey, // payer
        userYesAta, // ata
        provider.wallet.publicKey, // owner
        yesTokenMint // mint
      )
    );
  }

  // Check if user's NO token account exists, if not, create it
  try {
    await getAccount(provider.connection, userNoAta);
    console.log('✅ User NO token account already exists:', userNoAta.toString());
  } catch {
    console.log('📝 Creating user NO token account:', userNoAta.toString());
    tx.add(
      createAssociatedTokenAccountInstruction(
        provider.wallet.publicKey, // payer
        userNoAta, // ata
        provider.wallet.publicKey, // owner
        noTokenMint // mint
      )
    );
  }

  // Add the swap instruction
  const swapIx = await program.methods
    .swap(amount, direction, tokenType, minimumReceiveAmount)
    .accounts({
      globalConfig,
      teamWallet,
      market: marketPubkey,
      globalVault,
      yesToken: yesTokenMint,
      noToken: noTokenMint,
      globalYesAta,
      globalNoAta,
      userYesAta,
      userNoAta,
      userInfo,
      user: provider.wallet.publicKey,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    })
    .instruction();

  tx.add(swapIx);

  return tx;
}

/**
 * Create a transaction to add liquidity to a market
 */
export async function createAddLiquidityInstruction(
  provider: AnchorProvider,
  marketPubkey: PublicKey,
  yesTokenMint: PublicKey,
  noTokenMint: PublicKey,
  amount: BN,
  teamWallet: PublicKey
): Promise<Transaction> {
  const program = getProgram(provider);
  const [globalConfig] = getGlobalConfigPDA();
  const [globalVault] = getGlobalVaultPDA();
  const [userInfo] = getUserInfoPDA(provider.wallet.publicKey, marketPubkey);

  const tx = await program.methods
    .addLiquidity(amount)
    .accounts({
      globalConfig,
      teamWallet,
      market: marketPubkey,
      globalVault,
      yesToken: yesTokenMint,
      noToken: noTokenMint,
      userInfo,
      user: provider.wallet.publicKey,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    })
    .transaction();

  return tx;
}

/**
 * Create a transaction to withdraw liquidity from a market
 */
export async function createWithdrawLiquidityInstruction(
  provider: AnchorProvider,
  marketPubkey: PublicKey,
  yesTokenMint: PublicKey,
  noTokenMint: PublicKey,
  amount: BN,
  teamWallet: PublicKey
): Promise<Transaction> {
  const program = getProgram(provider);
  const [globalConfig] = getGlobalConfigPDA();
  const [globalVault] = getGlobalVaultPDA();
  const [userInfo] = getUserInfoPDA(provider.wallet.publicKey, marketPubkey);

  const tx = await program.methods
    .withdrawLiquidity(amount)
    .accounts({
      globalConfig,
      teamWallet,
      market: marketPubkey,
      globalVault,
      yesToken: yesTokenMint,
      noToken: noTokenMint,
      userInfo,
      user: provider.wallet.publicKey,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    })
    .transaction();

  return tx;
}

export { connection };
