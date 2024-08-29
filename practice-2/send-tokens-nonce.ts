import "dotenv/config";
import {
    Connection,
    Keypair,
    PublicKey,
    clusterApiUrl,
    Transaction,
    sendAndConfirmTransaction,
    LAMPORTS_PER_SOL, SystemProgram, NONCE_ACCOUNT_LENGTH, NonceAccount
} from "@solana/web3.js";
import {
    createTransferCheckedInstruction,
    getOrCreateAssociatedTokenAccount,
    mintTo,
} from "@solana/spl-token";
import {airdropIfRequired, getExplorerLink} from "@solana-developers/helpers";
import bs58 from 'bs58';


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

// await airdropIfRequired(
//     connection,
//     bob.publicKey,
//     LAMPORTS_PER_SOL,
//     0.5 * LAMPORTS_PER_SOL
// );

const bobTokenAccount = await getOrCreateAssociatedTokenAccount(connection, bob, mint, bob.publicKey);
// ===========================================================================


// NONCE
// ===========================================================================
let nonce = Keypair.generate();
console.log(`nonce account: ${nonce.publicKey.toBase58()}`);

let initNonceTx = new Transaction().add(
    // create a nonce account
    SystemProgram.createAccount({
        fromPubkey: bob.publicKey,
        newAccountPubkey: nonce.publicKey,
        lamports:
            await connection.getMinimumBalanceForRentExemption(
                NONCE_ACCOUNT_LENGTH,
            ),
        space: NONCE_ACCOUNT_LENGTH,
        programId: SystemProgram.programId,
    }),
    // init nonce account
    SystemProgram.nonceInitialize({
        noncePubkey: nonce.publicKey, // nonce account pubkey
        authorizedPubkey: bob.publicKey, // nonce account authority (for advance and close)
    }),
);

await sendAndConfirmTransaction(connection, initNonceTx, [bob, nonce]);


let nonceAccountInfo = await connection.getAccountInfo(nonce.publicKey);
// @ts-ignore
let nonceAccount = NonceAccount.fromAccountData(nonceAccountInfo.data);
console.log(`nonce: ${nonceAccount.nonce}`);
console.log(`authority: ${nonceAccount.authorizedPubkey.toBase58()}`);
console.log(`fee calculator: ${JSON.stringify(nonceAccount.feeCalculator)}`);

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
    SystemProgram.nonceAdvance({
        noncePubkey: nonce.publicKey,
        authorizedPubkey: bob.publicKey,
    }),
    createTransferCheckedInstruction(
        aliceTokenAccount.address, // source
        mint, // mint
        bobTokenAccount.address, // destination
        alice.publicKey, // owner of source account
        5 * MINOR_UNITS_PER_MAJOR_UNITS, // amount to transfer
        2 // decimals of token
    )
);


transaction.recentBlockhash = nonceAccount.nonce;
transaction.feePayer = bob.publicKey;
transaction.partialSign(bob);
transaction.partialSign(alice);
// transaction.partialSign(nonce);


setTimeout(async function () {
    const serializedTransaction = transaction.serialize({requireAllSignatures: false});

    const signature = await connection.sendRawTransaction(serializedTransaction)

    console.log("View the transaction at:", getExplorerLink("transaction", signature, "devnet"));
}, 1000 * 60 * 2);

// ===========================================================================
