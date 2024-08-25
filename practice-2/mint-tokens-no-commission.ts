import "dotenv/config";
import {
    Connection,
    Keypair,
    PublicKey,
    clusterApiUrl,
    Transaction,
    sendAndConfirmTransaction,
    LAMPORTS_PER_SOL
} from "@solana/web3.js";
import {
    createMintToInstruction, createTransferInstruction, getOrCreateAssociatedTokenAccount, mintTo,
} from "@solana/spl-token";
import {airdropIfRequired, getExplorerLink} from "@solana-developers/helpers";


// CONSTANTS
// ===========================================================================
const MINOR_UNITS_PER_MAJOR_UNITS = Math.pow(10, 2);

const connection = new Connection(clusterApiUrl("devnet"));

const mint = new PublicKey(
    "4XwxQaU3qYHCFnkSxEmax1XKPsSEg4t3eouJp4Cm7PTW"
);
// ===========================================================================


// ALICE
// ===========================================================================
let alicePrivateKey = process.env["SECRET_KEY"];
if (alicePrivateKey === undefined) {
    console.log("Add SECRET_KEY to .env!");
    process.exit(1);
}
const alicePrivateKeyAsArray = Uint8Array.from(JSON.parse(alicePrivateKey));

const alice = Keypair.fromSecretKey(alicePrivateKeyAsArray);

console.log("Alice's public key: ", alice.publicKey)

// ATTENTION: UNCOMMENT THIS FOR THE FIRST TIME WHEN ALICE HAS NO MONEY

// await airdropIfRequired(
//     connection,
//     alice.publicKey,
//     LAMPORTS_PER_SOL,
//     0.5 * LAMPORTS_PER_SOL
// );

const aliceTokenAccount = await getOrCreateAssociatedTokenAccount(connection, alice, mint, alice.publicKey);
// ===========================================================================


// BOB
// ===========================================================================
let bobPrivateKey = process.env["BOB_SECRET_KEY"];

if (bobPrivateKey === undefined) {
    console.log("Add BOB_SECRET_KEY to .env!");
    process.exit(1);
}

const bobPrivateKeyAsArray = Uint8Array.from(JSON.parse(bobPrivateKey));

const bob = Keypair.fromSecretKey(bobPrivateKeyAsArray);

console.log("Bob's public key: ", bob.publicKey)

// ATTENTION: UNCOMMENT THIS FOR THE FIRST TIME WHEN BOB HAS NO MONEY

await airdropIfRequired(
    connection,
    bob.publicKey,
    LAMPORTS_PER_SOL,
    0.5 * LAMPORTS_PER_SOL
);

const bobTokenAccount = await getOrCreateAssociatedTokenAccount(connection, bob, mint, bob.publicKey);
// ===========================================================================


// LOGIC
// ===========================================================================

// Minting tokens to Alice's associated token account
await mintTo(
    connection,
    alice,
    mint,
    aliceTokenAccount.address,
    alice,
    10 * MINOR_UNITS_PER_MAJOR_UNITS
);

// Alice creates a transaction, signs it by herself and transfers to Bob to sign it too.
const transaction = new Transaction();

transaction.add(
    createTransferInstruction(
        aliceTokenAccount.address,
        bobTokenAccount.address,
        alice.publicKey,
        5 * MINOR_UNITS_PER_MAJOR_UNITS
    )
)

transaction.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;

transaction.feePayer = bob.publicKey;

// Signing on Alice's behalf (mint authority)
transaction.partialSign(alice);


// ...Sending it to Bob...


// Signing on Bob's behalf (fee payer)
transaction.partialSign(bob);

const signature = await sendAndConfirmTransaction(connection, transaction, [alice, bob]);

console.log("View the transaction at:", getExplorerLink("transaction", signature, "devnet"));

// ===========================================================================
