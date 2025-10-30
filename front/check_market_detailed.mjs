import { Connection, PublicKey } from '@solana/web3.js';
import { AnchorProvider, Program } from '@coral-xyz/anchor';
import { getAssociatedTokenAddress } from '@solana/spl-token';
import idl from './contract_source_code/programs/prediction-market/prediction_market.json' assert { type: 'json' };

const connection = new Connection('https://api.devnet.solana.com', 'confirmed');
const PROGRAM_ID = new PublicKey('EgEc7fuse6eQ3UwqeWGFncDtbTwozWCy4piydbeRaNrU');

const marketAddress = process.argv[2] || 'FBCXxXw4QBca76vKB3vSZBTEUdqt3ju3sNCPKaLtvH6Q';

async function checkMarket() {
  try {
    const provider = new AnchorProvider(
      connection,
      { publicKey: PublicKey.default },
      { commitment: 'confirmed' }
    );

    const program = new Program(idl, provider);
    const marketPubkey = new PublicKey(marketAddress);

    console.log('=== MARKET ACCOUNT ===');
    const marketData = await program.account.market.fetch(marketPubkey);
    console.log('Market:', marketPubkey.toString());
    console.log('');

    console.log('Token Mints:');
    console.log('  YES:', marketData.yesTokenMint.toString());
    console.log('  NO:', marketData.noTokenMint.toString());
    console.log('');

    console.log('Reserves (in lamports):');
    console.log('  Real YES SOL Reserves:', marketData.realYesSolReserves.toString());
    console.log('  Real NO SOL Reserves:', marketData.realNoSolReserves.toString());
    console.log('  Total:', (marketData.realYesSolReserves.toNumber() + marketData.realNoSolReserves.toNumber()) / 1e9, 'SOL');
    console.log('');

    console.log('Virtual Reserves (in lamports):');
    console.log('  Virtual YES SOL Reserves:', marketData.virtualYesSolReserves.toString());
    console.log('  Virtual NO SOL Reserves:', marketData.virtualNoSolReserves.toString());
    console.log('');

    console.log('Token Supply:');
    console.log('  YES Total Supply:', marketData.tokenYesTotalSupply.toString(), `(${marketData.tokenYesTotalSupply.toNumber() / 1e9} tokens)`);
    console.log('  NO Total Supply:', marketData.tokenNoTotalSupply.toString(), `(${marketData.tokenNoTotalSupply.toNumber() / 1e9} tokens)`);
    console.log('');

    console.log('Status:', marketData.status);
    console.log('Creator:', marketData.creator.toString());
    console.log('');

    // Check global vault token balances
    const [globalVault] = PublicKey.findProgramAddressSync(
      [Buffer.from('global-vault')],
      PROGRAM_ID
    );

    console.log('=== GLOBAL VAULT ===');
    console.log('Global Vault:', globalVault.toString());

    const yesAta = await getAssociatedTokenAddress(
      marketData.yesTokenMint,
      globalVault,
      true
    );
    const noAta = await getAssociatedTokenAddress(
      marketData.noTokenMint,
      globalVault,
      true
    );

    console.log('YES Token ATA:', yesAta.toString());
    console.log('NO Token ATA:', noAta.toString());

    // Get actual token account balances
    const yesAccountInfo = await connection.getTokenAccountBalance(yesAta);
    const noAccountInfo = await connection.getTokenAccountBalance(noAta);

    console.log('');
    console.log('Actual Token Balances in Vault:');
    console.log('  YES tokens:', yesAccountInfo.value.uiAmountString);
    console.log('  NO tokens:', noAccountInfo.value.uiAmountString);

  } catch (error) {
    console.error('Error:', error.message);
  }
}

checkMarket();
