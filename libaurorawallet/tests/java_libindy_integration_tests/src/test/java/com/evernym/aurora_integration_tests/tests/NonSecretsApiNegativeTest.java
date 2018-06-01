package com.evernym.aurora_integration_tests.tests;

import org.hyperledger.indy.sdk.IOException;
import org.hyperledger.indy.sdk.IndyException;
import org.hyperledger.indy.sdk.InvalidStructureException;
import org.hyperledger.indy.sdk.non_secrets.WalletRecord;
import org.hyperledger.indy.sdk.wallet.Wallet;
import org.hyperledger.indy.sdk.wallet.WalletInputException;
import org.hyperledger.indy.sdk.wallet.WalletItemNotFoundException;
import org.testng.Assert;
import org.testng.annotations.AfterClass;
import org.testng.annotations.AfterMethod;
import org.testng.annotations.DataProvider;
import org.testng.annotations.Test;

import java.util.ArrayList;
import java.util.concurrent.ExecutionException;

public class NonSecretsApiNegativeTest extends BaseTest {

    private String walletName = "testWallet" + System.currentTimeMillis();
    private Wallet wallet = null;


    @DataProvider()
    public Object[][] wrongCredentials() {


        // wrong credentials
        String missingCredentialsKey = "{" +
                "    \"storage_credentials\": {" +
                "        \"user\": \"" + CREDENTIALS_USERNAME + "\"," +
                "        \"pass\": \"" + CREDENTIALS_PASSWORD + "\"" +
                "    }" +
                "}";
        String expectedErrorClass = WalletInputException.class.toString();
        if(expectedErrorClass.startsWith("class "))
            expectedErrorClass = expectedErrorClass.substring("class ".length());
        String expectedErrorClassWithMessage = expectedErrorClass + ": Input provided to wallet operations is considered not valid.";

        String missingCredentialsUser = "{" +
                "    \"key\": \"" + CREDENTIALS_KEY + "\"," +
                "    \"storage_credentials\": {" +
                "        \"pass\": \"" + CREDENTIALS_PASSWORD + "\"" +
                "    }" +
                "}";
        expectedErrorClass = InvalidStructureException.class.toString();
        if(expectedErrorClass.startsWith("class "))
            expectedErrorClass = expectedErrorClass.substring("class ".length());
        String expectedErrorClassWithMessage2 = expectedErrorClass + ": A value being processed is not valid.";


        String missingCredentialsPassword = "{" +
                "    \"key\": \"" + CREDENTIALS_KEY + "\"," +
                "    \"storage_credentials\": {" +
                "        \"user\": \"" + CREDENTIALS_USERNAME + "\"" +
                "    }" +
                "}";
        String wrongCredentialsPassword = "{" +
                "    \"key\": \"" + CREDENTIALS_KEY + "\"," +
                "    \"storage_credentials\": {" +
                "        \"user\": \"" + CREDENTIALS_USERNAME + "\"," +
                "        \"pass\": \"" + CREDENTIALS_PASSWORD + "WRONG" + "\"" +
                "    }" +
                "}";

        expectedErrorClass = IOException.class.toString();
        if(expectedErrorClass.startsWith("class "))
            expectedErrorClass = expectedErrorClass.substring("class ".length());
        String expectedErrorClassWithMessage3 = expectedErrorClass + ": An IO error occurred.";

        Object[][] toReturn = {
                {missingCredentialsKey, expectedErrorClassWithMessage},
                {missingCredentialsUser, expectedErrorClassWithMessage2},
                {missingCredentialsPassword, expectedErrorClassWithMessage2},
                {wrongCredentialsPassword, expectedErrorClassWithMessage3}
        };
        return toReturn;

    }

    @DataProvider()
    public Object[][] wrongConfig() {

        String expectedErrorClass = null;

        // wrong config
        String missingReadHost = "{" +
                "   \"write_host\": \"" + CONFIG_WRITE_HOST + "\"," +
                "   \"port\": " + CONFIG_PORT +
                "}";
        expectedErrorClass = InvalidStructureException.class.toString();
        if(expectedErrorClass.startsWith("class "))
            expectedErrorClass = expectedErrorClass.substring("class ".length());
        String expectedErrorClassWithMessage = expectedErrorClass + ": A value being processed is not valid.";

        String missingWriteHost = "{" +
                "   \"read_host\": \"" + CONFIG_READ_HOST + "\"," +
                "   \"port\": " + CONFIG_PORT +
                "}";

        String missingPort = "{" +
                "   \"read_host\": \"" + CONFIG_READ_HOST + "\"," +
                "   \"write_host\": \"" + CONFIG_WRITE_HOST + "\"" +
                "}";

        String wrongPort = "{" +
                "   \"read_host\": \"" + CONFIG_READ_HOST + "\"," +
                "   \"write_host\": \"" + CONFIG_WRITE_HOST + "\"," +
                "   \"port\": " + CONFIG_PORT+"2" +
                "}";
        expectedErrorClass = IOException.class.toString();
        if(expectedErrorClass.startsWith("class "))
            expectedErrorClass = expectedErrorClass.substring("class ".length());
        String expectedErrorClassWithMessage2 = expectedErrorClass + ": An IO error occurred.";


        Object[][] toReturn = {
                {missingReadHost, expectedErrorClassWithMessage},
                {missingWriteHost, expectedErrorClassWithMessage},
                {missingPort, expectedErrorClassWithMessage},
                {wrongPort, expectedErrorClassWithMessage2}
        };
        return toReturn;
    }
    
    @Test(dataProvider = "wrongConfig")
    public void createAndOpenWitInvalidConfig(String wrongConfig, String expectedErrorClass) {
        try {
            Wallet.createWallet(POOL, walletName, TYPE, wrongConfig, CREDENTIALS).get();
            wallet = Wallet.openWallet(walletName, null, CREDENTIALS).get();
            Assert.assertTrue(false); // this line should not be reached, previous line should throw an exception
        } catch (Exception e) {
            Assert.assertTrue(e instanceof ExecutionException, "Expected Exception is of ExecutionException type. Actaul type is: " + e.getClass());
            Assert.assertEquals(e.getCause().toString(), expectedErrorClass, "Cause is as expected");
        }
    }

    @Test(dataProvider = "wrongCredentials")
    public void createAndOpenWitInvalidCredentials(String wrongCredentials, String expectedErrorClass) {
        try {
            Wallet.createWallet(POOL, walletName, TYPE, CONFIG, wrongCredentials).get();
            wallet = Wallet.openWallet(walletName, null, wrongCredentials).get();
            Assert.assertTrue(false); // this line should not be reached, previous line should throw an exception
        } catch (Exception e) {
            Assert.assertTrue(e instanceof ExecutionException, "Expected Exception is of ExecutionException type. Actaul type is: " + e.getClass());
            Assert.assertEquals(e.getCause().toString(), expectedErrorClass, "Cause is as expected");
        }
    }

    @AfterMethod(alwaysRun = true)
    public void afterMethod() throws IndyException, InterruptedException, ExecutionException {

        try{Wallet.deleteWallet(walletName, CREDENTIALS).get();}
        catch(Exception e){}
    }
}

