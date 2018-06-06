package aurora_integration_tests.tests;

import org.hyperledger.indy.sdk.IOException;
import org.hyperledger.indy.sdk.IndyException;
import org.hyperledger.indy.sdk.InvalidStructureException;
import org.hyperledger.indy.sdk.LibIndy;
import org.hyperledger.indy.sdk.non_secrets.WalletRecord;
import org.hyperledger.indy.sdk.non_secrets.WalletSearch;
import org.hyperledger.indy.sdk.wallet.*;
import org.json.JSONArray;
import org.json.JSONObject;
import org.testng.Assert;
import org.testng.annotations.AfterMethod;
import org.testng.annotations.DataProvider;
import org.testng.annotations.Test;

import java.util.concurrent.ExecutionException;

public class NonSecretsApiNegativeTest extends BaseTest {

    private String walletName = "testWallet" + System.currentTimeMillis();
    private Wallet wallet = null;


    String id = "RecordId";
    String id2 = "RecordId2";
    String value = "RecordValue";
    String value2 = "RecordValue2";


    @DataProvider()
    public Object[][] wrongCredentials() {

        // wrong credentials
        JSONObject json = new JSONObject(CREDENTIALS);
        json.remove("key");
        String missingCredentialsKey =  json.toString();

        String expectedErrorClass = WalletInputException.class.toString();
        if(expectedErrorClass.startsWith("class "))
            expectedErrorClass = expectedErrorClass.substring("class ".length());
        String expectedErrorClassWithMessage = expectedErrorClass + ": Input provided to wallet operations is considered not valid.";

        json = new JSONObject(CREDENTIALS);
        json.getJSONObject("storage_credentials").remove("user");
        String missingCredentialsUser = json.toString();

        expectedErrorClass = InvalidStructureException.class.toString();
        if(expectedErrorClass.startsWith("class "))
            expectedErrorClass = expectedErrorClass.substring("class ".length());
        String expectedErrorClassWithMessage2 = expectedErrorClass + ": A value being processed is not valid.";


        json = new JSONObject(CREDENTIALS);
        json.getJSONObject("storage_credentials").remove("pass");
        String missingCredentialsPassword = json.toString();


        json = new JSONObject(CREDENTIALS);
        json.getJSONObject("storage_credentials").put("pass", CREDENTIALS_PASSWORD + "WRONG");
        String wrongCredentialsPassword = json.toString();

        expectedErrorClass = IOException.class.toString();
        if(expectedErrorClass.startsWith("class "))
            expectedErrorClass = expectedErrorClass.substring("class ".length());
        String expectedErrorClassWithMessage3 = expectedErrorClass + ": An IO error occurred.";

        Object[][] toReturn = {
                {missingCredentialsKey, expectedErrorClassWithMessage, "missingCredentialsKey"},
                {missingCredentialsUser, expectedErrorClassWithMessage2, "missingCredentialsUser"},
                {missingCredentialsPassword, expectedErrorClassWithMessage2, "missingCredentialsPassword"},
                {wrongCredentialsPassword, expectedErrorClassWithMessage3, "wrongCredentialsPassword"}
        };
        return toReturn;

    }

