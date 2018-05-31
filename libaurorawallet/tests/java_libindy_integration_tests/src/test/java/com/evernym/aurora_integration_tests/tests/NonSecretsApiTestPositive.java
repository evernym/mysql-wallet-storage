package com.evernym.aurora_integration_tests.tests;

import org.hyperledger.indy.sdk.IndyException;
import org.hyperledger.indy.sdk.non_secrets.WalletRecord;
import org.hyperledger.indy.sdk.non_secrets.WalletSearch;
import org.hyperledger.indy.sdk.wallet.Wallet;
import org.hyperledger.indy.sdk.wallet.WalletNotFoundException;
import org.json.JSONArray;
import org.json.JSONObject;
import org.testng.Assert;
import org.testng.annotations.AfterClass;
import org.testng.annotations.Test;

import java.util.concurrent.ExecutionException;

public class NonSecretsApiTestPositive extends BaseTest {

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

    @Test (priority = 0)
    public void createAndOpenWallet() throws Exception {

        Wallet.createWallet(POOL, walletName, TYPE, CONFIG, CREDENTIALS).get();
        wallet = Wallet.openWallet(walletName, null, CREDENTIALS).get();
    }

    @Test (dependsOnMethods = "createAndOpenWallet", priority = 1)
    public void addRecord() throws Exception {
        WalletRecord.add(wallet, type, id, value, tagsEmpty).get();
    }

    @Test (dependsOnMethods = "addRecord", priority = 2)
    public void getRecordWithTags() throws Exception {

        String recordJson = WalletRecord.get(wallet, type, id, optionsTags).get();

        JSONObject actual = new JSONObject(recordJson);

        JSONObject expected = new JSONObject()
                .put("id", id)
                .putOpt("type", JSONObject.NULL)
                .put("value", value)
                .put("tags", "{}");

        Assert.assertTrue(expected.similar(actual), "expected '" + expected.toString() + "' matches actual '" + actual.toString() + "'");
    }

    @Test (dependsOnMethods = "addRecord", priority = 2)
    public void getRecordWithoutTags() throws Exception {

        String recordJson = WalletRecord.get(wallet, type, id, optionsEmpty).get();

        JSONObject actual = new JSONObject(recordJson);

        JSONObject expected = new JSONObject()
                .put("id", id)
                .putOpt("type", JSONObject.NULL)
                .put("value", value)
                .put("tags", JSONObject.NULL);

        Assert.assertTrue(expected.similar(actual), "expected '" + expected.toString() + "' matches actual '" + actual.toString() + "'");
    }

    @Test (dependsOnMethods = "addRecord", priority = 3)
    public void updateRecordValue() throws Exception {
        WalletRecord.updateValue(wallet, type, id, value2).get();

        JSONObject expected = new JSONObject()
                .put("id", id)
                .putOpt("type", JSONObject.NULL)
                .put("value", value2)
                .put("tags", "{}");

        // get record with options: retriveTags
        String recordJson = WalletRecord.get(wallet, type, id, optionsTags).get();
        JSONObject actual = new JSONObject(recordJson);

        Assert.assertTrue(expected.similar(actual), "expected '" + expected.toString() + "' matches actual '" + actual.toString() + "'");
    }

    @Test (dependsOnMethods = "updateRecordValue", priority = 4)
    public void updateRecordTags() throws Exception {

        WalletRecord.updateTags(wallet, type, id, tags).get();
        JSONObject expected = new JSONObject()
                .put("id", id)
                .putOpt("type", JSONObject.NULL)
                .put("value", value2)
                .put("tags", tags);

        // get record with options: retriveTags
        String recordJson = WalletRecord.get(wallet, type, id, optionsTags).get();
        JSONObject actual = new JSONObject(recordJson);

        Assert.assertEquals(expected.getString("value"), actual.getString("value"), "Value is as expected");

        JSONObject expectedTagsJson = new JSONObject(expected.getString("tags"));
        JSONObject actualTagsJson = new JSONObject(actual.getString("tags"));
        Assert.assertTrue(expectedTagsJson.similar(actualTagsJson),
                "expected tags '" + expectedTagsJson.toString() + "' matches actual tags'" + actualTagsJson.toString() + "'");
    }

