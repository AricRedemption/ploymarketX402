'use client';

import { SolanaWalletProvider } from '@/components/SolanaWalletProvider';
import type { ReactNode } from 'react';

export function Providers({ children }: { children: ReactNode }) {
  return (
    <SolanaWalletProvider>
      {children}
    </SolanaWalletProvider>
  );
}
