package aurora_integration_tests.tests;
import aurora_integration_tests.main.PrepareDatabaseRunnable;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.testng.annotations.Test;

import java.util.ArrayList;
import java.util.List;


public class PerformanceTests extends BaseTest{

public static int DB_THREADS_COUNT = 10;
public static int TOTAL_WALLET_CNT = 10;
public static int RECORDS_PER_WALLET_CNT = 10;

private static Logger logger = LoggerFactory.getLogger(PerformanceTests.class);

public void populate_database(int TOTAL_WALLET_CNT, int RECORDS_PER_WALLET_CNT, String custom_tags_per_record_data, int percent_of_custom_tags_per_record) throws InterruptedException {
    logger.debug("Start populating DB...");
    List<Thread> threads = new ArrayList<>();
    Thread thread;

    for (int thread_num =1; thread_num<=DB_THREADS_COUNT; thread_num ++){
         thread = new Thread(
                new PrepareDatabaseRunnable(POOL, WALLET_TYPE, CONFIG, CREDENTIALS, DB_THREADS_COUNT, thread_num, TOTAL_WALLET_CNT, RECORDS_PER_WALLET_CNT, custom_tags_per_record_data, percent_of_custom_tags_per_record));
        thread.start();
        threads.add(thread);
    }
    for (Thread result: threads) {
        result.join();
    }
    logger.debug("Finished with populating DB");
}


@Test(priority = 0)
    public void test() throws Exception{
    populate_database(TOTAL_WALLET_CNT, RECORDS_PER_WALLET_CNT, "{\"name\": \"John\", \"surname\": \"Doe\", \"country\": \"Serbia\"}", 20);
}

}


