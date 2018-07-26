package mysql_integration_tests.tests.performance;


import mysql_integration_tests.main.Action;
import mysql_integration_tests.main.PerfThreadRunnable;
import mysql_integration_tests.main.PopulateDatabaseRunnable;
import mysql_integration_tests.main.db.DBQueries;
import mysql_integration_tests.tests.BaseTest;
import org.testng.annotations.AfterMethod;

import java.sql.SQLException;
import java.time.Duration;
import java.time.Instant;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;


public class BasePerfTest extends BaseTest {

    private static String walletName;


    protected void populateDatabase ( int dbThreadsCnt, int totalWalletCnt, int recordsPerWalletCnt, String customTagsPerRecordData,int percentOfCustomTagsPerRecord) throws InterruptedException {
        //logger.debug("Start populating DB...");
        List<Thread> threads = new ArrayList<>();
        Thread thread;

        for (int threadNum = 1; threadNum <= dbThreadsCnt; threadNum++) {
            thread = new Thread(
                    new PopulateDatabaseRunnable(getDefaultConfig("") , getDefaultCredentials(), dbThreadsCnt, threadNum, totalWalletCnt, recordsPerWalletCnt, customTagsPerRecordData, percentOfCustomTagsPerRecord));
            thread.start();
            threads.add(thread);
        }
        for (Thread t : threads) {
            t.join();
        }
        //logger.debug("Finished with populating DB");
    }


    public void sendRequests (int threadsCnt, int totalWalletCnt, int recordsPerWalletCnt, String customTagsPerRecordData, Action action) throws InterruptedException {
        List<List<Duration>> allExecutionTimes = new ArrayList<>();
        PerfThreadRunnable runnable = null;
        Instant start = Instant.now();
        Thread thread;
        List<Thread> threads = new ArrayList<>();
        for (int thread_num = 1; thread_num < threadsCnt + 1; thread_num++) {
            runnable = new PerfThreadRunnable(getDefaultConfig(""), getDefaultCredentials(), threadsCnt, thread_num, totalWalletCnt, recordsPerWalletCnt, customTagsPerRecordData, action);
            thread = new Thread(runnable);
            thread.start();
            threads.add(thread);
        }
        for (Thread t : threads) {
            t.join();
            allExecutionTimes.add(runnable.getExecutionTimes());
        }
        Instant finish = Instant.now();
        Duration totalExecutionTime = (Duration.between(start, finish));
        if (totalExecutionTime.isZero()) {
            totalExecutionTime.plus(Duration.ofSeconds(1));
        }
        if (totalWalletCnt == 0){
            totalWalletCnt = 1;
        }
        if (recordsPerWalletCnt == 0){
            recordsPerWalletCnt = 1;
        }
        Duration maxExecutionTime = Duration.ofSeconds(0);
        Duration sumExecutionTime = Duration.ofSeconds(0);

        for (List<Duration> time: allExecutionTimes) {
            List<Duration> temp = new ArrayList<>();
            for (Duration t: time) {
                sumExecutionTime = sumExecutionTime.plus(t);
            }
            temp.add(Collections.max(time));
            maxExecutionTime = Collections.max(temp);
        }

        System.out.println("=============================== SUMMARY ===============================\n" +
                "Max Execution Time: \t" +maxExecutionTime+ "\n" +
                "Operations executed: \t" +totalWalletCnt * recordsPerWalletCnt+ "\n" +
                "Sum of Execution Times: \t" +sumExecutionTime+ "\n" +
                "Total Duration: \t" + totalExecutionTime+ "\n" +
                "Aprox TPS: \t" + (totalWalletCnt * recordsPerWalletCnt) / totalExecutionTime.toMillis()/1000
        );
    }

    @AfterMethod
    public void cleanup() throws SQLException {
        DBQueries.deleteAll();
    }
}
