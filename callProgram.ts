import {
  Connection,
  Keypair,
  Transaction,
  sendAndConfirmTransaction,
  TransactionInstruction,
  PublicKey,
  Finality,
} from "@solana/web3.js";
import fs from "fs";

const connection = new Connection("https://api.devnet.solana.com", "confirmed");

const payer = Keypair.fromSecretKey(
  Uint8Array.from(JSON.parse(fs.readFileSync("/home/x0rc1ph3r/.config/solana/id.json", "utf8")))
);

const PROGRAM_ID = new PublicKey("6ujAE4E7VL17fLsUj87kFqvzhC5gKTzwA3TSvxKhdMcv");

const tests = [
  { name: "http://localhost:3000", hex: "026c6f63616c686f73743a33303030" },
  { name: "http://subdomain.localhost:3000", hex: "02737562646f6d61696e2e6c6f63616c686f73743a33303030" },
  { name: "https://localhost.net", hex: "016c6f63616c686f737405" },
  { name: "https://google.com", hex: "01676f6f676c6503" },
  { name: "https://a.a", hex: "01612e61" },
  { name: "https://a.com", hex: "016103" },
  { name: "https://git@github.com:username/repo.git", hex: "0167697440676974687562033a757365726e616d652f7265706f07" },
  {
    name: "https://a-really-long-url-that-probably-would-be-so-hard-to-actually-use-but-whatever.com",
    hex: "01612d7265616c6c792d6c6f6e672d75726c2d746861742d70726f6261626c792d776f756c642d62652d736f2d686172642d746f2d61637475616c6c792d7573652d6275742d776861746576657203",
  },
  { name: "https://🦝👀🍹🌏.net", hex: "01f09fa69df09f9180f09f8db9f09f8c8f05" },
  {
    name: "https://something.yourcooldomain.com?query_param=123&val=true",
    hex: "01736f6d657468696e672e796f7572636f6f6c646f6d61696e033f71756572795f706172616d3d3132332676616c3d74727565",
  },
];

const results: { Name: string; "CU Used": number; "Compression Ratio": string; Explorer: string }[] = [];

(async () => {
  console.log("Starting compute unit tests...");
  for (const { name, hex } of tests) {
    const instructionData = Buffer.from(hex, "hex");

    const instruction = new TransactionInstruction({
      programId: PROGRAM_ID,
      keys: [],
      data: instructionData,
    });

    const tx = new Transaction().add(instruction);

    try {
      const signature = await sendAndConfirmTransaction(connection, tx, [payer]);

      const parsed = await connection.getTransaction(signature, {
        commitment: "confirmed",
      });

      const logLine = parsed?.meta?.logMessages?.find((line) =>
        line.includes("Program 6ujAE4E7VL17fLsUj87kFqvzhC5gKTzwA3TSvxKhdMcv consumed")
      );

      console.log(logLine)

      const cuUsed = logLine ? parseInt(logLine.match(/consumed (\d+)/)?.[1] || "0") : 0;

      results.push({
        Name: name,
        "CU Used": cuUsed,
        "Compression Ratio": (name.length/ (hex.length / 2)).toFixed(2),
        Explorer: `https://explorer.solana.com/tx/${signature}?cluster=devnet`,
      });
    } catch (err: any) {
      results.push({
        Name: name,
        "CU Used": -1,
        "Compression Ratio": "Error",
        Explorer: `Failed: ${err.message}`,
      });
    }
  }

  console.log("\nCompute Unit Summary:\n");
  console.table(results);
})();
