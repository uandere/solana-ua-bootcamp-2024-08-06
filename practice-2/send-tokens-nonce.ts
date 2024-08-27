import "dotenv/config";
import {
    Connection,
    Keypair,
    PublicKey,
    clusterApiUrl,
    Transaction,
    sendAndConfirmTransaction,
    SystemProgram,
    NONCE_ACCOUNT_LENGTH,
} from "@solana/web3.js";
import {
    createMintToInstruction,
    createTransferInstruction,
    getOrCreateAssociatedTokenAccount,
    mintTo,
} from "@solana/spl-token";
import { airdropIfRequired, getExplorerLink } from "@solana-developers/helpers";

// CONSTANTS
// ===========================================================================
const MINOR_UNITS_PER_MAJOR_UNITS = Math.pow(10, 2);

const connection = new Connection(clusterApiUrl("devnet"));

const mint = new PublicKey("4XwxQaU3qYHCFnkSxEmax1XKPsSEg4t3eouJp4Cm7PTW");
// ===========================================================================

// BOB (ONLINE) - Nonce Account Setup
// ===========================================================================
let bobPrivateKey = process.env["BOB_SECRET_KEY"];

if (bobPrivateKey === undefined) {
    console.log("Add BOB_SECRET_KEY to .env!");
    process.exit(1);
}
const bobPrivateKeyAsArray = Uint8Array.from(JSON.parse(bobPrivateKey));
const bob = Keypair.fromSecretKey(bobPrivateKeyAsArray);

console.log("Bob's public key: ", bob.publicKey);

// Create a new nonce account for Bob
const nonceAccount = Keypair.generate();
const minimumBalanceForNonce = await connection.getMinimumBalanceForRentExemption(
    NONCE_ACCOUNT_LENGTH
);

const createNonceAccountIx = SystemProgram.createAccount({
    fromPubkey: bob.publicKey,
    newAccountPubkey: nonceAccount.publicKey,
    lamports: minimumBalanceForNonce,
    space: NONCE_ACCOUNT_LENGTH,
    programId: SystemProgram.programId,
});

const initNonceAccountIx = SystemProgram.nonceInitialize({
    noncePubkey: nonceAccount.publicKey,
    authorizedPubkey: bob.publicKey, // Bob will be the authority for this nonce account
});

const nonceTransaction = new Transaction().add(
    createNonceAccountIx,
    initNonceAccountIx
);

await sendAndConfirmTransaction(connection, nonceTransaction, [bob, nonceAccount]);

// ALICE (OFFLINE)
// ===========================================================================
let alicePrivateKey = process.env["SECRET_KEY"];
if (alicePrivateKey === undefined) {
    console.log("Add SECRET_KEY to .env!");
    process.exit(1);
}
const alicePrivateKeyAsArray = Uint8Array.from(JSON.parse(alicePrivateKey));
const alice = Keypair.fromSecretKey(alicePrivateKeyAsArray);

console.log("Alice's public key: ", alice.publicKey);

const aliceTokenAccount = await getOrCreateAssociatedTokenAccount(
    connection,
    alice,
    mint,
    alice.publicKey
);

// Minting tokens to Alice's associated token account (Alice does this offline)
await mintTo(
    connection,
    alice,
    mint,
    aliceTokenAccount.address,
    alice,
    10 * MINOR_UNITS_PER_MAJOR_UNITS
);

// Alice creates and signs a transaction (offline) using the nonce account
const aliceTransaction = new Transaction({
    feePayer: bob.publicKey,
    recentBlockhash: nonceAccount.publicKey.toString(), // Use the nonce account as the blockhash
});

aliceTransaction.add(
    createTransferInstruction(
        aliceTokenAccount.address,
        bob.publicKey,  // Replace with Bob's public key
        alice.publicKey,
        5 * MINOR_UNITS_PER_MAJOR_UNITS
    )
);

aliceTransaction.partialSign(alice);

// Save or send the signed transaction to Bob
const signedTransactionData = aliceTransaction.serialize({
    requireAllSignatures: false
});

// BOB (ONLINE)
// ===========================================================================
const bobTokenAccount = await getOrCreateAssociatedTokenAccount(
    connection,
    bob,
    mint,
    bob.publicKey
);

// Bob deserializes Alice's signed transaction
const bobTransaction = Transaction.from(signedTransactionData);

// Get the nonce from the account
const nonceInfo = await connection.getNonce(nonceAccount.publicKey);
// @ts-ignore
bobTransaction.recentBlockhash = nonceInfo.nonce;

// Advance the nonce
const nonceAdvanceIx = SystemProgram.nonceAdvance({
    noncePubkey: nonceAccount.publicKey,
    authorizedPubkey: bob.publicKey,
});

bobTransaction.add(nonceAdvanceIx);

// Bob signs the transaction as the fee payer
bobTransaction.partialSign(bob);

const signature = await sendAndConfirmTransaction(
    connection,
    bobTransaction,
    [bob]
);

console.log("View the transaction at:", getExplorerLink("transaction", signature, "devnet"));
