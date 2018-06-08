package aurora_integration_tests.tests;

import org.hyperledger.indy.sdk.IndyException;
import org.hyperledger.indy.sdk.non_secrets.WalletRecord;
import org.hyperledger.indy.sdk.non_secrets.WalletSearch;
import org.hyperledger.indy.sdk.wallet.Wallet;
import org.hyperledger.indy.sdk.wallet.WalletItemNotFoundException;
import org.json.JSONArray;
import org.json.JSONObject;
import org.testng.Assert;
import org.testng.annotations.AfterClass;
import org.testng.annotations.DataProvider;
import org.testng.annotations.Test;

import java.util.concurrent.ExecutionException;

public class NonSecretsApiPositiveTest extends BaseTest {

    private String walletName = "testWallet" + System.currentTimeMillis();
    private Wallet wallet = null;

    String tagsWithDeletedTag1 =  "{\"tagName2\":\"5\",\"tagName3\":\"12\"}";
    String id = "RecordId";
    String value = "RecordValue";
    String value2 = "RecordValue2";


    private void prepareRecordsForSearch() throws IndyException, ExecutionException, InterruptedException {

        String tags = "";
        for(int i=0; i<12; i++) {
            String type = ITEM_TYPE;
            if( i % 2 == 1 ) type = ITEM_TYPE2; //every odd iteration will use ITEM_TYPE2

            // rotate tags every 2nd iteration
            if ( (i/2) % 2 == 1 ) {
                tags = TAGS2;
            } else {
                tags = TAGS3;
            }


            WalletRecord.add(wallet, type, "Search"+id+i, value, tags).get();
        }
    }


    @Test (priority = 0)
    public void createAndOpenWallet() throws Exception {

        Wallet.createWallet(POOL, walletName, WALLET_TYPE, CONFIG, CREDENTIALS).get();
        wallet = Wallet.openWallet(walletName, null, CREDENTIALS).get();
    }

    @Test (dependsOnMethods = "createAndOpenWallet", priority = 1)
    public void addRecords() throws Exception {
        WalletRecord.add(wallet, ITEM_TYPE, id, value, TAGS_EMPTY).get();

        /**
         * This can't go to beforeClass since we have the test for create and opent wallet
         * also can't be a part of searchTags test because that test uses data provider
         * and since this is actually "add records" I'll put it here
         */
        prepareRecordsForSearch();
    }

    @Test (dependsOnMethods = "addRecords", priority = 2)
    public void getRecordWithTags() throws Exception {

        String recordJson = WalletRecord.get(wallet, ITEM_TYPE, id, GET_OPTIONS_TAGS_ONLY).get();

        JSONObject actual = new JSONObject(recordJson);

        JSONObject expected = new JSONObject()
                .put("id", id)
                .putOpt("type", JSONObject.NULL)
                .put("value", JSONObject.NULL)
                .put("tags", new JSONObject());

        Assert.assertTrue(expected.similar(actual), "expected '" + expected.toString() + "' matches actual '" + actual.toString() + "'");
    }

    @Test (dependsOnMethods = "addRecords", priority = 2)
    public void getRecordWithoutTags() throws Exception {

        String recordJson = WalletRecord.get(wallet, ITEM_TYPE, id, GET_OPTIONS_EMPTY).get();

        JSONObject actual = new JSONObject(recordJson);

        JSONObject expected = new JSONObject()
                .put("id", id)
                .putOpt("type", JSONObject.NULL)
                .put("value", value) // value is returned by default
                .put("tags", JSONObject.NULL);

        Assert.assertTrue(expected.similar(actual), "expected '" + expected.toString() + "' matches actual '" + actual.toString() + "'");
    }

    @Test (dependsOnMethods = "addRecords", priority = 3)
    public void updateRecordValue() throws Exception {
        WalletRecord.updateValue(wallet, ITEM_TYPE, id, value2).get();

        JSONObject expected = new JSONObject()
                .put("id", id)
                .putOpt("type", JSONObject.NULL)
                .put("value", JSONObject.NULL)
                .put("tags", new JSONObject());

        // get record with options: retriveTags
        String recordJson = WalletRecord.get(wallet, ITEM_TYPE, id, GET_OPTIONS_TAGS_ONLY).get();
        JSONObject actual = new JSONObject(recordJson);

        Assert.assertTrue(expected.similar(actual), "expected '" + expected.toString() + "' matches actual '" + actual.toString() + "'");
    }

