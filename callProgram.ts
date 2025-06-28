import {
  Connection,
  Keypair,
  Transaction,
  sendAndConfirmTransaction,
  TransactionInstruction,
  PublicKey,
} from "@solana/web3.js";
import fs from "fs";

const connection = new Connection("https://api.devnet.solana.com", "confirmed");

const payer = Keypair.fromSecretKey(
  Uint8Array.from(JSON.parse(fs.readFileSync("/home/x0rc1ph3r/.config/solana/id.json", "utf8")))
);

const PROGRAM_ID = new PublicKey("ADUtWaDe3cn7V3oskWD7UWkdq9zxc6DcZKHoUH8vWBcD");

const tests = [
  { name: "http://localhost:3000", data: [36, 0, 63, 0, 48, 48, 49, 48, 48, 48, 49, 115, 49, 58, 48, 49, 116, 49, 99, 48, 48, 49, 111, 48, 49, 104, 49, 195, 177, 48, 49, 108, 48, 49, 97, 48, 49, 51, 49, 52, 166, 101, 209, 251, 84, 70, 95, 0] },
  { name: "http://subdomain.localhost:3000", data: [57, 0, 117, 0, 48, 48, 48, 48, 49, 58, 49, 105, 48, 49, 98, 49, 110, 48, 49, 111, 49, 48, 48, 48, 48, 49, 108, 48, 49, 100, 49, 116, 48, 48, 49, 46, 49, 51, 49, 115, 48, 48, 48, 49, 195, 177, 49, 117, 48, 49, 109, 49, 52, 48, 48, 49, 104, 49, 99, 49, 97, 228, 230, 23, 146, 146, 215, 137, 210, 23, 126, 56, 174, 97, 86, 216] },
  { name: "https://localhost.net", data: [34, 0, 49, 0, 48, 48, 48, 49, 50, 49, 97, 48, 49, 115, 49, 111, 48, 48, 49, 108, 48, 49, 116, 49, 99, 48, 48, 49, 195, 178, 49, 195, 177, 48, 49, 51, 49, 104, 250, 165, 29, 153, 237, 85, 0] },
  { name: "https://google.com", data: [25, 0, 19, 0, 48, 48, 48, 49, 108, 49, 195, 178, 49, 103, 48, 48, 49, 49, 49, 101, 48, 49, 111, 48, 49, 195, 177, 49, 50, 118, 69, 192] },
  { name: "https://a.a", data: [12, 0, 6, 0, 48, 48, 49, 46, 49, 50, 48, 49, 195, 177, 49, 97, 204] },
  { name: "https://a.com", data: [16, 0, 3, 0, 48, 48, 49, 195, 178, 49, 195, 177, 48, 49, 50, 48, 49, 49, 49, 97, 224] },
  { name: "https://git@github.com:username/repo.git", data: [70, 0, 176, 0, 48, 48, 48, 48, 49, 112, 49, 103, 48, 48, 49, 99, 49, 98, 49, 105, 48, 48, 48, 49, 97, 49, 110, 49, 117, 48, 48, 49, 58, 49, 195, 177, 49, 116, 48, 48, 48, 48, 49, 46, 49, 50, 49, 109, 48, 49, 114, 48, 49, 115, 49, 53, 48, 48, 48, 49, 195, 178, 49, 64, 48, 49, 104, 49, 47, 48, 49, 111, 49, 101, 211, 184, 89, 155, 216, 155, 228, 77, 244, 165, 129, 58, 88, 182, 250, 74, 39, 247, 94, 29, 1, 55] },
  {
    name: "https://a-really-long-url-that-probably-would-be-so-hard-to-actually-use-but-whatever.com",
    data: [70, 0, 83, 1, 48, 48, 48, 49, 108, 48, 48, 48, 49, 112, 49, 99, 48, 49, 110, 49, 103, 48, 49, 119, 48, 49, 49, 49, 50, 48, 49, 97, 48, 49, 98, 49, 114, 48, 48, 48, 49, 111, 49, 101, 48, 49, 117, 48, 49, 121, 48, 49, 118, 49, 100, 48, 48, 49, 116, 48, 49, 104, 48, 49, 115, 48, 49, 195, 177, 49, 195, 178, 49, 45, 214, 97, 27, 46, 242, 128, 183, 16, 81, 126, 156, 124, 210, 206, 67, 195, 38, 22, 230, 138, 23, 246, 159, 180, 125, 39, 191, 228, 116, 78, 82, 2, 222, 182, 158, 213, 156, 218, 89, 55, 75, 147, 0],
  },
  { name: "https://🦝👀🍹🌏.net", data: [37, 0, 12, 0, 48, 48, 48, 49, 50, 49, 195, 177, 48, 49, 240, 159, 166, 157, 49, 51, 48, 48, 49, 240, 159, 145, 128, 49, 240, 159, 141, 185, 48, 49, 240, 159, 140, 143, 49, 195, 178, 82, 224] },
  {
    name: "https://something.yourcooldomain.com?query_param=123&val=true",
    data: [84, 0, 13, 1, 48, 48, 48, 48, 48, 49, 112, 49, 113, 49, 99, 49, 111, 48, 48, 49, 117, 49, 101, 48, 48, 49, 95, 49, 105, 49, 114, 48, 48, 48, 48, 49, 116, 48, 49, 118, 49, 51, 48, 49, 110, 48, 49, 100, 49, 115, 48, 49, 97, 49, 109, 48, 48, 48, 49, 46, 48, 49, 195, 177, 49, 103, 48, 49, 61, 48, 49, 63, 49, 104, 48, 48, 48, 49, 38, 49, 49, 49, 121, 48, 49, 50, 49, 108, 222, 16, 4, 243, 155, 88, 110, 217, 103, 142, 148, 113, 39, 243, 27, 166, 203, 2, 111, 96, 162, 191, 88, 10, 122, 189, 115, 232, 248, 138, 191, 168, 58, 40],
  },
];

const results: { Name: string; "CU Used": number; "Compression Ratio": string; Explorer: string }[] = [];

(async () => {
  console.log("Starting compute unit tests...");
  for (const { name, data } of tests) {
    const instructionData = Buffer.from(data);

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
        line.includes("Program ADUtWaDe3cn7V3oskWD7UWkdq9zxc6DcZKHoUH8vWBcD consumed")
      );

      console.log(logLine)

      const cuUsed = logLine ? parseInt(logLine.match(/consumed (\d+)/)?.[1] || "0") : 0;

      results.push({
        Name: name,
        "CU Used": cuUsed,
        "Compression Ratio": (name.length / data.length).toFixed(2),
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
