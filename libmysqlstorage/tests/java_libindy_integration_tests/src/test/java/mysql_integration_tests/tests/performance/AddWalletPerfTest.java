package mysql_integration_tests.tests.performance;

import mysql_integration_tests.main.Action;
import org.testng.annotations.Test;

public class AddWalletPerfTest extends BasePerfTest{

    // DB_THREADS_CNT - number of threads used for db population
    // should be less or equal to max db connections supported on DB server
    // TOTAL_WALLET_CNT % DB_THREADS_CNT should be 0
    private static final int DB_THREADS_CNT = 25;
    private static final int THREADS_CNT = 1;
    private static final int TOTAL_WALLET_CNT = 10000;
    private static final int RECORDS_PER_WALLET_CNT = 10;
    private static final String customTagsPerRecordData = "";

    @Test()
    public void addWalletPerfTest() throws InterruptedException {
        sendRequests(THREADS_CNT, TOTAL_WALLET_CNT, RECORDS_PER_WALLET_CNT, customTagsPerRecordData, Action.AddWallet);
    }


}
