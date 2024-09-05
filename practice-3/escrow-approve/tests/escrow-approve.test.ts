import {beforeAll, describe, expect, test} from "@jest/globals";
import * as anchor from "@coral-xyz/anchor";
import {BN, Program} from "@coral-xyz/anchor";
import {EscrowApprove} from "../target/types/escrow_approve";
import {
    Connection,
    Keypair,
    LAMPORTS_PER_SOL,
    PublicKey,
    SystemProgram,
    Transaction,
    TransactionInstruction,
} from "@solana/web3.js";
import {
    createAssociatedTokenAccountIdempotentInstruction,
    createInitializeMint2Instruction,
    createMintToInstruction,
    getAssociatedTokenAddressSync,
    getMinimumBalanceForRentExemptMint,
    MINT_SIZE, TOKEN_2022_PROGRAM_ID,
    TOKEN_PROGRAM_ID
} from "@solana/spl-token";
import {randomBytes} from "crypto";

import {confirmTransaction, makeKeypairs} from "@solana-developers/helpers";

const TOKEN_PROGRAM: typeof TOKEN_2022_PROGRAM_ID | typeof TOKEN_PROGRAM_ID =
    TOKEN_2022_PROGRAM_ID;

export const getRandomBigNumber = (size: number = 8) => {
    return new BN(randomBytes(size));
};

function areBnEqual(a: unknown, b: unknown): boolean | undefined {
    const isABn = a instanceof BN;
    const isBBn = b instanceof BN;

    if (isABn && isBBn) {
        return a.eq(b);
    } else if (isABn === isBBn) {
        return undefined;
    } else {
        return false;
    }
}

expect.addEqualityTesters([areBnEqual]);

const createTokenAndMintTo = async (connection: Connection, payer: PublicKey, tokenMint: PublicKey, decimals: number, mintAuthority: PublicKey, mintTo: Array<{
    recepient: PublicKey; amount: number
}>): Promise<Array<TransactionInstruction>> => {
    let minimumLamports = await getMinimumBalanceForRentExemptMint(connection);

    let createTokeIxs = [SystemProgram.createAccount({
        fromPubkey: payer,
        newAccountPubkey: tokenMint,
        lamports: minimumLamports,
        space: MINT_SIZE,
        programId: TOKEN_PROGRAM,
    }), createInitializeMint2Instruction(tokenMint, decimals, mintAuthority, null, TOKEN_PROGRAM),];

    let mintToIxs = mintTo.flatMap(({recepient, amount}) => {
        const ataAddress = getAssociatedTokenAddressSync(tokenMint, recepient, false, TOKEN_PROGRAM);

        return [createAssociatedTokenAccountIdempotentInstruction(payer, ataAddress, recepient, tokenMint, TOKEN_PROGRAM), createMintToInstruction(tokenMint, ataAddress, mintAuthority, amount, [], TOKEN_PROGRAM),];
    });

    return [...createTokeIxs, ...mintToIxs];
};

const getTokenBalanceOn = (connection: Connection,) => async (tokenAccountAddress: PublicKey,): Promise<BN> => {
    const tokenBalance = await connection.getTokenAccountBalance(tokenAccountAddress);
    return new BN(tokenBalance.value.amount);
};

