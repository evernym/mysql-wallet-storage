package com.evernym.aurora_integration_tests.tests;

import com.evernym.aurora_integration_tests.main.non_secrets.WalletRecord;
import com.evernym.aurora_integration_tests.main.non_secrets.WalletSearch;
import org.hyperledger.indy.sdk.IOException;
import org.hyperledger.indy.sdk.IndyException;
import org.hyperledger.indy.sdk.wallet.Wallet;
import org.hyperledger.indy.sdk.wallet.WalletValueNotFoundException;
import org.json.JSONArray;
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
    String value2 = "RecordValue2";
    String tagsEmpty = "{}";
    String tags =  "{\"tagName1\":\"str1\",\"tagName2\":\"5\",\"tagName3\":\"12\"}";
    String tagsWithDeletedTag1 =  "{\"tagName2\":\"5\",\"tagName3\":\"12\"}";
    String queryEmpty = "{}";
    String optionsEmpty = "{}";
    String optionsTags = "{\"retrieveTags\": true}";

    @BeforeMethod(alwaysRun = true)
    public void createAndOpenWallet() throws IndyException, ExecutionException, InterruptedException {

        // create and open wallet
        Wallet.createWallet(POOL, walletName, TYPE, CONFIG, CREDENTIALS).get();
        wallet = Wallet.openWallet(walletName, null, CREDENTIALS).get();
    }

    @Test
    public void integrationTestUsingNonSecretsApi() throws Exception {

        /**
         * add record
         */
        WalletRecord.add(wallet, type, id, value, tagsEmpty).get();

        /**
         * get record with options: retriveTags
         */
        String recordJson = WalletRecord.get(wallet, type, id, optionsTags).get();

        JSONObject actual = new JSONObject(recordJson);

        JSONObject expected = new JSONObject()
                .put("id", id)
                .putOpt("type", JSONObject.NULL)
                .put("value", value)
                .put("tags", "{}");

        Assert.assertTrue(expected.similar(actual), "expected '" + expected.toString() + "' matches actual '" + actual.toString() + "'");

        /**
         * update record value
         */
        WalletRecord.updateValue(wallet, type, id, value2).get();

        expected = new JSONObject()
                .put("id", id)
                .putOpt("type", JSONObject.NULL)
                .put("value", value2)
                .put("tags", "{}");

        // get record with options: retriveTags
        recordJson = WalletRecord.get(wallet, type, id, optionsTags).get();
        actual = new JSONObject(recordJson);

        Assert.assertTrue(expected.similar(actual), "expected '" + expected.toString() + "' matches actual '" + actual.toString() + "'");


        /**
         * update record tags
         */
        WalletRecord.updateTags(wallet, type, id, tags).get();
        expected = new JSONObject()
                .put("id", id)
                .putOpt("type", JSONObject.NULL)
                .put("value", value2)
                .put("tags", tags);

        // get record with options: retriveTags
        recordJson = WalletRecord.get(wallet, type, id, optionsTags).get();
        actual = new JSONObject(recordJson);

        Assert.assertEquals(expected.getString("value"), actual.getString("value"), "Value is as expected");

        JSONObject expectedTagsJson = new JSONObject(expected.getString("tags"));
        JSONObject actualTagsJson = new JSONObject(actual.getString("tags"));
        Assert.assertTrue(expectedTagsJson.similar(actualTagsJson),
                "expected tags '" + expectedTagsJson.toString() + "' matches actual tags'" + actualTagsJson.toString() + "'");


        /**
         * Search records
         */
        WalletSearch search = WalletSearch.open(wallet, type, queryEmpty, optionsTags).get();

        String searchRecordsJson = search.fetchNextRecords(wallet, 1).get();

        JSONObject searchRecords = new JSONObject(searchRecordsJson);

        JSONArray records = searchRecords.getJSONArray("records");

        Assert.assertEquals(1, records.length());

        actual = new JSONObject(records.get(0));
        Assert.assertEquals(expected.getString("value"), actual.getString("value"), "Value is as expected");

        actualTagsJson = new JSONObject(actual.getString("tags"));
        Assert.assertTrue(expectedTagsJson.similar(actualTagsJson),
                "expected tags '" + expectedTagsJson.toString() + "' matches actual tags'" + actualTagsJson.toString() + "'");

        Assert.assertTrue(expected.similar(records.get(0)));

        search.close();


        /**
         * Delete record tags
         */
        WalletRecord.deleteTags(wallet, type, id, "[\"tagName1\"]").get();

        expected = new JSONObject()
                .put("id", id)
                .putOpt("type", JSONObject.NULL)
                .put("value", value2)
                .put("tags", tagsWithDeletedTag1);

        // get record with options: retriveTags
        recordJson = WalletRecord.get(wallet, type, id, optionsTags).get();
        actual = new JSONObject(recordJson);

        Assert.assertEquals(expected.getString("value"), actual.getString("value"), "Value is as expected");

        expectedTagsJson = new JSONObject(expected.getString("tags"));
        actualTagsJson = new JSONObject(actual.getString("tags"));
        Assert.assertTrue(expectedTagsJson.similar(actualTagsJson),
                "expected tags '" + expectedTagsJson.toString() + "' matches actual tags'" + actualTagsJson.toString() + "'");


        /**
         * Delete record
         */
        WalletRecord.delete(wallet, type, id).get();

        try {
            WalletRecord.get(wallet, type, id, optionsTags).get();
            Assert.assertTrue(false); // this line should not be reached, previous line should throw an exception
        } catch (Exception e) {
            Assert.assertTrue(e instanceof ExecutionException, "Expected Exception is of ExecutionException type. Actaul type is: " + e.getClass());
            Assert.assertTrue(e.getCause() instanceof WalletValueNotFoundException, "Expected Cause is WalletValueNotFoundException, actual is " + e.getCause().toString());
        }


        /**
         * get record without options set (known problem due to default JSON tag names)
         */
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