    @DataProvider()
    public Object[][] wrongConfig() {

        String expectedErrorClass = null;

        // wrong config
        JSONObject json = new JSONObject(CONFIG); json.remove("read_host");
        String missingReadHost = json.toString();

        expectedErrorClass = InvalidStructureException.class.toString();
        if(expectedErrorClass.startsWith("class "))
            expectedErrorClass = expectedErrorClass.substring("class ".length());
        String expectedErrorClassWithMessage = expectedErrorClass + ": A value being processed is not valid.";


        json = new JSONObject(CONFIG); json.remove("write_host");
        String missingWriteHost = json.toString();

        json = new JSONObject(CONFIG); json.remove("port");
        String missingPort = json.toString();

        json = new JSONObject(CONFIG); json.remove("db_name");
        String missingDBName = json.toString();

        expectedErrorClass = IOException.class.toString();
        if(expectedErrorClass.startsWith("class "))
            expectedErrorClass = expectedErrorClass.substring("class ".length());
        String expectedErrorClassWithMessage2 = expectedErrorClass + ": An IO error occurred.";


        String wrongPort = getConfig(CONFIG_READ_HOST, CONFIG_WRITE_HOST, CONFIG_PORT+"2", CONFIG_DB_NAME);
        String wrongDBName = getConfig(CONFIG_READ_HOST, CONFIG_WRITE_HOST, CONFIG_PORT, CONFIG_DB_NAME+"_2");
        String wrongReadHost = getConfig("1.2.3.4", CONFIG_WRITE_HOST, CONFIG_PORT, CONFIG_DB_NAME);
        String wrongWriteHost = getConfig(CONFIG_READ_HOST, "1.2.3.4", CONFIG_PORT, CONFIG_DB_NAME);

        Object[][] toReturn = {
                // config, expected error message, scenario
                {missingReadHost, expectedErrorClassWithMessage, "missingReadHost"},
                {missingWriteHost, expectedErrorClassWithMessage, "missingWriteHost"},
                {missingPort, expectedErrorClassWithMessage, "missingPort"},
                {missingDBName, expectedErrorClassWithMessage, "missingDBName"},
                {wrongPort, expectedErrorClassWithMessage2, "wrongPort"},
                {wrongDBName, expectedErrorClassWithMessage2, "wrongDBName"}
                ,{wrongReadHost, expectedErrorClassWithMessage2, "wrongReadHost"}
                ,{wrongWriteHost, expectedErrorClassWithMessage2, "wrongWriteHost"}
        };
        return toReturn;
    }
    
    @Test(dataProvider = "wrongConfig")
    public void createAndOpenWitInvalidConfig(String wrongConfig, String expectedErrorClass, String scenario) {
        try {
            Wallet.createWallet(POOL, walletName, WALLET_TYPE, wrongConfig, CREDENTIALS).get();
            wallet = Wallet.openWallet(walletName, null, CREDENTIALS).get();
            Assert.assertTrue(false, "Scenario: " + scenario); // this line should not be reached, previous line should throw an exception
        } catch (Exception e) {
            Assert.assertTrue(e instanceof ExecutionException, "Scenario: " + scenario + ", Expected Exception is of ExecutionException ITEM_TYPE. Actaul ITEM_TYPE is: " + e.getClass());
            Assert.assertEquals(e.getCause().toString(), expectedErrorClass, "Scenario: " + scenario + ", Cause is as expected");
        }
    }

    @Test(dataProvider = "wrongCredentials")
    public void createAndOpenWitInvalidCredentials(String wrongCredentials, String expectedErrorClass, String scenario) {
        try {
            Wallet.createWallet(POOL, walletName, WALLET_TYPE, CONFIG, wrongCredentials).get();
            wallet = Wallet.openWallet(walletName, null, wrongCredentials).get();
            Assert.assertTrue(false, "Scenario: " + scenario ); // this line should not be reached, previous line should throw an exception
        } catch (Exception e) {
            Assert.assertTrue(e instanceof ExecutionException, "Scenario: " + scenario + ", Expected Exception is of ExecutionException ITEM_TYPE. Actaul ITEM_TYPE is: " + e.getClass());
            Assert.assertEquals(e.getCause().toString(), expectedErrorClass, "Scenario: " + scenario + ", Cause is as expected");
        }
    }


    @DataProvider(name = "allGetOptionsJson")
    private Object[][] allGetOptionsJson() {
        Object[][] toReturn = {
                {GET_OPTIONS_ALL,           "GET_OPTIONS_ALL"},
                {GET_OPTIONS_EMPTY,         "GET_OPTIONS_EMPTY"},
                {GET_OPTIONS_TAGS_ONLY,     "GET_OPTIONS_TAGS_ONLY"},
                {GET_OPTIONS_VALUE_ONLY,    "GET_OPTIONS_VALUE_ONLY"},
                {GET_OPTIONS_TYPE_ONLY,     "GET_OPTIONS_TYPE_ONLY"}
        };
        return toReturn;
    }

    @Test (dataProvider = "allGetOptionsJson")
    public void getNonExistingKey(String getOptionJson, String scenario) throws IndyException, ExecutionException, InterruptedException {

        // create and open wallet
        Wallet.createWallet(POOL, walletName, WALLET_TYPE, CONFIG, CREDENTIALS).get();
        wallet = Wallet.openWallet(walletName, null, CREDENTIALS).get();

        // add one record
        WalletRecord.add(wallet, ITEM_TYPE, id, value, TAGS_EMPTY).get();

        // try to get non-existing record
        try {
            String recordJson = WalletRecord.get(wallet, ITEM_TYPE, id2, getOptionJson).get();
            Assert.assertTrue(false, "Scenario: " + scenario + ": This line should not be reached but actual result is: " + recordJson);
        } catch(Exception e) {
            Assert.assertTrue(e instanceof ExecutionException, "Scenario: " + scenario + ": Expected Exception is of ExecutionException ITEM_TYPE. Actaul ITEM_TYPE is: " + e.getClass());
            Assert.assertTrue(e.getCause() instanceof WalletItemNotFoundException, "Scenario: " + scenario + ": Cause is as expected. Actual cause is: " + e.getCause().getClass());
        }
    }

