import "dotenv/config";
import {clusterApiUrl, Connection, Keypair} from "@solana/web3.js";
import {createMint, createMultisig, getMint, getOrCreateAssociatedTokenAccount, mintTo} from "@solana/spl-token";

let privateKey = process.env["SECRET_KEY"];
if (privateKey === undefined) {
    console.log("Add SECRET_KEY to .env!");
    process.exit(1);
}
const asArray = Uint8Array.from(JSON.parse(privateKey));
const payer = Keypair.fromSecretKey(asArray);


const signer1 = Keypair.generate();
const signer2 = Keypair.generate();
const signer3 = Keypair.generate();

console.log("Signer 1 key: ", signer1.publicKey.toBase58());
console.log("Signer 2 key: ", signer2.publicKey.toBase58());
console.log("Signer 3 key: ", signer3.publicKey.toBase58());

const connection = new Connection(clusterApiUrl("devnet"));

const multisigKey = await createMultisig(
    connection,
    payer,
    [
        signer1.publicKey,
        signer2.publicKey,
        signer3.publicKey
    ],
    2
);

console.log(`Created 2/3 multisig ${multisigKey.toBase58()}`);


const mint = await createMint(
    connection,
    payer,
    multisigKey,
    multisigKey,
    9
);

const associatedTokenAccount = await getOrCreateAssociatedTokenAccount(
    connection,
    payer,
    mint,
    signer1.publicKey
);

try {
    await mintTo(
        connection,
        payer,
        mint,
        associatedTokenAccount.address,
        multisigKey,
        1
    )
} catch (error) {
    console.log("Error as expected: ", error);
}

await mintTo(
    connection,
    payer,
    mint,
    associatedTokenAccount.address,
    multisigKey,
    1,
    [
        signer1,
        signer2
    ]
)

const mintInfo = await getMint(
    connection,
    mint
)

console.log(`Minted ${mintInfo.supply} token`);
// Minted 1 token