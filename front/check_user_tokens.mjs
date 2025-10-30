import { Connection, PublicKey } from '@solana/web3.js';
import { getAssociatedTokenAddress, getAccount } from '@solana/spl-token';

const connection = new Connection('https://api.devnet.solana.com', 'confirmed');

// Get from user - replace with your values
const USER_WALLET = process.argv[2];
const YES_TOKEN = process.argv[3];
const NO_TOKEN = process.argv[4];

if (!USER_WALLET || !YES_TOKEN || !NO_TOKEN) {
  console.log('Usage: node check_user_tokens.mjs <USER_WALLET> <YES_TOKEN> <NO_TOKEN>');
  console.log('Example: node check_user_tokens.mjs CRD9Pe7ou8yidVnCn3a1rUejSczCA1xsSwajfqS5Xfwb 25C8k3... AypNh...');
  process.exit(1);
}

async function checkBalances() {
  try {
    const userPubkey = new PublicKey(USER_WALLET);
    const yesTokenMint = new PublicKey(YES_TOKEN);
    const noTokenMint = new PublicKey(NO_TOKEN);

    console.log('Checking token balances for:');
    console.log('User:', userPubkey.toString());
    console.log('YES Token:', yesTokenMint.toString());
    console.log('NO Token:', noTokenMint.toString());
    console.log('');

    // Check YES token
    try {
      const yesAta = await getAssociatedTokenAddress(yesTokenMint, userPubkey);
      console.log('YES Token ATA:', yesAta.toString());
      
      const yesAccount = await getAccount(connection, yesAta);
      const yesBalance = Number(yesAccount.amount) / 1e9;
      console.log('✅ YES Token Balance:', yesBalance);
    } catch (err) {
      console.log('❌ YES Token Account not found or error:', err.message);
    }

    console.log('');

    // Check NO token
    try {
      const noAta = await getAssociatedTokenAddress(noTokenMint, userPubkey);
      console.log('NO Token ATA:', noAta.toString());
      
      const noAccount = await getAccount(connection, noAta);
      const noBalance = Number(noAccount.amount) / 1e9;
      console.log('✅ NO Token Balance:', noBalance);
    } catch (err) {
      console.log('❌ NO Token Account not found or error:', err.message);
    }

  } catch (error) {
    console.error('Error:', error.message);
  }
}

checkBalances();
