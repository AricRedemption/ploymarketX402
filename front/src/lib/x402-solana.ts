/**
 * X402 Solana Integration
 * Creates payment transactions for Solana using @solana/wallet-adapter
 */

import { Connection, PublicKey, Transaction, SystemProgram } from '@solana/web3.js';
import {
  getAssociatedTokenAddress,
  createTransferCheckedInstruction,
  createAssociatedTokenAccountInstruction,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID
} from '@solana/spl-token';
import { PaymentRequirements } from 'x402/types';
import { WalletContextState } from '@solana/wallet-adapter-react';

/**
 * Creates an unsigned SPL token transfer transaction for X402 payment
 * This transaction transfers USDC from the user to the payTo address
 */
export async function createSolanaPaymentTransaction(
  wallet: WalletContextState,
  paymentRequirements: PaymentRequirements,
  connection: Connection
): Promise<Transaction> {
  if (!wallet.publicKey) {
    throw new Error('Wallet not connected');
  }

  // Parse payment requirements
  const usdcMint = new PublicKey(paymentRequirements.asset);
  const recipient = new PublicKey(paymentRequirements.payTo);
  const amount = BigInt(paymentRequirements.maxAmountRequired);

  // Get associated token accounts
  const senderATA = await getAssociatedTokenAddress(
    usdcMint,
    wallet.publicKey,
    false,
    TOKEN_PROGRAM_ID
  );

  const recipientATA = await getAssociatedTokenAddress(
    usdcMint,
    recipient,
    false,
    TOKEN_PROGRAM_ID
  );

  // Check if sender's ATA exists and has sufficient balance
  const senderAccountInfo = await connection.getAccountInfo(senderATA);
  if (!senderAccountInfo) {
    // Get all token accounts for this wallet to help debug
    const tokenAccounts = await connection.getParsedTokenAccountsByOwner(
      wallet.publicKey,
      { programId: TOKEN_PROGRAM_ID }
    );

    console.error('Expected USDC mint:', usdcMint.toString());
    console.error('Expected USDC ATA:', senderATA.toString());
    console.error('Your token accounts:', tokenAccounts.value.map(acc => ({
      mint: acc.account.data.parsed.info.mint,
      balance: acc.account.data.parsed.info.tokenAmount.uiAmount,
    })));

    throw new Error(
      `You do not have a USDC token account for mint ${usdcMint.toString()}. ` +
      `Found ${tokenAccounts.value.length} token accounts. ` +
      `Please check the console for details.`
    );
  }

  // Check if recipient's ATA exists
  const recipientAccountInfo = await connection.getAccountInfo(recipientATA);

  // Create transaction
  const transaction = new Transaction();

  // If recipient's ATA doesn't exist, create it first
  if (!recipientAccountInfo) {
    console.log('Recipient USDC account does not exist, creating it...');
    const createATAInstruction = createAssociatedTokenAccountInstruction(
      wallet.publicKey,  // payer
      recipientATA,      // ata
      recipient,         // owner
      usdcMint,          // mint
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );
    transaction.add(createATAInstruction);
  }

  // Add transfer instruction
  const transferInstruction = createTransferCheckedInstruction(
    senderATA,      // source
    usdcMint,       // mint
    recipientATA,   // destination
    wallet.publicKey, // owner
    amount,         // amount
    6,              // decimals (USDC has 6 decimals)
    [],             // multiSigners
    TOKEN_PROGRAM_ID
  );

  transaction.add(transferInstruction);

  // Get recent blockhash
  const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash();
  transaction.recentBlockhash = blockhash;
  transaction.feePayer = wallet.publicKey;
  transaction.lastValidBlockHeight = lastValidBlockHeight;

  return transaction;
}

/**
 * Signs and serializes a Solana transaction for X402
 * Returns the base64-encoded signed transaction
 */
export async function signAndSerializeTransaction(
  wallet: WalletContextState,
  transaction: Transaction
): Promise<string> {
  if (!wallet.signTransaction) {
    throw new Error('Wallet does not support transaction signing');
  }

  // Sign the transaction with the wallet
  const signedTransaction = await wallet.signTransaction(transaction);

  // Serialize to base64
  const serialized = signedTransaction.serialize();
  return Buffer.from(serialized).toString('base64');
}

/**
 * Creates a complete X402 payment payload for Solana
 */
export async function createSolanaPaymentPayload(
  wallet: WalletContextState,
  paymentRequirements: PaymentRequirements,
  connection: Connection
): Promise<{
  x402Version: number;
  scheme: 'exact';
  network: string;
  payload: {
    transaction: string;
  };
}> {
  // Create the transaction
  const transaction = await createSolanaPaymentTransaction(
    wallet,
    paymentRequirements,
    connection
  );

  // Sign and serialize
  const signedTransactionBase64 = await signAndSerializeTransaction(wallet, transaction);

  // Return X402 payment payload
  return {
    x402Version: 1,
    scheme: 'exact',
    network: paymentRequirements.network as any,
    payload: {
      transaction: signedTransactionBase64,
    },
  };
}
