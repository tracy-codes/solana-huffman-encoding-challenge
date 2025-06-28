import {
  Connection,
  Keypair,
  Transaction,
  sendAndConfirmTransaction,
  TransactionInstruction,
  PublicKey,
} from "@solana/web3.js";
import fs from "fs";

const connection = new Connection('https://api.devnet.solana.com', 'confirmed');
const payer = Keypair.fromSecretKey(
  Uint8Array.from(JSON.parse(fs.readFileSync("/home/x0rc1ph3r/.config/solana/id.json", "utf8")))
);

const PROGRAM_ID = new PublicKey('6ujAE4E7VL17fLsUj87kFqvzhC5gKTzwA3TSvxKhdMcv'); // 📛 Replace

const hexPayload = "01f09fa69df09f9180f09f8db9f09f8c8f05";

const instructionData = Buffer.from(hexPayload, "hex");

const instruction = new TransactionInstruction({
  programId: PROGRAM_ID,
  keys: [],
  data: instructionData,
});

(async () => {
  const tx = new Transaction().add(instruction);
  const signature = await sendAndConfirmTransaction(connection, tx, [payer]);
  console.log("✅ Transaction sent:", `https://explorer.solana.com/tx/${signature}?cluster=devnet`);
})();
