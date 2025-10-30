// 交易创建函数
import { Connection, PublicKey, Transaction, Keypair } from "@solana/web3.js";
import { BN, Program } from "@coral-xyz/anchor";
import { PredictionMarket } from "../target/types/prediction_market";
import { 
  TEST_DECIMALS,
  TEST_INITIAL_VIRTUAL_TOKEN_RESERVES,
  TEST_INITIAL_REAL_TOKEN_RESERVES,
  SEED_CONFIG,
  SEED_MARKET,
  SEED_USERINFO,
  SEED_GLOBAL,
  SEED_METADATA
} from "./constant";

// 创建配置交易
export async function createConfigTx(
  payer: PublicKey,
  config: any,
  connection: Connection,
  program: Program<PredictionMarket>
): Promise<Transaction> {
  const [configPda] = PublicKey.findProgramAddressSync(
    [Buffer.from(SEED_CONFIG)],
    program.programId
  );
  
  const [globalVaultPda] = PublicKey.findProgramAddressSync(
    [Buffer.from(SEED_GLOBAL)],
    program.programId
  );

  const tx = await program.methods
    .configure(config)
    .accounts({
      payer: payer,
      config: configPda,
      globalVault: globalVaultPda,
      systemProgram: PublicKey.default,
      tokenProgram: PublicKey.default,
      associatedTokenProgram: PublicKey.default,
    })
    .transaction();

  return tx;
}

// 创建市场交易
export async function createMarketTx(
  yesSymbol: string,
  yesUri: string,
  creator: PublicKey,
  teamWallet: PublicKey,
  noToken: PublicKey,
  connection: Connection,
  program: Program<PredictionMarket>
): Promise<{ tx: Transaction; yes_tokenKp: Keypair }> {
  const yesTokenKp = Keypair.generate();
  
  const [configPda] = PublicKey.findProgramAddressSync(
    [Buffer.from(SEED_CONFIG)],
    program.programId
  );
  
  const [globalVaultPda] = PublicKey.findProgramAddressSync(
    [Buffer.from(SEED_GLOBAL)],
    program.programId
  );

  const [marketPda] = PublicKey.findProgramAddressSync(
    [Buffer.from(SEED_MARKET), yesTokenKp.publicKey.toBytes(), noToken.toBytes()],
    program.programId
  );

  const tx = await program.methods
    .createMarket({
      yesSymbol,
      yesUri,
      startSlot: null,
      endingSlot: null
    })
    .accounts({
      globalConfig: configPda,
      globalVault: globalVaultPda,
      creator: creator,
      yesToken: yesTokenKp.publicKey,
      noToken: noToken,
      market: marketPda,
      yesTokenMetadataAccount: PublicKey.default,
      noTokenMetadataAccount: PublicKey.default,
      globalYesTokenAccount: PublicKey.default,
      teamWallet: teamWallet,
      systemProgram: PublicKey.default,
      rent: PublicKey.default,
      tokenProgram: PublicKey.default,
      associatedTokenProgram: PublicKey.default,
      mplTokenMetadataProgram: PublicKey.default,
    })
    .transaction();

  return { tx, yes_tokenKp };
}

// 铸造NO代币交易
export async function mintNoTokenTx(
  noSymbol: string,
  noUri: string,
  creator: PublicKey,
  connection: Connection,
  program: Program<PredictionMarket>
): Promise<{ tx: Transaction; no_tokenKp: Keypair }> {
  const noTokenKp = Keypair.generate();
  
  const [configPda] = PublicKey.findProgramAddressSync(
    [Buffer.from(SEED_CONFIG)],
    program.programId
  );
  
  const [globalVaultPda] = PublicKey.findProgramAddressSync(
    [Buffer.from(SEED_GLOBAL)],
    program.programId
  );

  const tx = await program.methods
    .mintNoToken(noSymbol, noUri)
    .accounts({
      globalConfig: configPda,
      globalVault: globalVaultPda,
      creator: creator,
      noToken: noTokenKp.publicKey,
      noTokenMetadataAccount: PublicKey.default,
      globalNoTokenAccount: PublicKey.default,
      systemProgram: PublicKey.default,
      rent: PublicKey.default,
      tokenProgram: PublicKey.default,
      associatedTokenProgram: PublicKey.default,
      mplTokenMetadataProgram: PublicKey.default,
    })
    .transaction();

  return { tx, no_tokenKp };
}

