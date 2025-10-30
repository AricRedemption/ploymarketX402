import { useState } from 'react';
import { useWallet, useConnection } from '@solana/wallet-adapter-react';
import { PublicKey, Transaction } from '@solana/web3.js';
import { BN } from '@coral-xyz/anchor';
import { useSolanaProgram } from './useSolanaProgram';
import { createSwapInstruction } from '@/lib/solana/client';
import { TradeDirection, TokenType } from '@/lib/solana/types';
import { solToLamports } from '@/lib/solana/config';

export function useSwap() {
  const wallet = useWallet();
  const { connection } = useConnection();
  const { provider } = useSolanaProgram();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const buyTokens = async (
    marketAddress: string,
    yesTokenMint: string,
    noTokenMint: string,
    tokenType: 'yes' | 'no',
    solAmount: number, // Amount of SOL to spend
    teamWallet: string
  ) => {
    if (!wallet.publicKey || !provider) {
      throw new Error('Wallet not connected');
    }

    try {
      setLoading(true);
      setError(null);

      const marketPubkey = new PublicKey(marketAddress);
      const yesTokenPubkey = new PublicKey(yesTokenMint);
      const noTokenPubkey = new PublicKey(noTokenMint);
      const teamWalletPubkey = new PublicKey(teamWallet);

      // Convert SOL to lamports
      const amount = new BN(solToLamports(solAmount));

      // Set minimum receive amount (with 1% slippage tolerance)
      const minimumReceiveAmount = new BN(0); // Calculate based on expected output

      console.log('🔄 Creating swap transaction...', {
        marketAddress,
        tokenType,
        solAmount,
        amountLamports: amount.toString(),
      });

      // Create swap instruction
      const tx = await createSwapInstruction(
        provider,
        marketPubkey,
        yesTokenPubkey,
        noTokenPubkey,
        amount,
        TradeDirection.Buy,
        tokenType === 'yes' ? TokenType.Yes : TokenType.No,
        minimumReceiveAmount,
        teamWalletPubkey
      );

      console.log('✅ Swap transaction created, preparing to send...');
      console.log('   Instructions:', tx.instructions.length);

      // Send transaction
      tx.feePayer = wallet.publicKey;
      tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;

      console.log('🔐 Requesting wallet signature...');
      const signed = await wallet.signTransaction!(tx);

      console.log('✅ Transaction signed, sending to network...');
      const signature = await connection.sendRawTransaction(signed.serialize(), {
        skipPreflight: false,
        preflightCommitment: 'confirmed',
      });

      // Confirm transaction
      await connection.confirmTransaction(signature, 'confirmed');

      console.log('✅ Swap successful:', signature);
      console.log('Transaction details:', {
        signature,
        marketAddress,
        tokenType,
        solAmount,
        userWallet: wallet.publicKey.toString(),
        yesTokenMint,
        noTokenMint,
      });
      console.log('🔗 View on Solscan:', `https://solscan.io/tx/${signature}?cluster=devnet`);

      setLoading(false);

      return signature;
    } catch (err: any) {
      console.error('❌ Swap error:', err);
      console.error('Error details:', {
        message: err.message,
        code: err.code,
        logs: err.logs,
      });

      // Try to extract more meaningful error info
      if (err.message) {
        console.error('Error message:', err.message);
      }
      if (err.logs) {
        console.error('Transaction logs:', err.logs);
      }

      setError(err as Error);
      setLoading(false);
      throw err;
    }
  };

  const sellTokens = async (
    marketAddress: string,
    yesTokenMint: string,
    noTokenMint: string,
    tokenType: 'yes' | 'no',
    tokenAmount: number, // Amount of tokens to sell
    teamWallet: string
  ) => {
    if (!wallet.publicKey || !provider) {
      throw new Error('Wallet not connected');
    }

    try {
      setLoading(true);
      setError(null);

      const marketPubkey = new PublicKey(marketAddress);
      const yesTokenPubkey = new PublicKey(yesTokenMint);
      const noTokenPubkey = new PublicKey(noTokenMint);
      const teamWalletPubkey = new PublicKey(teamWallet);

      // Convert token amount (assuming 9 decimals)
      const amount = new BN(tokenAmount * 1e9);

      // Set minimum receive amount
      const minimumReceiveAmount = new BN(0);

      // Create swap instruction
      const tx = await createSwapInstruction(
        provider,
        marketPubkey,
        yesTokenPubkey,
        noTokenPubkey,
        amount,
        TradeDirection.Sell,
        tokenType === 'yes' ? TokenType.Yes : TokenType.No,
        minimumReceiveAmount,
        teamWalletPubkey
      );

      // Send transaction
      tx.feePayer = wallet.publicKey;
      tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;

      const signed = await wallet.signTransaction!(tx);
      const signature = await connection.sendRawTransaction(signed.serialize());

      await connection.confirmTransaction(signature, 'confirmed');

      console.log('Sell successful:', signature);
      setLoading(false);

      return signature;
    } catch (err) {
      console.error('Sell error:', err);
      setError(err as Error);
      setLoading(false);
      throw err;
    }
  };

  return {
    buyTokens,
    sellTokens,
    loading,
    error,
  };
}