    @DataProvider(name = "badFormatOptionsJson")
    private Object[][] badFormatOptionsJson() {
        Object[][] toReturn = {
                // jsonOptions, scenario
                {"\"\"", "emptyString"},
                {GET_OPTIONS_ALL.substring(1), "noCloseBracket"},
                {GET_OPTIONS_ALL.substring(0, GET_OPTIONS_ALL.length()-1), "noOpenBracket"},
                {GET_OPTIONS_ALL.substring(1, GET_OPTIONS_ALL.length()-1), "noOpenNoCloseBracket"},
                {"{retrieveTags: true, retrieveValue: true, retrieveType: true}", "noDoubleQuotes"}
        };
        return toReturn;
    }

    @Test (dataProvider = "badFormatOptionsJson")
    public void getKeyWithBadFormatForOptionsJson (String optionsJson, String scenario) throws IndyException, ExecutionException, InterruptedException {

        // create and open wallet
        Wallet.createWallet(POOL, walletName, WALLET_TYPE, CONFIG, CREDENTIALS).get();
        wallet = Wallet.openWallet(walletName, null, CREDENTIALS).get();

        // add one record
        WalletRecord.add(wallet, ITEM_TYPE, id, value, TAGS_EMPTY).get();

        // try to get item with bad format options json
        try {
            String recordJson = WalletRecord.get(wallet, ITEM_TYPE, id2, optionsJson).get();
            Assert.assertTrue(false, "Scenario: " + scenario + ": This line should not be reached but actual result is: " + recordJson);
        } catch (Exception e) {
            Assert.assertTrue(e instanceof ExecutionException, "Scenario: " + scenario + ": Expected Exception is of ExecutionException ITEM_TYPE. Actaul ITEM_TYPE is: " + e.getClass());
            Assert.assertTrue(e.getCause() instanceof InvalidStructureException, "Scenario: " + scenario + ": Cause is as expected. Actual cause is: " + e.getCause().getClass());
        }
    }

    @Test
    public void updateNonExistingKey() throws IndyException, ExecutionException, InterruptedException {

        // create and open wallet
        Wallet.createWallet(POOL, walletName, WALLET_TYPE, CONFIG, CREDENTIALS).get();
        wallet = Wallet.openWallet(walletName, null, CREDENTIALS).get();

        // add one record
        WalletRecord.add(wallet, ITEM_TYPE, id, value, TAGS_EMPTY).get();

        // try to update non-existing record value
        try {
            WalletRecord.updateValue(wallet, ITEM_TYPE, id2, "new_value").get();
            Assert.assertTrue(false, "This line should not be reached");
        } catch(Exception e) {
            Assert.assertTrue(e instanceof ExecutionException, "Expected Exception is of ExecutionException ITEM_TYPE. Actaul ITEM_TYPE is: " + e.getClass());
            Assert.assertTrue(e.getCause() instanceof WalletItemNotFoundException, "Cause is as expected. Actual cause is: " + e.getCause().getClass());
        }

        // try to update non-existing record tags
        try {
            WalletRecord.updateTags(wallet, ITEM_TYPE, id2, "{\"tag1\": \"value1\"}").get();
            Assert.assertTrue(false, "This line should not be reached");
        } catch(Exception e) {
            Assert.assertTrue(e instanceof ExecutionException, "Expected Exception is of ExecutionException ITEM_TYPE. Actaul ITEM_TYPE is: " + e.getClass());
            Assert.assertTrue(e.getCause() instanceof WalletItemNotFoundException, "Cause is as expected. Actual cause is: " + e.getCause().getClass());
        }

    }


