import { useState } from 'react';
import { useWallet, useConnection } from '@solana/wallet-adapter-react';
import { PublicKey, Keypair, SystemProgram, SYSVAR_RENT_PUBKEY } from '@solana/web3.js';
import { BN } from '@coral-xyz/anchor';
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { useSolanaProgram } from './useSolanaProgram';
import { mintNoTokenRpc, createMarketRpc } from '@/lib/solana/client';
import { solToLamports, DEPLOYED_CONFIG } from '@/lib/solana/config';

export function useCreateMarket() {
  const wallet = useWallet();
  const { connection } = useConnection();
  const { provider } = useSolanaProgram();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const createMarket = async (
    question: string,
    description: string,
    category: string,
    endDate: Date | null,
    initialLiquiditySol: number
  ) => {
    if (!wallet.publicKey || !provider) {
      throw new Error('Wallet not connected');
    }

    try {
      setLoading(true);
      setError(null);

      // Generate new mint keypairs for Yes and No tokens
      const yesTokenMint = Keypair.generate();
      const noTokenMint = Keypair.generate();

      // Convert end date to slot (approximate - 1 slot = ~400ms on Solana)
      // For simplicity, we'll use current slot + (days * 216000)
      const currentSlot = await connection.getSlot();
      const endingSlot = endDate
        ? currentSlot + Math.floor((endDate.getTime() / 1000 - Date.now() / 1000)) * 2.5 // ~2.5 slots per second
        : 0; // 0 means no end date

      // Convert initial liquidity to lamports
      const initialLiquidity = new BN(solToLamports(initialLiquiditySol));

      // Generate symbols for tokens based on question
      // Metaplex symbol max length is 10 characters
      const baseSymbol = question.substring(0, 6).toUpperCase().replace(/[^A-Z]/g, '') || 'MKT';
      const noSymbol = `${baseSymbol.slice(0, 7)}-NO`; // Max 10 chars: 7 + "-NO" = 10
      const yesSymbol = `${baseSymbol.slice(0, 6)}-YES`; // Max 10 chars: 6 + "-YES" = 10

      // Use the team wallet from deployed config
      const teamWallet = DEPLOYED_CONFIG.teamWallet;

      let signature: string;
      try {
        // Check if global config exists
        const { getGlobalConfigPDA } = await import('@/lib/solana/config');
        const [globalConfigPDA] = getGlobalConfigPDA();
        const configAccount = await connection.getAccountInfo(globalConfigPDA);

        if (!configAccount) {
          throw new Error('Global config account not found. The program may not be initialized properly.');
        }

        console.log('Global config found:', globalConfigPDA.toString());
        console.log('Config account owner:', configAccount.owner.toString());

        console.log('Step 1: Minting NO token...');
        console.log('NO Token Mint:', noTokenMint.publicKey.toString());
        console.log('Team Wallet:', teamWallet.toString());
        console.log('Creator:', wallet.publicKey.toString());

        // Step 1: Mint the NO token first (using RPC - handles signing automatically)
        const noTokenSignature = await mintNoTokenRpc(
          provider,
          noTokenMint,
          noSymbol,
          '' // Empty URI for now
        );

        console.log('NO token transaction sent:', noTokenSignature);
        await connection.confirmTransaction(noTokenSignature, 'confirmed');
        console.log('NO token minted successfully');

        console.log('Step 2: Creating market...');
        // Step 2: Create the market with both tokens
        // Construct CreateMarketParams
        const params = {
          yesSymbol,
          yesUri: '', // Empty URI for now
          startSlot: null, // Start immediately
          endingSlot: endDate ? new BN(endingSlot) : null,
        };

        // Create market (using RPC - handles signing automatically)
        signature = await createMarketRpc(
          provider,
          yesTokenMint, // Pass the Keypair for YES token
          noTokenMint.publicKey, // Pass the PublicKey for NO token (already minted)
          params,
          teamWallet
        );

        console.log('Market creation transaction sent:', signature);
        await connection.confirmTransaction(signature, 'confirmed');
      } catch (txError: any) {
        console.error('Transaction error details:', txError);
        if (txError.logs) {
          console.error('Transaction logs:', txError.logs);
        }

        // Handle specific error types
        if (txError.name === 'WalletSignTransactionError') {
          throw new Error('Transaction signing was cancelled or failed. Please try again.');
        }

        throw new Error(`Market creation failed: ${txError.message || JSON.stringify(txError) || 'Unknown error'}`);
      }

      console.log('Market created successfully:', signature);
      setLoading(false);

      return {
        signature,
        yesTokenMint: yesTokenMint.publicKey.toString(),
        noTokenMint: noTokenMint.publicKey.toString(),
      };
    } catch (err) {
      console.error('Create market error:', err);
      setError(err as Error);
      setLoading(false);
      throw err;
    }
  };

  return {
    createMarket,
    loading,
    error,
  };
}