// 交易代币
export async function swapTx(
  user: PublicKey,
  yesToken: PublicKey,
  noToken: PublicKey,
  amount: number,
  style: number,
  tokenType: number,
  connection: Connection,
  program: Program<PredictionMarket>
): Promise<Transaction> {
  const [configPda] = PublicKey.findProgramAddressSync(
    [Buffer.from(SEED_CONFIG)],
    program.programId
  );
  
  const [globalVaultPda] = PublicKey.findProgramAddressSync(
    [Buffer.from(SEED_GLOBAL)],
    program.programId
  );

  const [marketPda] = PublicKey.findProgramAddressSync(
    [Buffer.from(SEED_MARKET), yesToken.toBytes(), noToken.toBytes()],
    program.programId
  );

  const tx = await program.methods
    .swap(new BN(amount), style, tokenType, new BN(0))
    .accounts({
      globalConfig: configPda,
      teamWallet: PublicKey.default,
      market: marketPda,
      globalVault: globalVaultPda,
      yesToken: yesToken,
      noToken: noToken,
      globalYesAta: PublicKey.default,
      globalNoAta: PublicKey.default,
      userYesAta: PublicKey.default,
      userNoAta: PublicKey.default,
      userInfo: PublicKey.default,
      user: user,
      systemProgram: PublicKey.default,
      tokenProgram: PublicKey.default,
      associatedTokenProgram: PublicKey.default,
    })
    .transaction();

  return tx;
}

// 市场结算
export async function resolutionTx(
  user: PublicKey,
  authority: PublicKey,
  yesToken: PublicKey,
  noToken: PublicKey,
  connection: Connection,
  program: Program<PredictionMarket>
): Promise<Transaction> {
  const [configPda] = PublicKey.findProgramAddressSync(
    [Buffer.from(SEED_CONFIG)],
    program.programId
  );
  
  const [globalVaultPda] = PublicKey.findProgramAddressSync(
    [Buffer.from(SEED_GLOBAL)],
    program.programId
  );

  const [marketPda] = PublicKey.findProgramAddressSync(
    [Buffer.from(SEED_MARKET), yesToken.toBytes(), noToken.toBytes()],
    program.programId
  );

  const tx = await program.methods
    .resolution(new BN(0), new BN(0), 0, true)
    .accounts({
      globalConfig: configPda,
      market: marketPda,
      globalVault: globalVaultPda,
      yesToken: yesToken,
      noToken: noToken,
      userInfo: PublicKey.default,
      user: user,
      authority: authority,
      systemProgram: PublicKey.default,
      tokenProgram: PublicKey.default,
      associatedTokenProgram: PublicKey.default,
    })
    .transaction();

  return tx;
}

// 添加流动性
export async function addLiquidityTx(
  user: PublicKey,
  yesToken: PublicKey,
  noToken: PublicKey,
  amount: number,
  connection: Connection,
  program: Program<PredictionMarket>
): Promise<Transaction> {
  const [configPda] = PublicKey.findProgramAddressSync(
    [Buffer.from(SEED_CONFIG)],
    program.programId
  );
  
  const [globalVaultPda] = PublicKey.findProgramAddressSync(
    [Buffer.from(SEED_GLOBAL)],
    program.programId
  );

  const [marketPda] = PublicKey.findProgramAddressSync(
    [Buffer.from(SEED_MARKET), yesToken.toBytes(), noToken.toBytes()],
    program.programId
  );

  const tx = await program.methods
    .addLiquidity(new BN(amount))
    .accounts({
      globalConfig: configPda,
      teamWallet: PublicKey.default,
      market: marketPda,
      globalVault: globalVaultPda,
      yesToken: yesToken,
      noToken: noToken,
      userInfo: PublicKey.default,
      user: user,
      systemProgram: PublicKey.default,
      tokenProgram: PublicKey.default,
      associatedTokenProgram: PublicKey.default,
    })
    .transaction();

  return tx;
}

// 提取流动性
export async function withdrawLiquidityTx(
  user: PublicKey,
  yesToken: PublicKey,
  noToken: PublicKey,
  amount: number,
  connection: Connection,
  program: Program<PredictionMarket>
): Promise<Transaction> {
  const [configPda] = PublicKey.findProgramAddressSync(
    [Buffer.from(SEED_CONFIG)],
    program.programId
  );
  
  const [globalVaultPda] = PublicKey.findProgramAddressSync(
    [Buffer.from(SEED_GLOBAL)],
    program.programId
  );

  const [marketPda] = PublicKey.findProgramAddressSync(
    [Buffer.from(SEED_MARKET), yesToken.toBytes(), noToken.toBytes()],
    program.programId
  );

  const tx = await program.methods
    .withdrawLiquidity(new BN(amount))
    .accounts({
      globalConfig: configPda,
      teamWallet: PublicKey.default,
      market: marketPda,
      globalVault: globalVaultPda,
      yesToken: yesToken,
      noToken: noToken,
      userInfo: PublicKey.default,
      user: user,
      systemProgram: PublicKey.default,
      tokenProgram: PublicKey.default,
      associatedTokenProgram: PublicKey.default,
    })
    .transaction();

  return tx;
}