    @Test
    public void deleteNonExistingKey() throws IndyException, ExecutionException, InterruptedException {

        // create and open wallet
        Wallet.createWallet(POOL, walletName, WALLET_TYPE, CONFIG, CREDENTIALS).get();
        wallet = Wallet.openWallet(walletName, null, CREDENTIALS).get();

        // add one record
        WalletRecord.add(wallet, ITEM_TYPE, id, value, TAGS_EMPTY).get();

        // try to delete non-existing record value
        try {
            WalletRecord.delete(wallet, ITEM_TYPE, id2).get();
            Assert.assertTrue(false, "This line should not be reached");
        } catch(Exception e) {
            Assert.assertTrue(e instanceof ExecutionException, "Expected Exception is of ExecutionException ITEM_TYPE. Actaul ITEM_TYPE is: " + e.getClass());
            Assert.assertTrue(e.getCause() instanceof WalletItemNotFoundException, "Cause is as expected. Actual cause is: " + e.getCause().getClass());
        }
    }

    @Test
    public void deleteTagFromNonExistingKey() throws IndyException, ExecutionException, InterruptedException {

        // create and open wallet
        Wallet.createWallet(POOL, walletName, WALLET_TYPE, CONFIG, CREDENTIALS).get();
        wallet = Wallet.openWallet(walletName, null, CREDENTIALS).get();

        // add one record
        WalletRecord.add(wallet, ITEM_TYPE, id, value, TAGS_EMPTY).get();

        // try to delete tag for non-existing item
        try {
            WalletRecord.deleteTags(wallet, ITEM_TYPE, id2, "[\"tag1\"]").get();
            Assert.assertTrue(false, "This line should not be reached");
        } catch(Exception e) {
            Assert.assertTrue(e instanceof ExecutionException, "Expected Exception is of ExecutionException ITEM_TYPE. Actaul ITEM_TYPE is: " + e.getClass());
            Assert.assertTrue(e.getCause() instanceof WalletItemNotFoundException, "Cause is as expected. Actual cause is: " + e.getCause().getClass());
        }
    }


    @DataProvider(name = "badFormatTagsList")
    private Object[][] badFormatTagsList() {
        Object[][] toReturn = {
                // jsonOptions, scenario
                {"\"\"", "emptyString"},
                {"[\"tag\", \"tag2\"", "noCloseBracket"},
                {"\"tag\", \"tag2\"]", "noOpenBracket"},
                {"\"tag\", \"tag2\"", "noOpenNoCloseBracket"},
                {"[tag]", "noDoubleQuotesForTag"}
        };
        return toReturn;
    }

    @Test (dataProvider = "badFormatTagsList")
    public void deleteTagWithBadFormatForTagsList(String jsonTagsList, String scenario) throws IndyException, ExecutionException, InterruptedException {

        // create and open wallet
        Wallet.createWallet(POOL, walletName, WALLET_TYPE, CONFIG, CREDENTIALS).get();
        wallet = Wallet.openWallet(walletName, null, CREDENTIALS).get();

        // add one record
        WalletRecord.add(wallet, ITEM_TYPE, id, value, TAGS_EMPTY).get();

        // try to delete tags with bad format for tags list
        try {
            WalletRecord.deleteTags(wallet, ITEM_TYPE, id, jsonTagsList).get();
            Assert.assertTrue(false, "Scenario: " + scenario + ": This line should not be reached");
        } catch(Exception e) {
            Assert.assertTrue(e instanceof ExecutionException, "Scenario: " + scenario + ": Expected Exception is of ExecutionException ITEM_TYPE. Actaul ITEM_TYPE is: " + e.getClass());
            Assert.assertTrue(e.getCause() instanceof WalletDecodingException, "Scenario: " + scenario + ": Cause is as expected. Actual cause is: " + e.getCause().getClass());
        }
    }

    @Test (dataProvider = "badFormatTagsList")
    public void addTagWithBadFormatForTagsList(String jsonTagsList, String scenario) throws IndyException, ExecutionException, InterruptedException {

        // create and open wallet
        Wallet.createWallet(POOL, walletName, WALLET_TYPE, CONFIG, CREDENTIALS).get();
        wallet = Wallet.openWallet(walletName, null, CREDENTIALS).get();

        // add one record
        WalletRecord.add(wallet, ITEM_TYPE, id, value, TAGS_EMPTY).get();

        // try to add tags with bad format for tags list
        try {
            WalletRecord.addTags(wallet, ITEM_TYPE, id, jsonTagsList).get();
            Assert.assertTrue(false, "Scenario: " + scenario + ": This line should not be reached");
        } catch(Exception e) {
            Assert.assertTrue(e instanceof ExecutionException, "Scenario: " + scenario + ": Expected Exception is of ExecutionException ITEM_TYPE. Actaul ITEM_TYPE is: " + e.getClass());
            Assert.assertTrue(e.getCause() instanceof WalletDecodingException, "Scenario: " + scenario + ": Cause is as expected. Actual cause is: " + e.getCause().getClass());
        }
    }

