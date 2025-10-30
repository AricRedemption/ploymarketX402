// 工具函数
import { Connection, PublicKey, Transaction, Keypair } from "@solana/web3.js";
import { BN, Program } from "@coral-xyz/anchor";
import { PredictionMarket } from "../target/types/prediction_market";

export async function execTx(
  tx: Transaction,
  connection: Connection,
  payer: any
): Promise<string> {
  const signature = await connection.sendTransaction(tx, [payer], {
    skipPreflight: true,
    commitment: "confirmed",
  });
  
  await connection.confirmTransaction(signature, "confirmed");
  return signature;
}
