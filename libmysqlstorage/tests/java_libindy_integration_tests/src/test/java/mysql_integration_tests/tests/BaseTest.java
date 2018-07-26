package mysql_integration_tests.tests;

import mysql_integration_tests.main.MySQLPluggableStorage;
import mysql_integration_tests.main.db.DBConnection;
import mysql_integration_tests.main.db.DBQueries;
import org.hyperledger.indy.sdk.IndyException;
import org.hyperledger.indy.sdk.non_secrets.WalletRecord;
import org.hyperledger.indy.sdk.wallet.Wallet;
import org.testng.Assert;
import org.testng.annotations.AfterSuite;
import org.testng.annotations.BeforeClass;
import org.testng.annotations.BeforeSuite;

import java.io.*;
import java.nio.file.Paths;
import java.sql.SQLException;
import java.util.Properties;
import java.util.concurrent.ExecutionException;

public class BaseTest {

    private final String defaultConfigPropertiesFile = "resources/test.properties";
    Properties props = new Properties();
    protected static DBConnection dbConn;

    protected static String WALLET_TYPE = "mysql";

    protected static final String POOL = "Pool1";
    protected static final String ITEM_TYPE = "TestType";
    protected static final String ITEM_TYPE2 = "TestType2";
    protected static String RECORD_ID = "RecordId";
    protected static String RECORD_VALUE = "RecordValue";

    protected static final String QUERY_EMPTY = "{}";
    protected static final String TAGS_EMPTY = "{}";
    protected static final String TAGS =  "{\"tagName1\":\"str1\",\"tagName2\":\"5\",\"~tagName3\":\"12\"}";
    protected static final String TAGS2 =  "{\"tagName1\":\"str2\",\"tagName2\":\"6\",\"~tagName3\":\"13\"}";
    protected static final String TAGS3 =  "{\"tagName1\":\"str3\",\"tagName2\":\"7\",\"~tagName3\":\"14\"}";

    protected static final String GET_OPTIONS_EMPTY = "{}";
    protected static final String GET_OPTIONS_ALL = "{\"retrieveTags\": true, \"retrieveValue\": true, \"retrieveType\": true}";
    protected static final String GET_OPTIONS_TAGS_ONLY = "{\"retrieveTags\": true, \"retrieveValue\": false, \"retrieveType\": false}";
    protected static final String GET_OPTIONS_VALUE_ONLY = "{\"retrieveTags\": false, \"retrieveValue\": true, \"retrieveType\": false}";
    protected static final String GET_OPTIONS_TYPE_ONLY = "{\"retrieveTags\": false, \"retrieveValue\": false, \"retrieveType\": true}";

    protected static final String SEARCH_OPTIONS_EMPTY = "{}";
    protected static final String SEARCH_OPTIONS_ALL_RETRIEVE_FALSE = "{\"retrieveTags\": false, \"retrieveValue\": false, \"retrieveType\": false, " +
            "\"retrieveTotalCount\": false, \"retrieveRecords\": false}";
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


    protected static String CONFIG_READ_HOST, CONFIG_WRITE_HOST, CONFIG_PORT, CONFIG_DB_NAME;
    protected static String CREDENTIALS_KEY, CREDENTIALS_USERNAME,CREDENTIALS_PASSWORD;


    protected static File TMP_FOLDER = new File("data/tmp");
    protected static File EXPORT_WALLET_FILE = getFileInTempFolder("exportWallet.wallet");
    protected static String EXPORT_WALLET_CONFIG_JSON = "{" +
            "\"key\": \"some_key\"," +
            "\"path\": \"" + EXPORT_WALLET_FILE.getAbsolutePath() + "\"" +
            "}";


    @BeforeSuite(alwaysRun = true)
    public void init() throws IOException, ClassNotFoundException, InstantiationException, IllegalAccessException, SQLException {

        // load properties
        props.load(new FileInputStream(defaultConfigPropertiesFile));

        // init config vars
        CONFIG_READ_HOST        = props.getProperty("config.read_host");
        CONFIG_WRITE_HOST       = props.getProperty("config.write_host");
        CONFIG_PORT             = props.getProperty("config.port");
        CONFIG_DB_NAME          = props.getProperty("config.db_name");

        CREDENTIALS_KEY         = props.getProperty("credentials.key");
        CREDENTIALS_USERNAME    = props.getProperty("credentials.username");
        CREDENTIALS_PASSWORD    = props.getProperty("credentials.password");

        dbConn = new DBConnection("jdbc:mysql://"
                + CONFIG_WRITE_HOST
                + ":"
                + CONFIG_PORT
                + "/"
                + CONFIG_DB_NAME
                + "?useSSL=false",
                CREDENTIALS_USERNAME,
                CREDENTIALS_PASSWORD);

        // Verify connection is working
        dbConn.execute("SELECT 1");
        DBQueries.setDBConnection(dbConn);

        // init mysql storage
        MySQLPluggableStorage.api.mysql_storage_init();

        // Create tmp dir and clean contents of it
        if(!TMP_FOLDER.exists()) TMP_FOLDER.mkdirs();
    }

    @BeforeClass(alwaysRun = true)
    public void cleanFoldersInUse() {
        // clean Tmp folder
        cleanTmpDir();

        // clean indy_client folder
        cleanIndyClientWalletsFolder();
    }

    protected static String getDefaultConfig(String walletName) {
        return getConfig(
                walletName,
                WALLET_TYPE,
                CONFIG_READ_HOST,
                CONFIG_WRITE_HOST,
                CONFIG_PORT,
                CONFIG_DB_NAME
                );
    }

    protected static String getConfig(String walletName, String walletType, String readHost, String writeHost, String port, String dbName) {
        return "{" +
                "    \"id\": \"" + walletName + "\"," +
                "    \"storage_type\": \"" + walletType + "\"," +
                "    \"storage_config\": {" +
                "       \"read_host\": \"" + readHost + "\"," +
                "       \"write_host\": \"" + writeHost + "\"," +
                "       \"port\": " + port + "," +
                "       \"db_name\": \"" + dbName + "\"" +
                "   }" +
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

    protected void prepareRecordsForSearch(Wallet wallet) throws IndyException, ExecutionException, InterruptedException {

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


            WalletRecord.add(wallet, type, "Search"+ RECORD_ID +i, RECORD_VALUE, tags).get();
        }
    }

    protected static File getFileInTempFolder(String fileName) {
        return new File(TMP_FOLDER.getAbsolutePath() + "/" + fileName);
    }

    private static void cleanTmpDir() {
        deleteFolder(TMP_FOLDER);
    }

    protected static void cleanIndyClientWalletsFolder() {
        File folder = Paths.get(System.getProperty("user.home"),
                ".indy_client",
                "wallet").toFile();
        deleteFolder(folder);
    }

    protected static void deleteFolder(File folder) {
        File[] files = folder.listFiles();
        if(files!=null) { //some JVMs return null for empty dirs
            for(File f: files) {
                if(f.isDirectory()) {
                    deleteFolder(f);
                    f.delete();
                } else {
                    f.delete();
                }
            }
        }
    }

    @AfterSuite(alwaysRun = true)
    public void cleanup () throws SQLException {
        this.dbConn.close();
    }
}
