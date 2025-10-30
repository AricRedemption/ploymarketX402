import { useState } from 'react';
import { useWallet, useConnection } from '@solana/wallet-adapter-react';
import { PublicKey } from '@solana/web3.js';
import { BN } from '@coral-xyz/anchor';
import { useSolanaProgram } from './useSolanaProgram';
import { createAddLiquidityInstruction, createWithdrawLiquidityInstruction } from '@/lib/solana/client';
import { solToLamports } from '@/lib/solana/config';

export function useLiquidity() {
  const wallet = useWallet();
  const { connection } = useConnection();
  const { provider } = useSolanaProgram();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const addLiquidity = async (
    marketAddress: string,
    yesTokenMint: string,
    noTokenMint: string,
    solAmount: number,
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

      const amount = new BN(solToLamports(solAmount));

      console.log('💰 Adding liquidity...', {
        marketAddress,
        solAmount,
        amountLamports: amount.toString(),
      });

      const tx = await createAddLiquidityInstruction(
        provider,
        marketPubkey,
        yesTokenPubkey,
        noTokenPubkey,
        amount,
        teamWalletPubkey
      );

      tx.feePayer = wallet.publicKey;
      tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;

      console.log('🔐 Requesting wallet signature...');
      const signed = await wallet.signTransaction!(tx);

      console.log('✅ Transaction signed, sending to network...');
      const signature = await connection.sendRawTransaction(signed.serialize(), {
        skipPreflight: false,
        preflightCommitment: 'confirmed',
      });

      await connection.confirmTransaction(signature, 'confirmed');

      console.log('✅ Liquidity added successfully:', signature);
      console.log('🔗 View on Solscan:', `https://solscan.io/tx/${signature}?cluster=devnet`);

      setLoading(false);
      return signature;
    } catch (err: any) {
      console.error('❌ Add liquidity error:', err);
      console.error('Error details:', {
        message: err.message,
        code: err.code,
        logs: err.logs,
      });

      setError(err as Error);
      setLoading(false);
      throw err;
    }
  };

  const withdrawLiquidity = async (
    marketAddress: string,
    yesTokenMint: string,
    noTokenMint: string,
    solAmount: number,
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

      const amount = new BN(solToLamports(solAmount));

      console.log('💸 Withdrawing liquidity...', {
        marketAddress,
        solAmount,
        amountLamports: amount.toString(),
      });

      const tx = await createWithdrawLiquidityInstruction(
        provider,
        marketPubkey,
        yesTokenPubkey,
        noTokenPubkey,
        amount,
        teamWalletPubkey
      );

      tx.feePayer = wallet.publicKey;
      tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;

      console.log('🔐 Requesting wallet signature...');
      const signed = await wallet.signTransaction!(tx);

      console.log('✅ Transaction signed, sending to network...');
      const signature = await connection.sendRawTransaction(signed.serialize(), {
        skipPreflight: false,
        preflightCommitment: 'confirmed',
      });

      await connection.confirmTransaction(signature, 'confirmed');

      console.log('✅ Liquidity withdrawn successfully:', signature);
      console.log('🔗 View on Solscan:', `https://solscan.io/tx/${signature}?cluster=devnet`);

      setLoading(false);
      return signature;
    } catch (err: any) {
      console.error('❌ Withdraw liquidity error:', err);
      console.error('Error details:', {
        message: err.message,
        code: err.code,
        logs: err.logs,
      });

      setError(err as Error);
      setLoading(false);
      throw err;
    }
  };

  return {
    addLiquidity,
    withdrawLiquidity,
    loading,
    error,
  };
}