describe("escrow-approve", () => {
    anchor.setProvider(anchor.AnchorProvider.env());

    const provider = anchor.getProvider();

    const connection = provider.connection;

    const program = anchor.workspace.EscrowApprove as Program<EscrowApprove>;

    const [maker, taker, tokenMintA, tokenMintB] = makeKeypairs(4);

    const [makerTokenAccountA, makerTokenAccountB, takerTokenAccountA, takerTokenAccountB] = [maker, taker,].flatMap((owner) => [tokenMintA, tokenMintB].map((tokenMint) => getAssociatedTokenAddressSync(tokenMint.publicKey, owner.publicKey, false, TOKEN_PROGRAM)));

    const offerId = getRandomBigNumber();

    beforeAll(async () => {
        const giveMakerAndTakerSolIxs: Array<TransactionInstruction> = [maker, taker,].map((owner) => SystemProgram.transfer({
            fromPubkey: provider.publicKey, toPubkey: owner.publicKey, lamports: 10 * LAMPORTS_PER_SOL,
        }));

        const tokenASetupIxs = await createTokenAndMintTo(connection, provider.publicKey, tokenMintA.publicKey, 6, maker.publicKey, [{
            recepient: maker.publicKey, amount: 100_000_000
        }, {recepient: taker.publicKey, amount: 20_000_000},]);

        const tokenBSetupIxs = await createTokenAndMintTo(connection, provider.publicKey, tokenMintB.publicKey, 6, taker.publicKey, [{
            recepient: maker.publicKey, amount: 5_000_000
        }, {recepient: taker.publicKey, amount: 300_000_000},]);

        let tx = new Transaction();
        tx.instructions = [...giveMakerAndTakerSolIxs, ...tokenASetupIxs, ...tokenBSetupIxs,];

        const _setupTxSig = await provider.sendAndConfirm(tx, [maker, taker, tokenMintA, tokenMintB,]);
    });

    const makeOfferTx = async (maker: Keypair, offerId: BN, offeredTokenMint: PublicKey, offeredAmount: BN, wantedTokenMint: PublicKey, wantedAmount: BN): Promise<{
        offerAddress: PublicKey;
    }> => {
        const transactionSignature = await program.methods
            .makeOffer(offerId, offeredAmount, wantedAmount)
            .accounts({
                maker: maker.publicKey,
                tokenMintA: offeredTokenMint,
                tokenMintB: wantedTokenMint,
                tokenProgram: TOKEN_PROGRAM, // Ensure token program is passed
            })
            .signers([maker])
            .rpc();

        await confirmTransaction(connection, transactionSignature);

        const [offerAddress, _offerBump] = PublicKey.findProgramAddressSync([Buffer.from("offer"), maker.publicKey.toBuffer(), offerId.toArrayLike(Buffer, "le", 8),], program.programId);

        return {offerAddress};
    };

    const takeOfferTx = async (offerAddress: PublicKey, taker: Keypair,): Promise<void> => {
        const transactionSignature = await program.methods
            .takeOffer()
            .accounts({
                taker: taker.publicKey, offer: offerAddress, tokenProgram: TOKEN_PROGRAM,
            })
            .signers([taker])
            .rpc();

        await confirmTransaction(connection, transactionSignature);
    };

    test("Offer created by Maker", async () => {
        const offeredA = new BN(10_000_000); // Maker offers 10,000,000 token A
        const wantedB = new BN(100_000_000); // Maker wants 100,000,000 token B

        const getTokenBalance = getTokenBalanceOn(connection);

        // Maker creates an offer
        const {offerAddress} = await makeOfferTx(maker, offerId, tokenMintA.publicKey, // Offering token A
            offeredA, tokenMintB.publicKey, // Wanting token B
            wantedB);

        // Ensure the offer account has correct data
        const offerAccount = await program.account.offer.fetch(offerAddress);
        expect(offerAccount.maker).toEqual(maker.publicKey);
        expect(offerAccount.tokenMintA).toEqual(tokenMintA.publicKey);
        expect(offerAccount.tokenMintB).toEqual(tokenMintB.publicKey);
        expect(offerAccount.tokenBWantedAmount).toEqual(wantedB);
    });

    test("Offer taken by Taker, tokens balances are updated", async () => {
        const getTokenBalance = getTokenBalanceOn(connection);

        // Retrieve the existing offer
        const [offerAddress, _offerBump] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("offer"),
                maker.publicKey.toBuffer(),
                offerId.toArrayLike(Buffer, "le", 8),
            ],
            program.programId
        );


        // Verify balances before the offer is taken
        expect(await getTokenBalance(makerTokenAccountB)).toEqual(new BN(5_000_000));  // Maker has 5,000,000 token B
        expect(await getTokenBalance(takerTokenAccountA)).toEqual(new BN(20_000_000)); // Taker has 20,000,000 token A
        expect(await getTokenBalance(takerTokenAccountB)).toEqual(new BN(300_000_000)); // Taker has 300,000,000 token B

        // Taker accepts the offer
        await takeOfferTx(offerAddress, taker);

        // After the offer is taken:
        // Maker should have 105,000,000 of token B (received 100,000,000 from taker)
        // Taker should have 30,000,000 of token A (received 10,000,000 from maker)
        expect(await getTokenBalance(makerTokenAccountA)).toEqual(new BN(90_000_000)); // Maker's token A stays at 90,000,000
        expect(await getTokenBalance(makerTokenAccountB)).toEqual(new BN(105_000_000)); // Maker received 100,000,000 token B from taker

        expect(await getTokenBalance(takerTokenAccountA)).toEqual(new BN(30_000_000)); // Taker received 10,000,000 token A from maker
        expect(await getTokenBalance(takerTokenAccountB)).toEqual(new BN(200_000_000)); // Taker gave 100,000,000 token B to maker
    });
});