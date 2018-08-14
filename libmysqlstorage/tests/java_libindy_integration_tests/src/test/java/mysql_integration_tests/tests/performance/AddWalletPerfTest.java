package mysql_integration_tests.tests.performance;

import mysql_integration_tests.main.Action;
import org.testng.annotations.Test;

public class AddWalletPerfTest extends BasePerfTest{

    private static final int DB_THREADS_CNT = 10;
    private static final int THREADS_CNT = 5;
    private static final int TOTAL_WALLET_CNT = 10;
    private static final int RECORDS_PER_WALLET_CNT = 0;
    private static final String customTagsPerRecordData = "";
    private static final int PERCENT_OF_CUSTOM_TAGS_PER_RECORD = 0;

    @Test()
    public void addWalletPerfTest() throws InterruptedException {
        sendRequests(THREADS_CNT, TOTAL_WALLET_CNT, RECORDS_PER_WALLET_CNT, "", Action.AddWallet);
    }


}