    @Test (dependsOnMethods = "updateRecordValue", priority = 4)
    public void updateRecordTags() throws Exception {

        WalletRecord.updateTags(wallet, ITEM_TYPE, id, TAGS).get();
        JSONObject expected = new JSONObject()
                .put("id", id)
                .putOpt("type", JSONObject.NULL)
                .put("value", JSONObject.NULL)
                .put("tags", new JSONObject(TAGS));

        // get record with options: retriveTags
        String recordJson = WalletRecord.get(wallet, ITEM_TYPE, id, GET_OPTIONS_TAGS_ONLY).get();
        JSONObject actual = new JSONObject(recordJson);

        Assert.assertTrue(expected.similar(actual),
                "expected '" + expected.toString() + "' matches actual '" + actual.toString() + "'");
    }


    // TODO: add query strings for all operators
    @DataProvider (name = "searchQueries")
    private Object[][] searchQueries() {
        String queryJson = "{" +
                "\"tagName1\": \"str1\", " +
                "\"tagName2\": \"5\"" +
                "}";

        JSONObject jsonObject = new JSONObject()
                .put("id", id)
                .putOpt("type", ITEM_TYPE)
                .put("value", value2)
                .put("tags", new JSONObject(TAGS));

        JSONArray jsonArray = new JSONArray();
        jsonArray.put(jsonObject);

        String queryJsonIn = "{" +
                "\"tagName1\": {\"$in\": [\"str1\", \"blabla\"]}, " +
                "\"tagName2\": \"5\"" +
                "}";

        String queryJsonInNot = "{" +
                "\"tagName1\": {\"$in\": [\"str1\", \"blabla\"]}, " +
                "\"$not\": {\"tagName2\": \"12\"}" +
                "}";

        JSONArray jsonArray2 = new JSONArray("[" +
                "{\"id\":\"SearchRecordId6\",\"type\":\"TestType\",\"value\":\"RecordValue\",\"tags\":{\"tagName1\":\"str2\",\"tagName2\":\"6\",\"tagName3\":\"13\"}}," +
                "{\"id\":\"SearchRecordId8\",\"type\":\"TestType\",\"value\":\"RecordValue\",\"tags\":{\"tagName1\":\"str3\",\"tagName2\":\"7\",\"tagName3\":\"14\"}}," +
                "{\"id\":\"SearchRecordId2\",\"type\":\"TestType\",\"value\":\"RecordValue\",\"tags\":{\"tagName1\":\"str2\",\"tagName2\":\"6\",\"tagName3\":\"13\"}}," +
                "{\"id\":\"SearchRecordId10\",\"type\":\"TestType\",\"value\":\"RecordValue\",\"tags\":{\"tagName1\":\"str2\",\"tagName2\":\"6\",\"tagName3\":\"13\"}}," +
                "{\"id\":\"SearchRecordId0\",\"type\":\"TestType\",\"value\":\"RecordValue\",\"tags\":{\"tagName1\":\"str3\",\"tagName2\":\"7\",\"tagName3\":\"14\"}}," +
                "{\"id\":\"SearchRecordId4\",\"type\":\"TestType\",\"value\":\"RecordValue\",\"tags\":{\"tagName1\":\"str3\",\"tagName2\":\"7\",\"tagName3\":\"14\"}}," +
                "{\"id\":\"RecordId\",\"type\":\"TestType\",\"value\":\"RecordValue2\",\"tags\":{\"tagName1\":\"str1\",\"tagName2\":\"5\",\"tagName3\":\"12\"}}]");

        String complexQuery = "{\"$not\":{\"" +
                "{xxxxxx}\":\"{xxxxxx}\"," +
                "\"$or\":[\"{xxxxxxx}\":{\"$gt\":\"{xxxxxxxxx}\"},\"$not\":{\"{xxxxxx}\":{\"$lte\":\"{xxxxxxx}\"}},{\"{xxxxxxx}\":{\"$lt\":\"{xxxxxxxx}\"}," +
                "\"$not\":{\"{xxxxxxxxx}\":{\"$gte\":\"{xxxxxxxxxx}\"}}}}],\"$not\":{\"{xxxxxxxxx}\":{\"$like\":\"{xxxxxxxxxx}\"}},{\"{xxxxxxxxx}\":\"{xxxxxxxxx}\",\"$not\":{\"{xxxxxxxxxx}\":{\"$neq\":\"{xxxxxx}\"}}}}}";

        Object[][] toReturn = {
            {queryJson, jsonArray},
            {queryJsonIn, jsonArray},
            {queryJsonInNot, jsonArray},
            {QUERY_EMPTY, jsonArray2}
        };

        return toReturn;
    }

