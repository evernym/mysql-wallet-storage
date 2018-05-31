package com.evernym.aurora_integration_tests.tests;

import com.evernym.aurora_integration_tests.main.AuroraPluggableStorage;
import org.testng.annotations.BeforeSuite;

import java.io.*;
import java.util.Properties;

public class BaseTest {

    private final String defaultConfigPropertiesFile = "resources/test.properties";
    Properties props = new Properties();

    protected String TYPE = "aurora";

    protected static final String POOL = "Pool1";

    protected static String CONFIG;
    protected static String CREDENTIALS;

    protected static String CONFIG_READ_HOST, CONFIG_WRITE_HOST, CONFIG_PORT, CONFIG_DB_NAME;
    protected static String CREDENTIALS_KEY, CREDENTIALS_USERNAME,CREDENTIALS_PASSWORD;


    @BeforeSuite(alwaysRun = true)
    public void init() throws IOException {

        // load properties
        File configProperties = new File(defaultConfigPropertiesFile);
        props.load(new FileInputStream(defaultConfigPropertiesFile));

        // init config vars
        CONFIG_READ_HOST        = props.getProperty("config.read_host.ip");
        CONFIG_WRITE_HOST       = props.getProperty("config.write_host.ip");
        CONFIG_PORT             = props.getProperty("config.port");
        CONFIG_DB_NAME          = props.getProperty("config.db_name");
        CREDENTIALS_KEY         = props.getProperty("credentials.key");
        CREDENTIALS_USERNAME    = props.getProperty("credentials.username");
        CREDENTIALS_PASSWORD    = props.getProperty("credentials.password");

        // init aurora storage
        AuroraPluggableStorage.api.aurora_storage_init();

        // init test data
        CONFIG = "{" +
                "   \"read_host\": \"" + CONFIG_READ_HOST + "\"," +
                "   \"write_host\": \"" + CONFIG_WRITE_HOST + "\"," +
                "   \"port\": " + CONFIG_PORT + "," +
                "   \"db_name\": \"" + CONFIG_DB_NAME + "\"" +
                "}";
        CREDENTIALS = "{" +
                "    \"key\": \"" + CREDENTIALS_KEY + "\"," +
                "    \"storage_credentials\": {" +
                "        \"user\": \"" + CREDENTIALS_USERNAME + "\"," +
                "        \"pass\": \"" + CREDENTIALS_PASSWORD + "\"" +
                "    }" +
                "}";
    }
}
