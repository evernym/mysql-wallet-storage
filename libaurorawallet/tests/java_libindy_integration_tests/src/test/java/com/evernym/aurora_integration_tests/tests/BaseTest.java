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


    @BeforeSuite(alwaysRun = true)
    public void init() throws IOException {

        // load properties
        File configProperties = new File(defaultConfigPropertiesFile);
        props.load(new FileInputStream(defaultConfigPropertiesFile));

        // init aurora storage
        AuroraPluggableStorage.api.aurora_storage_init();

        // init test data
        CONFIG = "{" +
                "   \"read_host\": \"" + props.getProperty("config.read_host.ip") + "\"," +
                "   \"write_host\": \"" + props.getProperty("config.write_host.ip") + "\"," +
                "   \"port\": " + props.getProperty("config.port") +
                "}";
        CREDENTIALS = "{" +
                "    \"key\": \"" + props.getProperty("credentials.key") + "\"," +
                "    \"storage_credentials\": {" +
                "        \"user\": \"" + props.getProperty("credentials.username") + "\"," +
                "        \"pass\": \"" + props.getProperty("credentials.password") + "\"" +
                "    }" +
                "}";
    }
}
