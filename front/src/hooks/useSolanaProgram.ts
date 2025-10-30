import { useMemo } from 'react';
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { AnchorProvider } from '@coral-xyz/anchor';
import { getProgram } from '@/lib/solana/client';

/**
 * Hook to get the Anchor program instance
 */
export function useSolanaProgram() {
  const { connection } = useConnection();
  const wallet = useWallet();

  const provider = useMemo(() => {
    if (!wallet.publicKey) return null;

    return new AnchorProvider(
      connection,
      wallet as any,
      { commitment: 'confirmed' }
    );
  }, [connection, wallet]);

  const program = useMemo(() => {
    if (!provider) return null;
    return getProgram(provider);
  }, [provider]);

  return { program, provider, connection };
}
