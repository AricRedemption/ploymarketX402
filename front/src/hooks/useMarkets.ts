import { useState, useEffect } from 'react';
import { PublicKey } from '@solana/web3.js';
import { useConnection } from '@solana/wallet-adapter-react';
import { Market } from '@/lib/solana/types';
import { fetchAllMarkets, getAccountCreationTime, fetchTokenMetadata } from '@/lib/solana/client';
import { useSolanaProgram } from './useSolanaProgram';
import { getMarketTitle } from '@/lib/marketTitles';

/**
 * Hook to fetch all markets from the blockchain
 */
export function useMarkets() {
  const { connection } = useConnection();
  const { program } = useSolanaProgram();
  const [markets, setMarkets] = useState<Market[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    let mounted = true;

    async function loadMarkets() {
      if (!program) {
        // Use fallback without program
        try {
          const fetchedMarkets = await fetchAllMarkets();
          if (mounted) {
            setMarkets(fetchedMarkets);
            setLoading(false);
          }
        } catch (err) {
          console.error('Error loading markets:', err);
          if (mounted) {
            setError(err as Error);
            setLoading(false);
          }
        }
        return;
      }

      try {
        setLoading(true);

        // Fetch all Market accounts using the program
        const marketAccounts = await program.account.market.all();

        // Get current slot for end date calculations
        const currentSlot = await connection.getSlot();

        // Fetch creation times and metadata for all markets in parallel
        const creationTimePromises = marketAccounts.map(({ publicKey }: { publicKey: PublicKey }) =>
          getAccountCreationTime(publicKey)
        );

        const metadataPromises = marketAccounts.map(({ account }: { account: any }) =>
          account.yesTokenMint ? fetchTokenMetadata(account.yesTokenMint) : Promise.resolve(null)
        );

        const [creationTimes, metadataList] = await Promise.all([
          Promise.all(creationTimePromises),
          Promise.all(metadataPromises)
        ]);

        const parsedMarkets: Market[] = marketAccounts.map((account: any, index: number) => {
          const data = account.account;
          const pubkey = account.publicKey;

          // Calculate prices from reserves
          const totalReserves = data.realYesSolReserves.toNumber() + data.realNoSolReserves.toNumber();
          const yesPrice = totalReserves > 0
            ? data.realYesSolReserves.toNumber() / totalReserves
            : 0.5; // Default to 50% if no liquidity
          const noPrice = 1 - yesPrice;

          // Calculate liquidity (total SOL in both pools)
          const liquidity = (data.realYesSolReserves.toNumber() + data.realNoSolReserves.toNumber()) / 1e9;

          // Calculate volume from trading activity
          // For now, estimate based on liquidity (this should be tracked via events in production)
          const volume = liquidity * 0.5; // Conservative estimate

          // Check if tokens are set (not default PublicKey)
          const defaultPubkey = '11111111111111111111111111111111';
          const yesTokenStr = data.yesTokenMint.toString();
          const noTokenStr = data.noTokenMint.toString();
          const hasYesToken = yesTokenStr !== defaultPubkey;
          const hasNoToken = noTokenStr !== defaultPubkey;

          // Generate market identifier from address
          const marketId = pubkey.toString().slice(0, 8);
          const marketAddress = pubkey.toString();

          // Get token metadata for this market
          const metadata = metadataList[index];

          // Check if we have a custom title for this market
          const marketTitleConfig = getMarketTitle(marketAddress);

          // Use configured title if available, otherwise use metadata or generic title
          let question: string;
          let description: string;
          let category: string;

          if (marketTitleConfig) {
            // Use the configured market title
            question = marketTitleConfig.question;
            description = marketTitleConfig.description || `Market: ${marketId}`;
            category = marketTitleConfig.category || 'Crypto';
          } else if (metadata && metadata.name) {
            // Use the token metadata name as the market title
            question = metadata.name.trim();

            // Create description from metadata
            description = `Market: ${marketId}`;
            if (hasYesToken && hasNoToken) {
              description += ` • YES: ${metadata.symbol || yesTokenStr.slice(0, 4)}`;
            }

            category = 'Crypto';
          } else {
            // Generate generic title for markets without configuration or metadata
            if (hasYesToken && hasNoToken) {
              question = `Binary Prediction Market ${marketId}`;
            } else {
              question = `Prediction Market ${marketId} (Tokens not initialized)`;
            }

            // Create detailed description with token info
            description = `Market: ${marketId}...${pubkey.toString().slice(-4)}`;
            if (hasYesToken && hasNoToken) {
              description += ` • YES: ${yesTokenStr.slice(0, 4)}...${yesTokenStr.slice(-4)}`;
              description += ` • NO: ${noTokenStr.slice(0, 4)}...${noTokenStr.slice(-4)}`;
            }
            description += ` • Supply: ${(data.tokenYesTotalSupply.toNumber() / 1e9).toFixed(0)}B`;

            category = 'Crypto';
          }

          // Calculate end date from ending slot if available
          let endDate: Date | null = null;
          if (data.endingSlot && !data.endingSlot.isZero()) {
            // The endingSlot is an absolute slot number
            // Calculate remaining slots from current slot
            const slotsRemaining = data.endingSlot.toNumber() - currentSlot;
            const secondsRemaining = slotsRemaining / 2.5; // ~2.5 slots per second
            // Only set endDate if it's in the future
            if (secondsRemaining > 0) {
              endDate = new Date(Date.now() + secondsRemaining * 1000);
            }
          }

          return {
            id: pubkey.toString(),
            address: pubkey,
            question,
            description,
            category,
            createdAt: creationTimes[index],
            endDate,
            volume,
            liquidity,
            yesPrice,
            noPrice,
            yesTokenMint: data.yesTokenMint,
            noTokenMint: data.noTokenMint,
            creator: data.creator,
            status: data.isCompleted ? 'closed' : 'active',
            tags: [],
            contractData: data,
          };
        });

        // Sort markets by creation time (newest first)
        parsedMarkets.sort((a, b) => b.createdAt.getTime() - a.createdAt.getTime());

        if (mounted) {
          setMarkets(parsedMarkets);
          setLoading(false);
        }
      } catch (err) {
        console.error('Error loading markets:', err);
        if (mounted) {
          setError(err as Error);
          setLoading(false);
        }
      }
    }

    loadMarkets();

    return () => {
      mounted = false;
    };
  }, [program]);

  const refetch = async () => {
    setLoading(true);
    setError(null);
    // Trigger re-fetch by updating a dependency
  };

  return { markets, loading, error, refetch };
}