    @Test (dependsOnMethods = "updateRecordTags", dataProvider = "searchQueries", priority = 5)
    public void searchRecordsWithQuery(String searchQuery, JSONArray expectedJSONArray) throws Exception {

        WalletSearch search = WalletSearch.open(wallet, ITEM_TYPE, searchQuery, SEARCH_OPTIONS_ALL).get();

        String searchRecordsJson = search.fetchNextRecords(wallet, 20).get();

        JSONObject searchRecords = new JSONObject(searchRecordsJson);

        Assert.assertEquals(searchRecords.getLong("totalCount"), expectedJSONArray.length());

        JSONArray records = searchRecords.getJSONArray("records");

        Assert.assertEquals(records.length(), expectedJSONArray.length());

        // similar is not working for JSON arrays so we have to iterate through array

        for (Object expectedObject : expectedJSONArray) {
            JSONObject expectedJsonObject = (JSONObject) expectedObject;
            boolean foundInActualResults = false;

            for (Object actualObject : records) {
                JSONObject actualJsonObject = (JSONObject) actualObject;
                if(expectedJsonObject.similar(actualJsonObject)) {
                    foundInActualResults = true;
                    break;
                }
            }

            Assert.assertTrue(foundInActualResults, "Row '" + expectedJsonObject.toString() + "' is found in array '" + records + "'");
        }

        search.close();
    }

    @Test (dependsOnMethods = "updateRecordTags", priority = 6)
    public void deleteTags() throws Exception {

        WalletRecord.deleteTags(wallet, ITEM_TYPE, id, "[\"tagName1\"]").get();

        JSONObject expected = new JSONObject()
                .put("id", id)
                .putOpt("type", ITEM_TYPE)
                .put("value", value2)
                .put("tags", new JSONObject(tagsWithDeletedTag1));

        // get record with options: retriveTags
        String recordJson = WalletRecord.get(wallet, ITEM_TYPE, id, GET_OPTIONS_ALL).get();
        JSONObject actual = new JSONObject(recordJson);

        Assert.assertTrue(expected.similar(actual),
                "expected '" + expected.toString() + "' matches actual'" + actual.toString() + "'");
    }

    @Test (dependsOnMethods = "deleteTags", priority = 7)
    public void addTags() throws Exception {

        WalletRecord.addTags(wallet, ITEM_TYPE, id, "{\"tagName1\": \"str1\"}").get();

        JSONObject expected = new JSONObject()
                .put("id", id)
                .putOpt("type", ITEM_TYPE)
                .put("value", value2)
                .put("tags", new JSONObject(TAGS));

        // get record with options: retriveTags
        String recordJson = WalletRecord.get(wallet, ITEM_TYPE, id, GET_OPTIONS_ALL).get();
        JSONObject actual = new JSONObject(recordJson);

        Assert.assertTrue(expected.similar(actual),
                "expected '" + expected.toString() + "' matches actual '" + actual.toString() + "'");
    }

    @Test (dependsOnMethods = "addRecords", priority = 8)
    public void deleteRecord() throws Exception {

        WalletRecord.delete(wallet, ITEM_TYPE, id).get();

        try {
            WalletRecord.get(wallet, ITEM_TYPE, id, GET_OPTIONS_TAGS_ONLY).get();
            Assert.assertTrue(false); // this line should not be reached, previous line should throw an exception
        } catch (Exception e) {
            Assert.assertTrue(e instanceof ExecutionException, "Expected Exception is of ExecutionException ITEM_TYPE. Actaul ITEM_TYPE is: " + e.getClass());
            Assert.assertTrue(e.getCause() instanceof WalletItemNotFoundException, "Expected Cause is WalletItemNotFoundException, actual is " + e.getCause().toString());
        }
    }

    @Test (dependsOnMethods = "createAndOpenWallet", priority = 9)
    public void closeAndDeleteWallet() throws Exception {

        wallet.closeWallet().get();
        Wallet.deleteWallet(walletName, CREDENTIALS).get();
        // create wallet with same name as proof that delete was successful
        Wallet.createWallet(POOL, walletName, WALLET_TYPE, CONFIG, CREDENTIALS).get();

        wallet = Wallet.openWallet(walletName, null, CREDENTIALS).get();
        wallet.closeWallet().get();
    }

    @AfterClass(alwaysRun = true)
    public void afterMethod() throws IndyException, InterruptedException, ExecutionException {

        Wallet.deleteWallet(walletName, CREDENTIALS).get();
    }
}
