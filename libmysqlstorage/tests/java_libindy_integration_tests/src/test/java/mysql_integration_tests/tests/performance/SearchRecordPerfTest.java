package mysql_integration_tests.tests.performance;

import mysql_integration_tests.main.Action;
import org.testng.annotations.BeforeClass;
import org.testng.annotations.Test;

public class SearchRecordPerfTest extends BasePerfTest{

    private static final int DB_THREADS_CNT = 10;
    private static final int THREADS_CNT = 5;
    private static final int TOTAL_WALLET_CNT = 10;
    private static final int RECORDS_PER_WALLET_CNT = 10;
    private static final String customTagsPerRecordDataOld = "{\"name\": \"John\", \"surname\": \"Doe\", \"country\": \"Serbia\"}";
    private static final String customTagsPerRecordDataNew = "{\"$in\": [\"John\", \"john\"]}, \"age\": {\"$in\": [\"Serbia\", \"serbia\"]}";
    private static final int PERCENT_OF_CUSTOM_TAGS_PER_RECORD = 20;

    @BeforeClass()
    public void prepareDB() throws Exception {
        populateDatabase(DB_THREADS_CNT, TOTAL_WALLET_CNT, RECORDS_PER_WALLET_CNT, customTagsPerRecordDataOld, PERCENT_OF_CUSTOM_TAGS_PER_RECORD);
    }

    @Test()
    public void searchRecordTagsPerfTest() throws InterruptedException {
        sendRequests(THREADS_CNT, TOTAL_WALLET_CNT, 0, customTagsPerRecordDataNew, Action.SearchRecords);
    }
}
