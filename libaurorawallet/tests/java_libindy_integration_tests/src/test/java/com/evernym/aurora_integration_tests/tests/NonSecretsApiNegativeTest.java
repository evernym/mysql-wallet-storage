package com.evernym.aurora_integration_tests.tests;

import org.testng.annotations.DataProvider;

public class NonSecretsApiNegativeTest extends BaseTest {

    @DataProvider()
    public Object[][] wrongConfigAndCredentials() {


        // wrong credentials
        String missingCredentialsKey = "{" +
                "    \"storage_credentials\": {" +
                "        \"user\": \"" + CREDENTIALS_USERNAME + "\"," +
                "        \"pass\": \"" + CREDENTIALS_PASSWORD + "\"" +
                "    }" +
                "}";
        String expectedErrorCause = "";

        String missingCredentialsUser = "{" +
                "    \"key\": \"" + CREDENTIALS_KEY + "\"," +
                "    \"storage_credentials\": {" +
                "        \"pass\": \"" + CREDENTIALS_PASSWORD + "\"" +
                "    }" +
                "}";
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
                "        \"pass\": \"" + CREDENTIALS_PASSWORD+"WRONG" + "\"" +
                "    }" +
                "}";



        // wrong config
        String missingReadHost = "{" +
                "   \"write_host\": \"" + CONFIG_WRITE_HOST + "\"," +
                "   \"port\": " + CONFIG_PORT +
                "}";
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

        Object[][] toReturn = {
                {missingCredentialsKey, expectedErrorCause},
                {missingCredentialsUser, expectedErrorCause},
                {missingCredentialsPassword, expectedErrorCause},
                {wrongCredentialsPassword, expectedErrorCause}
        };
        return toReturn;
    }
}
