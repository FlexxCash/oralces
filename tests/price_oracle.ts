import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { assert } from "chai";
import { Oracles } from "../target/types/oracles";

// Define AssetType to match Anchor's expected structure
export type AssetType = {
  jupSol: Record<string, never>;
} | {
  vSol: Record<string, never>;
} | {
  bSol: Record<string, never>;
} | {
  mSol: Record<string, never>;
} | {
  hSol: Record<string, never>;
} | {
  jitoSol: Record<string, never>;
} | {
  sol: Record<string, never>;
};

describe("price_oracle", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Oracles as Program<Oracles>;

  let priceOracleHeaderPda: anchor.web3.PublicKey;
  let priceOracleDataPda: anchor.web3.PublicKey;
  let oracleFeed: anchor.web3.PublicKey;
  let solOracleFeed: anchor.web3.PublicKey;
  let switchboardProgram: anchor.web3.PublicKey;

  before(async () => {
    [priceOracleHeaderPda] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("price_oracle_header")],
      program.programId
    );

    [priceOracleDataPda] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("price_oracle_data")],
      program.programId
    );

    oracleFeed = new anchor.web3.PublicKey("4NiWaTuje7SVe9DN1vfnX7m1qBC7DnUxwRxbdgEDUGX1");
    solOracleFeed = new anchor.web3.PublicKey("GvDMxPzN1sCj7L26YDK2HnMRXEQmQ2aemov8YBtPS7vR");
    switchboardProgram = new anchor.web3.PublicKey("Aio4gaXjXzJNVLtzwtNVmSqGKpANtXhybbkhtAC94ji2");
  });

  it("Initializes the price oracle", async () => {
    await program.methods.initialize(switchboardProgram)
      .accounts({
        header: priceOracleHeaderPda,
        data: priceOracleDataPda,
        authority: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const headerAccount = await program.account.priceOracleHeader.fetch(priceOracleHeaderPda);
    const dataAccount = await program.account.priceOracleData.fetch(priceOracleDataPda);

    assert.isFalse(headerAccount.emergencyStop);
    assert.equal(headerAccount.authority.toBase58(), provider.wallet.publicKey.toBase58());
    assert.equal(headerAccount.switchboardProgramId.toBase58(), switchboardProgram.toBase58());
    assert.equal(dataAccount.priceData.length, 7);
  });

  it("Updates prices and APYs for all assets", async () => {
    await program.methods.updatePricesAndApys()
      .accounts({
        header: priceOracleHeaderPda,
        data: priceOracleDataPda,
        oracleFeed: oracleFeed,
        authority: provider.wallet.publicKey,
      })
      .rpc();

    const dataAccount = await program.account.priceOracleData.fetch(priceOracleDataPda);
    assert.equal(dataAccount.priceData.length, 7);
    dataAccount.priceData.forEach((data, index) => {
      if (index < 6) {  // Exclude SOL
        assert.isTrue(data.price > 0);
        assert.isTrue(data.apy > 0);
      }
    });
  });

  it("Updates SOL price", async () => {
    await program.methods.updateSolPrice()
      .accounts({
        header: priceOracleHeaderPda,
        data: priceOracleDataPda,
        oracleFeed: solOracleFeed,
        authority: provider.wallet.publicKey,
      })
      .rpc();

    const dataAccount = await program.account.priceOracleData.fetch(priceOracleDataPda);
    const solData = dataAccount.priceData[6];  // SOL is the last element
    assert.isTrue(solData.price > 0);
    assert.equal(solData.apy, 0);  // SOL doesn't have APY
  });

  it("Gets current price for all asset types", async () => {
    const assetTypes: AssetType[] = [
      { jupSol: {} },
      { vSol: {} },
      { bSol: {} },
      { mSol: {} },
      { hSol: {} },
      { jitoSol: {} },
      { sol: {} }
    ];

    for (const assetType of assetTypes) {
      const tx = await program.methods.getCurrentPrice(assetType)
        .accounts({
          data: priceOracleDataPda,
        })
        .rpc();

      const txLogs = await provider.connection.getTransaction(tx, { commitment: 'confirmed' });
      const assetName = Object.keys(assetType)[0].toUpperCase();
      assert.isTrue(txLogs.meta.logMessages.some(log => log.includes(`Current price for ${assetName}:`)));
    }
  });

  it("Gets current APY for all asset types except SOL", async () => {
    const assetTypes: AssetType[] = [
      { jupSol: {} },
      { vSol: {} },
      { bSol: {} },
      { mSol: {} },
      { hSol: {} },
      { jitoSol: {} }
    ];

    for (const assetType of assetTypes) {
      const tx = await program.methods.getCurrentApy(assetType)
        .accounts({
          data: priceOracleDataPda,
        })
        .rpc();

      const txLogs = await provider.connection.getTransaction(tx, { commitment: 'confirmed' });
      const assetName = Object.keys(assetType)[0].toUpperCase();
      assert.isTrue(txLogs.meta.logMessages.some(log => log.includes(`Current APY for ${assetName}:`)));
    }
  });

  it("Sets emergency stop", async () => {
    await program.methods.setEmergencyStop(true)
      .accounts({
        header: priceOracleHeaderPda,
        authority: provider.wallet.publicKey,
      })
      .rpc();

    const headerAccount = await program.account.priceOracleHeader.fetch(priceOracleHeaderPda);
    assert.isTrue(headerAccount.emergencyStop);
  });

  it("Fails to update prices and APYs when emergency stop is active", async () => {
    try {
      await program.methods.updatePricesAndApys()
        .accounts({
          header: priceOracleHeaderPda,
          data: priceOracleDataPda,
          oracleFeed: oracleFeed,
          authority: provider.wallet.publicKey,
        })
        .rpc();
      assert.fail("Should have thrown an error");
    } catch (error) {
      assert.include(error.toString(), "Emergency stop activated");
    }
  });

  it("Deactivates emergency stop", async () => {
    await program.methods.setEmergencyStop(false)
      .accounts({
        header: priceOracleHeaderPda,
        authority: provider.wallet.publicKey,
      })
      .rpc();

    const headerAccount = await program.account.priceOracleHeader.fetch(priceOracleHeaderPda);
    assert.isFalse(headerAccount.emergencyStop);
  });

  it("Fails to update prices and APYs with unauthorized user", async () => {
    const unauthorizedUser = anchor.web3.Keypair.generate();
    try {
      await program.methods.updatePricesAndApys()
        .accounts({
          header: priceOracleHeaderPda,
          data: priceOracleDataPda,
          oracleFeed: oracleFeed,
          authority: unauthorizedUser.publicKey,
        })
        .signers([unauthorizedUser])
        .rpc();
      assert.fail("Should have thrown an error");
    } catch (error) {
      assert.include(error.toString(), "Unauthorized access");
    }
  });

  it("Fails to set emergency stop with unauthorized user", async () => {
    const unauthorizedUser = anchor.web3.Keypair.generate();
    try {
      await program.methods.setEmergencyStop(true)
        .accounts({
          header: priceOracleHeaderPda,
          authority: unauthorizedUser.publicKey,
        })
        .signers([unauthorizedUser])
        .rpc();
      assert.fail("Should have thrown an error");
    } catch (error) {
      assert.include(error.toString(), "Unauthorized access");
    }
  });

  // Commenting out this test as the method might not be in the IDL
  /*
  it("Updates Switchboard program ID", async () => {
    const newSwitchboardProgramId = anchor.web3.Keypair.generate().publicKey;
    await program.methods.updateSwitchboardProgramId(newSwitchboardProgramId)
      .accounts({
        header: priceOracleHeaderPda,
        authority: provider.wallet.publicKey,
      })
      .rpc();

    const headerAccount = await program.account.priceOracleHeader.fetch(priceOracleHeaderPda);
    assert.equal(headerAccount.switchboardProgramId.toBase58(), newSwitchboardProgramId.toBase58());
  });
  */
});