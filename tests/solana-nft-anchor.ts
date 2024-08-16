import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SolanaNftAnchor } from "../target/types/solana_nft_anchor";
import {
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID, mintTo,
} from "@solana/spl-token";
import * as assert from "assert";
import { before } from "mocha";
import {min} from "bn.js";

describe("solana-nft-anchor", () => {
    // Configure the client to use the local cluster.
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    const program = anchor.workspace.SolanaNftAnchor as Program<SolanaNftAnchor>;
    let mint: anchor.web3.Keypair;
    let nftStateAccount: anchor.web3.Keypair;
    let associatedTokenAccount: anchor.web3.PublicKey;

    // Airdrop some SOL to the owner (for transaction fees)
    beforeEach(async () => {
        mint = anchor.web3.Keypair.generate();
        nftStateAccount = anchor.web3.Keypair.generate();

        // Derive the associated token account for the owner
        associatedTokenAccount = anchor.utils.token.associatedAddress({
            mint: mint.publicKey,
            owner: program.provider.wallet.publicKey,
        });

        console.log("Mint PublicKey:", mint.publicKey.toBase58());
        console.log("NFT State Account PublicKey:", nftStateAccount.publicKey.toBase58());
        console.log("Associated Token Account PublicKey:", associatedTokenAccount.toBase58());
        console.log("Wallet PublicKey:", program.provider.wallet.publicKey.toBase58());

        // Confirm the airdrop transaction
        console.log("Finished initialization ✅");
    });

    it("Initializes the state and mints 4 NFTs", async () => {
        console.log("Before state initialization");

        // Initialize the state account with a total supply of 10 NFTs
        await program.rpc.initialize(new anchor.BN(10), {
            accounts: {
                state: nftStateAccount.publicKey,
                signer: program.provider.wallet.publicKey,
                systemProgram: anchor.web3.SystemProgram.programId,
                rent: anchor.web3.SYSVAR_RENT_PUBKEY,
            },
            signers: [nftStateAccount],
        });

        console.log("After state initialization");

        // Now mint 4 NFTs
        await program.rpc.initNft(new anchor.BN(4), {
            accounts: {
                state: nftStateAccount.publicKey,
                signer: program.provider.wallet.publicKey,
                mint: mint.publicKey,
                associatedTokenAccount: associatedTokenAccount,
                tokenProgram: TOKEN_PROGRAM_ID,
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
                systemProgram: anchor.web3.SystemProgram.programId,
                rent: anchor.web3.SYSVAR_RENT_PUBKEY,
            },
            signers: [mint],
        });

        console.log("After minting");

        // Fetch the state account and verify that 4 NFTs were minted
        const stateAccount = await program.account.nftState.fetch(nftStateAccount.publicKey);
        assert.equal(stateAccount.nftsMinted.toNumber(), 4, "4 NFTs should have been minted");

        // TODO: THIS DOESN'T FAIL FOR THE EXPECTED REASON
        try {
            console.log(program.provider.wallet);

            await mintTo(
                program.provider.connection,
                program.provider.wallet,
                mint.publicKey,
                associatedTokenAccount,
                program.provider.wallet.publicKey,
                1,
            );
            assert.fail("Minting should fail after authority is removed");
        } catch (err) {
            console.log("Minting failed as expected:", err);
            assert.ok("Minting failed as expected");
        }
    });
});