    @Test
    public void insertDuplicateKey() throws IndyException, ExecutionException, InterruptedException {
        // create and open wallet
        Wallet.createWallet(POOL, walletName, WALLET_TYPE, CONFIG, CREDENTIALS).get();
        wallet = Wallet.openWallet(walletName, null, CREDENTIALS).get();

        // add one record
        WalletRecord.add(wallet, ITEM_TYPE, id, value, TAGS_EMPTY).get();

        try {
            WalletRecord.add(wallet, ITEM_TYPE, id, value, TAGS_EMPTY).get();
            Assert.assertTrue(false, "This line should not be reached");
        } catch (Exception e) {
            Assert.assertTrue(e instanceof ExecutionException, "Expected Exception is of ExecutionException ITEM_TYPE. Actaul ITEM_TYPE is: " + e.getClass());
            Assert.assertTrue(e.getCause() instanceof WalletItemAlreadyExistsException, "Cause is as expected. Actual cause is: " + e.getCause().getClass());
        }
    }

    @Test
    public void openAnOpenedWallet() throws IndyException, ExecutionException, InterruptedException {
        // create and open wallet
        Wallet.createWallet(POOL, walletName, WALLET_TYPE, CONFIG, CREDENTIALS).get();
        wallet = Wallet.openWallet(walletName, null, CREDENTIALS).get();

        try {
            Wallet.openWallet(walletName, null, CREDENTIALS).get();
            Assert.assertTrue(false, "This line should not be reached");
        } catch (Exception e) {
            Assert.assertTrue(e instanceof ExecutionException, "Expected Exception is of ExecutionException ITEM_TYPE. Actaul ITEM_TYPE is: " + e.getClass());
            Assert.assertTrue(e.getCause() instanceof WalletAlreadyOpenedException, "Cause is as expected. Actual cause is: " + e.getCause().getClass());
        }
    }

    @Test
    public void openNonExistingWallet() {
        try {
            wallet = Wallet.openWallet(walletName, null, CREDENTIALS).get();
        } catch (Exception e) {
            Assert.assertTrue(e instanceof ExecutionException, "Expected Exception is of ExecutionException ITEM_TYPE. Actaul ITEM_TYPE is: " + e.getClass());
            Assert.assertTrue(e.getCause() instanceof WalletNotFoundException, "Cause is as expected. Actual cause is: " + e.getCause().getClass());
        }
    }

    @Test
    public void createWalletWithDuplicateName() throws IndyException, ExecutionException, InterruptedException {
        Wallet.createWallet(POOL, walletName, WALLET_TYPE, CONFIG, CREDENTIALS).get();
        try {
            Wallet.createWallet(POOL, walletName, WALLET_TYPE, CONFIG, CREDENTIALS).get();
        } catch (Exception e) {
            Assert.assertTrue(e instanceof ExecutionException, "Expected Exception is of ExecutionException ITEM_TYPE. Actaul ITEM_TYPE is: " + e.getClass());
            Assert.assertTrue(e.getCause() instanceof WalletExistsException, "Cause is as expected. Actual cause is: " + e.getCause().getClass());
        }
    }

    @Test ()
    public void deleteNonexistingWallet() throws Exception {

        try {
            Wallet.deleteWallet(walletName+"blabla", CREDENTIALS).get();
        } catch (Exception e) {
            Assert.assertTrue(e instanceof ExecutionException, "Expected Exception is of ExecutionException ITEM_TYPE. Actaul ITEM_TYPE is: " + e.getClass());
            Assert.assertTrue(e.getCause() instanceof IOException, "Cause is as expected. Actual cause is: " + e.getCause().getClass());
        }
    }




    /**
     * Cleanup method
     */

    @AfterMethod(alwaysRun = true)
    public void afterMethod() {

        try{
            if(wallet != null) {
                wallet.closeWallet().get();
                wallet = null;
            }
            Wallet.deleteWallet(walletName, CREDENTIALS).get();
        }
        catch(Exception e){}
    }
}

