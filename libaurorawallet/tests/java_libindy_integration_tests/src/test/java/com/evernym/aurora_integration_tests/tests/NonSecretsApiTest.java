package com.evernym.aurora_integration_tests.tests;

import com.evernym.aurora_integration_tests.main.non_secrets.WalletRecord;
import org.hyperledger.indy.sdk.IndyException;
import org.hyperledger.indy.sdk.wallet.Wallet;
import org.json.JSONObject;
import org.testng.Assert;
import org.testng.annotations.AfterMethod;
import org.testng.annotations.BeforeMethod;
import org.testng.annotations.Test;

import java.util.concurrent.ExecutionException;

public class NonSecretsApiTest extends BaseTest {

    private String walletName = "testWallet" + System.currentTimeMillis();
    private Wallet wallet = null;

    String type = "TestType";
    String id = "RecordId";
    String value = "RecordValue";
    String tagsEmpty = "{}";
    String optionsEmpty = "{}";
    String optionsTags = "{\"retrieveTags\": true}";

    @BeforeMethod(alwaysRun = true)
    public void createAndOpenWallet() throws IndyException, ExecutionException, InterruptedException {

        // create and open wallet
        Wallet.createWallet(POOL, walletName, TYPE, CONFIG, CREDENTIALS).get();
        wallet = Wallet.openWallet(walletName, null, CREDENTIALS).get();
    }

    @Test
    public void integrationTestUsingNonSecretsApi() throws IndyException, ExecutionException, InterruptedException {

        // add record
        WalletRecord.add(wallet, type, id, value, tagsEmpty).get();

        // get record with options: retriveTags
        String recordJson = WalletRecord.get(wallet, type, id, optionsTags).get();

        JSONObject actual = new JSONObject(recordJson);

        JSONObject expected = new JSONObject()
                .put("id", id)
                .putOpt("type", JSONObject.NULL)
                .put("value", value)
                .put("tags", "{}");

        Assert.assertTrue(expected.similar(actual), "expected '" + expected.toString() + "' matches actual '" + actual.toString() + "'");

        // get record without options set (known problem due to default JSON tag names)
        recordJson = WalletRecord.get(wallet, type, id, optionsEmpty).get();

        actual = new JSONObject(recordJson);

        expected = new JSONObject()
                .put("id", id)
                .putOpt("type", JSONObject.NULL)
                .put("value", value)
                .put("tags", JSONObject.NULL);

        Assert.assertTrue(expected.similar(actual), "expected '" + expected.toString() + "' matches actual '" + actual.toString() + "'");

    }

    @AfterMethod (alwaysRun = true)
    public void afterMethod() throws IndyException, InterruptedException, ExecutionException {

        Wallet.deleteWallet(walletName, CREDENTIALS).get();
    }
}
