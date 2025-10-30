import { Connection } from '@solana/web3.js';

const connection = new Connection('https://api.devnet.solana.com', 'confirmed');

const signature = process.argv[2] || 'gW7HjUq7mUqPuA4C3ETvRY48MfFq3aqUTHPHgjdDoQjmhmLm97F2EzY9NBauVC5kgpaSpTo4iqeLUdmFQbxNJyz';

async function checkTransaction() {
  console.log('Checking transaction:', signature);
  console.log('');

  try {
    const tx = await connection.getTransaction(signature, {
      maxSupportedTransactionVersion: 0,
      commitment: 'confirmed',
    });

    if (!tx) {
      console.log('❌ Transaction not found');
      return;
    }

    console.log('Transaction Status:', tx.meta?.err ? '❌ FAILED' : '✅ SUCCESS');
    console.log('');

    if (tx.meta?.err) {
      console.log('Error:', JSON.stringify(tx.meta.err, null, 2));
      console.log('');
    }

    console.log('=== TOKEN BALANCES CHANGES ===');
    if (tx.meta?.preTokenBalances && tx.meta?.postTokenBalances) {
      console.log('\nPre-balances:');
      tx.meta.preTokenBalances.forEach((balance, i) => {
        console.log(`  [${i}] Account: ${balance.owner} | Mint: ${balance.mint} | Amount: ${balance.uiTokenAmount.uiAmountString}`);
      });

      console.log('\nPost-balances:');
      tx.meta.postTokenBalances.forEach((balance, i) => {
        console.log(`  [${i}] Account: ${balance.owner} | Mint: ${balance.mint} | Amount: ${balance.uiTokenAmount.uiAmountString}`);
      });
    }

    console.log('\n=== SOL BALANCE CHANGES ===');
    if (tx.meta?.preBalances && tx.meta?.postBalances && tx.transaction.message.accountKeys) {
      tx.transaction.message.accountKeys.forEach((key, i) => {
        const preBalance = tx.meta.preBalances[i] / 1e9;
        const postBalance = tx.meta.postBalances[i] / 1e9;
        const diff = postBalance - preBalance;
        if (diff !== 0) {
          console.log(`  ${key.toString()}: ${preBalance.toFixed(4)} → ${postBalance.toFixed(4)} (${diff > 0 ? '+' : ''}${diff.toFixed(4)} SOL)`);
        }
      });
    }

    console.log('\n=== PROGRAM LOGS ===');
    if (tx.meta?.logMessages) {
      tx.meta.logMessages.forEach(log => console.log(`  ${log}`));
    }

  } catch (error) {
    console.error('Error fetching transaction:', error.message);
  }
}

checkTransaction();
