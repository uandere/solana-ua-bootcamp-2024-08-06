import "dotenv/config";
import { Keypair } from "@solana/web3.js";
import bs58 from "bs58";  // Import the entire module

let privateKey = process.env["SECRET_KEY"];
if (privateKey === undefined) {
  console.log("Add SECRET_KEY to .env!");
  process.exit(1);
}

const asArray = Uint8Array.from(JSON.parse(privateKey));
const keypair = Keypair.fromSecretKey(asArray);

const base58PrivateKey = bs58.encode(keypair.secretKey);

console.log(`Public key: ${keypair.publicKey.toBase58()}`);
console.log(`Private key: ${base58PrivateKey}`);