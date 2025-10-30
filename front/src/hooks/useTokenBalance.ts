import { useState, useEffect } from 'react';
import { PublicKey } from '@solana/web3.js';
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { getAssociatedTokenAddress, getAccount } from '@solana/spl-token';

export interface TokenBalance {
  yesBalance: number;
  noBalance: number;
  loading: boolean;
  error: Error | null;
}

/**
 * Hook to fetch user's token balances for a specific market
 */
export function useTokenBalance(
  yesTokenMint: PublicKey | null,
  noTokenMint: PublicKey | null
): TokenBalance {
  const { connection } = useConnection();
  const { publicKey } = useWallet();
  const [yesBalance, setYesBalance] = useState<number>(0);
  const [noBalance, setNoBalance] = useState<number>(0);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    let mounted = true;

    async function fetchBalances() {
      if (!publicKey || !yesTokenMint || !noTokenMint) {
        if (mounted) {
          setYesBalance(0);
          setNoBalance(0);
          setLoading(false);
        }
        return;
      }

      try {
        setLoading(true);
        setError(null);

        // Check if tokens are set (not default PublicKey)
        const defaultPubkey = new PublicKey('11111111111111111111111111111111');
        const hasYesToken = !yesTokenMint.equals(defaultPubkey);
        const hasNoToken = !noTokenMint.equals(defaultPubkey);

        console.log('==========================================');
        console.log('🔍 Fetching token balances for:', {
          user: publicKey.toString(),
          yesToken: yesTokenMint.toString(),
          noToken: noTokenMint.toString(),
          hasYesToken,
          hasNoToken,
        });
        console.log('==========================================');

        let yesAmount = 0;
        let noAmount = 0;

        // Fetch YES token balance
        if (hasYesToken) {
          try {
            const yesAta = await getAssociatedTokenAddress(
              yesTokenMint,
              publicKey
            );
            console.log('✅ YES Token ATA:', yesAta.toString());
            console.log('🔗 View on Solscan:', `https://solscan.io/account/${yesAta.toString()}?cluster=devnet`);

            const yesAccount = await getAccount(connection, yesAta);
            yesAmount = Number(yesAccount.amount) / 1e9; // Convert from smallest unit
            console.log('✅ YES Token Balance:', yesAmount, 'tokens (raw:', yesAccount.amount.toString(), 'lamports)');
            console.log('   Account Owner:', yesAccount.owner.toString());
            console.log('   Mint:', yesAccount.mint.toString());
          } catch (err: any) {
            // Account doesn't exist or has no balance
            console.log('❌ No YES token account or error:', err.message);
            console.log('   Error details:', err);
          }
        }

        // Fetch NO token balance
        if (hasNoToken) {
          try {
            const noAta = await getAssociatedTokenAddress(
              noTokenMint,
              publicKey
            );
            console.log('✅ NO Token ATA:', noAta.toString());
            console.log('🔗 View on Solscan:', `https://solscan.io/account/${noAta.toString()}?cluster=devnet`);

            const noAccount = await getAccount(connection, noAta);
            noAmount = Number(noAccount.amount) / 1e9; // Convert from smallest unit
            console.log('✅ NO Token Balance:', noAmount, 'tokens (raw:', noAccount.amount.toString(), 'lamports)');
            console.log('   Account Owner:', noAccount.owner.toString());
            console.log('   Mint:', noAccount.mint.toString());
          } catch (err: any) {
            // Account doesn't exist or has no balance
            console.log('❌ No NO token account or error:', err.message);
            console.log('   Error details:', err);
          }
        }

        if (mounted) {
          console.log('📊 Final Balances:', {
            YES: yesAmount,
            NO: noAmount,
          });
          console.log('==========================================\n');

          setYesBalance(yesAmount);
          setNoBalance(noAmount);
          setLoading(false);
        }
      } catch (err) {
        console.error('Error fetching token balances:', err);
        if (mounted) {
          setError(err as Error);
          setLoading(false);
        }
      }
    }

    fetchBalances();

    // Refresh balances every 10 seconds
    const interval = setInterval(fetchBalances, 10000);

    return () => {
      mounted = false;
      clearInterval(interval);
    };
  }, [connection, publicKey, yesTokenMint, noTokenMint]);

  return { yesBalance, noBalance, loading, error };
}
