package aurora_integration_tests.tests;

import org.hyperledger.indy.sdk.IOException;
import org.hyperledger.indy.sdk.IndyException;
import org.hyperledger.indy.sdk.InvalidStructureException;
import org.hyperledger.indy.sdk.wallet.Wallet;
import org.hyperledger.indy.sdk.wallet.WalletInputException;
import org.json.JSONObject;
import org.testng.Assert;
import org.testng.annotations.AfterMethod;
import org.testng.annotations.DataProvider;
import org.testng.annotations.Test;

import java.util.concurrent.ExecutionException;

public class NonSecretsApiNegativeTest extends BaseTest {

    private String walletName = "testWallet" + System.currentTimeMillis();
    private Wallet wallet = null;


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
                {wrongDBName, expectedErrorClassWithMessage2, "wrongDBName"},
                {wrongReadHost, expectedErrorClassWithMessage2, "wrongReadHost"},
                {wrongWriteHost, expectedErrorClassWithMessage2, "wrongWriteHost"}
        };
        return toReturn;
    }
    
    @Test(dataProvider = "wrongConfig")
    public void createAndOpenWitInvalidConfig(String wrongConfig, String expectedErrorClass, String scenario) {
        try {
            Wallet.createWallet(POOL, walletName, TYPE, wrongConfig, CREDENTIALS).get();
            wallet = Wallet.openWallet(walletName, null, CREDENTIALS).get();
            Assert.assertTrue(false, "Scenario: " + scenario); // this line should not be reached, previous line should throw an exception
        } catch (Exception e) {
            Assert.assertTrue(e instanceof ExecutionException, "Scenario: " + scenario + ", Expected Exception is of ExecutionException type. Actaul type is: " + e.getClass());
            Assert.assertEquals(e.getCause().toString(), expectedErrorClass, "Scenario: " + scenario + ", Cause is as expected");
        }
    }

    @Test(dataProvider = "wrongCredentials")
    public void createAndOpenWitInvalidCredentials(String wrongCredentials, String expectedErrorClass, String scenario) {
        try {
            Wallet.createWallet(POOL, walletName, TYPE, CONFIG, wrongCredentials).get();
            wallet = Wallet.openWallet(walletName, null, wrongCredentials).get();
            Assert.assertTrue(false, "Scenario: " + scenario ); // this line should not be reached, previous line should throw an exception
        } catch (Exception e) {
            Assert.assertTrue(e instanceof ExecutionException, "Scenario: " + scenario + ", Expected Exception is of ExecutionException type. Actaul type is: " + e.getClass());
            Assert.assertEquals(e.getCause().toString(), expectedErrorClass, "Scenario: " + scenario + ", Cause is as expected");
        }
    }

    @AfterMethod(alwaysRun = true)
    public void afterMethod() throws IndyException, InterruptedException, ExecutionException {

        try{Wallet.deleteWallet(walletName, CREDENTIALS).get();}
        catch(Exception e){}
    }
}