    @Test (dependsOnMethods = "updateRecordTags", priority = 5)
    public void searchRecords() throws Exception {

        WalletSearch search = WalletSearch.open(wallet, type, queryEmpty, optionsTags).get();

        String searchRecordsJson = search.fetchNextRecords(wallet, 1).get();

        JSONObject searchRecords = new JSONObject(searchRecordsJson);

        JSONArray records = searchRecords.getJSONArray("records");

        Assert.assertEquals(1, records.length());

        JSONObject expected = new JSONObject()
                .put("id", id)
                .putOpt("type", JSONObject.NULL)
                .put("value", value2)
                .put("tags", tags);
        JSONObject expectedTagsJson = new JSONObject(expected.getString("tags"));

        JSONObject actual = new JSONObject(records.get(0));
        Assert.assertEquals(expected.getString("value"), actual.getString("value"), "Value is as expected");

        JSONObject actualTagsJson = new JSONObject(actual.getString("tags"));
        Assert.assertTrue(expectedTagsJson.similar(actualTagsJson),
                "expected tags '" + expectedTagsJson.toString() + "' matches actual tags'" + actualTagsJson.toString() + "'");

        Assert.assertTrue(expected.similar(records.get(0)));

        search.close();
    }

    @Test (dependsOnMethods = "updateRecordTags", priority = 6)
    public void deleteTags() throws Exception {

        WalletRecord.deleteTags(wallet, type, id, "[\"tagName1\"]").get();

        JSONObject expected = new JSONObject()
                .put("id", id)
                .putOpt("type", JSONObject.NULL)
                .put("value", value2)
                .put("tags", tagsWithDeletedTag1);

        // get record with options: retriveTags
        String recordJson = WalletRecord.get(wallet, type, id, optionsTags).get();
        JSONObject actual = new JSONObject(recordJson);

        Assert.assertEquals(expected.getString("value"), actual.getString("value"), "Value is as expected");

        JSONObject expectedTagsJson = new JSONObject(expected.getString("tags"));
        JSONObject actualTagsJson = new JSONObject(actual.getString("tags"));
        Assert.assertTrue(expectedTagsJson.similar(actualTagsJson),
                "expected tags '" + expectedTagsJson.toString() + "' matches actual tags'" + actualTagsJson.toString() + "'");
    }

    @Test (dependsOnMethods = "deleteTags", priority = 7)
    public void addTags() throws Exception {

        WalletRecord.addTags(wallet, type, id, "{\"tagName1\": \"str1\"}").get();

        JSONObject expected = new JSONObject()
                .put("id", id)
                .putOpt("type", JSONObject.NULL)
                .put("value", value2)
                .put("tags", tags);

        // get record with options: retriveTags
        String recordJson = WalletRecord.get(wallet, type, id, optionsTags).get();
        JSONObject actual = new JSONObject(recordJson);

        Assert.assertEquals(expected.getString("value"), actual.getString("value"), "Value is as expected");

        JSONObject expectedTagsJson = new JSONObject(expected.getString("tags"));
        JSONObject actualTagsJson = new JSONObject(actual.getString("tags"));
        Assert.assertTrue(expectedTagsJson.similar(actualTagsJson),
                "expected tags '" + expectedTagsJson.toString() + "' matches actual tags'" + actualTagsJson.toString() + "'");
    }

    @Test (dependsOnMethods = "addTags", priority = 8)
    public void deleteRecord() throws Exception {

        WalletRecord.delete(wallet, type, id).get();

        try {
            WalletRecord.get(wallet, type, id, optionsTags).get();
            Assert.assertTrue(false); // this line should not be reached, previous line should throw an exception
        } catch (Exception e) {
            Assert.assertTrue(e instanceof ExecutionException, "Expected Exception is of ExecutionException type. Actaul type is: " + e.getClass());
            Assert.assertTrue(e.getCause() instanceof WalletNotFoundException, "Expected Cause is WalletNotFoundException, actual is " + e.getCause().toString());
        }
    }

    @Test (dependsOnMethods = "createAndOpenWallet", priority = 9)
    public void deleteWallet() throws Exception {

        Wallet.deleteWallet(walletName, CREDENTIALS).get();
        // create wallet with same name as proof that delete was successful
        Wallet.createWallet(POOL, walletName, TYPE, CONFIG, CREDENTIALS).get();
    }

    @AfterClass(alwaysRun = true)
    public void afterMethod() throws IndyException, InterruptedException, ExecutionException {

        Wallet.deleteWallet(walletName, CREDENTIALS).get();
    }
}
