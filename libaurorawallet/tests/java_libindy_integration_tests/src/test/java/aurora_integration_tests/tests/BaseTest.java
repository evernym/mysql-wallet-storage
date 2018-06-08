package aurora_integration_tests.tests;

import aurora_integration_tests.main.AuroraPluggableStorage;
import org.json.JSONObject;
import org.testng.annotations.BeforeSuite;

import java.io.*;
import java.util.HashMap;
import java.util.Properties;

public class BaseTest {

    private final String defaultConfigPropertiesFile = "resources/test.properties";
    Properties props = new Properties();

    protected String WALLET_TYPE = "aurora";

    protected static final String POOL = "Pool1";
    protected static final String ITEM_TYPE = "TestType";
    protected static final String ITEM_TYPE2 = "TestType2";

    protected static final String QUERY_EMPTY = "{}";
    protected static final String TAGS_EMPTY = "{}";
    protected static final String TAGS =  "{\"tagName1\":\"str1\",\"tagName2\":\"5\",\"tagName3\":\"12\"}";
    protected static final String TAGS2 =  "{\"tagName1\":\"str2\",\"tagName2\":\"6\",\"tagName3\":\"13\"}";
    protected static final String TAGS3 =  "{\"tagName1\":\"str3\",\"tagName2\":\"7\",\"tagName3\":\"14\"}";

    protected static final String GET_OPTIONS_EMPTY = "{}";
    protected static final String GET_OPTIONS_ALL = "{\"retrieveTags\": true, \"retrieveValue\": true, \"retrieveType\": true}";
    protected static final String GET_OPTIONS_TAGS_ONLY = "{\"retrieveTags\": true, \"retrieveValue\": false, \"retrieveType\": false}";
    protected static final String GET_OPTIONS_VALUE_ONLY = "{\"retrieveTags\": false, \"retrieveValue\": true, \"retrieveType\": false}";
    protected static final String GET_OPTIONS_TYPE_ONLY = "{\"retrieveTags\": false, \"retrieveValue\": false, \"retrieveType\": true}";

    protected static final String SEARCH_OPTIONS_EMPTY = "{}";
    protected static final String SEARCH_OPTIONS_ALL = "{\"retrieveTags\": true, \"retrieveValue\": true, \"retrieveType\": true, " +
                                                            "\"retrieveTotalCount\": true, \"retrieveRecords\": true}";
    protected static final String SEARCH_OPTIONS_TAGS_ONLY = "{\"retrieveTags\": true, \"retrieveValue\": false, \"retrieveType\": false, " +
                                                            "\"retrieveTotalCount\": false, \"retrieveRecords\": true}";
    protected static final String SEARCH_OPTIONS_VALUE_ONLY = "{\"retrieveTags\": false, \"retrieveValue\": true, \"retrieveType\": false, " +
                                                            "\"retrieveTotalCount\": false, \"retrieveRecords\": true}";
    protected static final String SEARCH_OPTIONS_TYPE_ONLY = "{\"retrieveTags\": false, \"retrieveValue\": false, \"retrieveType\": true, " +
                                                            "\"retrieveTotalCount\": false, \"retrieveRecords\": true}";
    protected static final String SEARCH_OPTIONS_TOTAL_COUNT_ONLY = "{\"retrieveTags\": false, \"retrieveValue\": false, \"retrieveType\": false, " +
                                                            "\"retrieveTotalCount\": true, \"retrieveRecords\": false}";
    protected static final String SEARCH_OPTIONS_RECORDS_ONLY = "{\"retrieveTags\": false, \"retrieveValue\": false, \"retrieveType\": false, " +
                                                            "\"retrieveTotalCount\": false, \"retrieveRecords\": true}";

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
        CONFIG_READ_HOST        = props.getProperty("config.read_host");
        CONFIG_WRITE_HOST       = props.getProperty("config.write_host");
        CONFIG_PORT             = props.getProperty("config.port");
        CONFIG_DB_NAME          = props.getProperty("config.db_name");

        CREDENTIALS_KEY         = props.getProperty("credentials.key");
        CREDENTIALS_USERNAME    = props.getProperty("credentials.username");
        CREDENTIALS_PASSWORD    = props.getProperty("credentials.password");

        // init aurora storage
        AuroraPluggableStorage.api.aurora_storage_init();

        // init test data
        CONFIG = getDefaultConfig();
        CREDENTIALS = getDefaultCredentials();
    }

    protected static String getDefaultConfig() {
        return getConfig(
                CONFIG_READ_HOST,
                CONFIG_WRITE_HOST,
                CONFIG_PORT,
                CONFIG_DB_NAME
                );
    }

    protected static String getConfig(String readHost, String writeHost, String port, String dbName) {
        return "{" +
                "   \"read_host\": \"" + readHost + "\"," +
                "   \"write_host\": \"" + writeHost + "\"," +
                "   \"port\": " + port + "," +
                "   \"db_name\": \"" + dbName + "\"" +
                "}";
    }

    protected static String getDefaultCredentials() {
        return getCredentials(
                CREDENTIALS_KEY,
                CREDENTIALS_USERNAME,
                CREDENTIALS_PASSWORD
        );
    }

    protected static String getCredentials(String key, String username, String password) {
        return "{" +
                "    \"key\": \"" + key + "\"," +
                "    \"storage_credentials\": {" +
                "        \"user\": \"" + username + "\"," +
                "        \"pass\": \"" + password + "\"" +
                "    }" +
                "}";
    }
}
