package com.evernym.aurora_integration_tests.tests;

import com.evernym.aurora_integration_tests.main.AuroraPluggableStorage;
import org.testng.annotations.BeforeSuite;

public class BaseTest {

    protected String TYPE = "aurora";

    protected static final String POOL = "Pool1";

    protected static final String CONFIG = "{" +
            "   \"read_host\": \"localhost\"," +
            "   \"write_host\": \"localhost\"," +
            "   \"port\": 3306" +
            "}";
    protected static final String CREDENTIALS = "{" +
            "    \"key\": \"MTIzNDU2Nzg5MDEyMzQ1Njc4OTAxMjM0NTY3ODkwMTI=\"," +
            "    \"storage_credentials\": {" +
            "        \"user\": \"wallet\"," +
            "        \"pass\": \"wallet\"" +
            "    }" +
            "}";


    String optionsFull = "{\"retrieveType\":true, \"retrieveValue\":true, \"retrieveTags\":true}";

    @BeforeSuite(alwaysRun = true)
    public void init() {
        AuroraPluggableStorage.api.aurora_storage_init();
    }
}
