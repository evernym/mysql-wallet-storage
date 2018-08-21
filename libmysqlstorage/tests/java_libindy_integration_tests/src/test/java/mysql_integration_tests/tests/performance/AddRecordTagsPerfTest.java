package mysql_integration_tests.tests.performance;

import mysql_integration_tests.main.Action;
import org.testng.annotations.BeforeClass;
import org.testng.annotations.Test;

public class AddRecordTagsPerfTest extends BasePerfTest {

    // DB_THREADS_CNT - number of threads used for db population
    // should be less or equal to max db connections supported on DB server
    // TOTAL_WALLET_CNT % DB_THREADS_CNT should be 0
    private static final int DB_THREADS_CNT = 25;
    private static final int THREADS_CNT = 1;
    private static final int TOTAL_WALLET_CNT = 10000;
    private static final int RECORDS_PER_WALLET_CNT = 10;
    private static final String CUSTOM_TAGS_PER_RECORD_DATA_PREPARE = "{\"tag1\": \"value1\", \"tag2\": \"value2\", \"tag3\": \"value3\"}";
    private static final String CUSTOM_TAGS_PER_RECORD_DATA_REQUEST = "{\"tag1\": \"newValue1\", \"name\": \"John\", \"surname\": \"Doe\"}";
    private static final int PERCENT_OF_CUSTOM_TAGS_PER_RECORD = 100;

    @BeforeClass()
    public void prepareDB() throws Exception {
        populateDatabase(DB_THREADS_CNT, TOTAL_WALLET_CNT, RECORDS_PER_WALLET_CNT, CUSTOM_TAGS_PER_RECORD_DATA_PREPARE, PERCENT_OF_CUSTOM_TAGS_PER_RECORD);
    }

    @Test()
    public void addRecordTagsPerfTest() throws InterruptedException {
        sendRequests(THREADS_CNT, TOTAL_WALLET_CNT, RECORDS_PER_WALLET_CNT, CUSTOM_TAGS_PER_RECORD_DATA_REQUEST, Action.AddRecordTags);
    }
}