/**
 * Hook to fetch a single market by ID
 */
export function useMarket(marketId: string | null) {
  const { connection } = useConnection();
  const { program } = useSolanaProgram();
  const [market, setMarket] = useState<Market | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    let mounted = true;

    async function loadMarket() {
      if (!marketId || !program) {
        if (mounted) setLoading(false);
        return;
      }

      try {
        setLoading(true);
        const marketPubkey = new PublicKey(marketId);
        const data = await program.account.market.fetch(marketPubkey);

        // Get current slot for end date calculation
        const currentSlot = await connection.getSlot();

        // Fetch creation time and metadata
        const [createdAt, metadata] = await Promise.all([
          getAccountCreationTime(marketPubkey),
          data.yesTokenMint ? fetchTokenMetadata(data.yesTokenMint) : Promise.resolve(null)
        ]);

        // Calculate prices
        const totalReserves = data.realYesSolReserves.toNumber() + data.realNoSolReserves.toNumber();
        const yesPrice = totalReserves > 0
          ? data.realYesSolReserves.toNumber() / totalReserves
          : 0.5; // Default to 50% if no liquidity
        const noPrice = 1 - yesPrice;

        const liquidity = (data.realYesSolReserves.toNumber() + data.realNoSolReserves.toNumber()) / 1e9;
        const volume = liquidity * 0.5; // Conservative estimate

        // Check if tokens are set (not default PublicKey)
        const defaultPubkey = '11111111111111111111111111111111';
        const yesTokenStr = data.yesTokenMint.toString();
        const noTokenStr = data.noTokenMint.toString();
        const hasYesToken = yesTokenStr !== defaultPubkey;
        const hasNoToken = noTokenStr !== defaultPubkey;

        // Generate market identifier
        const marketIdShort = marketId.slice(0, 8);

        // Check if we have a custom title for this market
        const marketTitleConfig = getMarketTitle(marketId);

        // Use configured title if available, otherwise use metadata or generic title
        let question: string;
        let description: string;
        let categoryValue: string;

        if (marketTitleConfig) {
          // Use the configured market title
          question = marketTitleConfig.question;
          description = marketTitleConfig.description || `Market: ${marketIdShort}`;
          categoryValue = marketTitleConfig.category || 'Crypto';
        } else if (metadata && metadata.name) {
          // Use the token metadata name as the market title
          question = metadata.name.trim();

          // Create description from metadata
          description = `Market: ${marketIdShort}`;
          if (hasYesToken && hasNoToken) {
            description += ` • YES: ${metadata.symbol || yesTokenStr.slice(0, 4)}`;
          }

          categoryValue = 'Crypto';
        } else {
          // Generate generic title for markets without configuration or metadata
          if (hasYesToken && hasNoToken) {
            question = `Binary Prediction Market ${marketIdShort}`;
          } else {
            question = `Prediction Market ${marketIdShort} (Tokens not initialized)`;
          }

          // Create detailed description with token info
          description = `Market: ${marketIdShort}...${marketId.slice(-4)}`;
          if (hasYesToken && hasNoToken) {
            description += ` • YES: ${yesTokenStr.slice(0, 4)}...${yesTokenStr.slice(-4)}`;
            description += ` • NO: ${noTokenStr.slice(0, 4)}...${noTokenStr.slice(-4)}`;
          }
          description += ` • Supply: ${(data.tokenYesTotalSupply.toNumber() / 1e9).toFixed(0)}B`;

          categoryValue = 'Crypto';
        }

        // Calculate end date from ending slot
        let endDate: Date | null = null;
        if (data.endingSlot && !data.endingSlot.isZero()) {
          // The endingSlot is an absolute slot number
          // Calculate remaining slots from current slot
          const slotsRemaining = data.endingSlot.toNumber() - currentSlot;
          const secondsRemaining = slotsRemaining / 2.5; // ~2.5 slots per second
          // Only set endDate if it's in the future
          if (secondsRemaining > 0) {
            endDate = new Date(Date.now() + secondsRemaining * 1000);
          }
        }

        const parsedMarket: Market = {
          id: marketId,
          address: marketPubkey,
          question,
          description,
          category: categoryValue,
          createdAt,
          endDate,
          volume,
          liquidity,
          yesPrice,
          noPrice,
          yesTokenMint: data.yesTokenMint,
          noTokenMint: data.noTokenMint,
          creator: data.creator,
          status: data.isCompleted ? 'closed' : 'active',
          tags: [],
          contractData: data,
        };

        if (mounted) {
          setMarket(parsedMarket);
          setLoading(false);
        }
      } catch (err) {
        console.error('Error loading market:', err);
        if (mounted) {
          setError(err as Error);
          setLoading(false);
        }
      }
    }

    loadMarket();

    return () => {
      mounted = false;
    };
  }, [marketId, program]);

  return { market, loading, error };
}
