package aurora_integration_tests.tests;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.testng.Assert;
import org.testng.annotations.BeforeClass;
import org.testng.annotations.Test;

public class WalletLongevityTest extends BaseTest {

    private static Logger logger = LoggerFactory.getLogger(WalletLongevityTest.class);

    /** Test parameters **/
    private static int numberOfThreads = 20;
    private static int numberOfWallets = numberOfThreads * 1000;
    private static int numOfIterrationsForStatusMessage = 500;


    /** Test data **/
    private static int numOfWalletsPerThread;
    private static int[] walletsStatuses;
    private static Thread[] workers;

    @BeforeClass (alwaysRun = true)
    public void prepareTestParameters() {
        // TODO: pick-up number of threads and number of wallets from config file if provided

        // set test data
        Assert.assertTrue(numberOfWallets % numberOfThreads == 0, "Number of wallets must be dividable by number of threads.");
        numOfWalletsPerThread = numberOfWallets / numberOfThreads;

        walletsStatuses = new int[numberOfWallets]; // zeroes are by default
        workers = new Thread[numberOfThreads];
    }

    @Test
    public void walletLongevityTest() {

        logger.info("Test parameters: " + printTestParameters());

        // prepare threads
        for(int i=0; i<workers.length; i++) {
            // 0-9999, 1000-1999, 2000-2999 ...
            int minID = i*numOfWalletsPerThread;
            int maxID = (i+1)*numOfWalletsPerThread-1;
            workers[i] = new Thread(new WalletWorker(minID, maxID));
            workers[i].setName("Wallets_" + minID + "-" + maxID);
        }

        // start threads
        for(Thread w : workers) {
            w.start();
        }

        InfoWorker infoWorker = new InfoWorker();
        Thread infoWorkerThread = new Thread(infoWorker);
        infoWorkerThread.setName("InfoWorker");
        infoWorkerThread.start();

        // wait for all threads to finish
        while(true) {
            boolean someRunning = false;
            for(Thread t : workers) {
                if(t.isAlive()) {
                    someRunning = true;
                    break;
                }
            }

            if(someRunning) {
                try{Thread.sleep(5000);}catch(Exception e){};
                continue;
            } else {
                infoWorker.setStopRunning(true);
                break;
            }
        }
    }

    private String printTestParameters() {
        return "numberOfThreads: " + numberOfThreads + ", numberOfWallets: " + numberOfWallets;


    }


    private class WalletWorker implements Runnable {

        int minID;
        int maxID;
        boolean stopRunning = false;

        public WalletWorker(int minWalletID, int maxWalletID) {
            minID = minWalletID;
            maxID = maxWalletID;
        }

        @Override
        public void run() {

            logger.debug("starting ...");
            try{Thread.sleep(2000);}catch(Exception e){};

            long statusMessageIterrationCounter = 0;

            //while(!stopRunning) {
            for(int i=0; i<1100; i++) {
                statusMessageIterrationCounter++;

                // get random wallet ID
                int walletID = minID + (int)(Math.random()*(maxID-minID+1));
                if(walletsStatuses[walletID] == 0) {
                    // create wallet
                    walletsStatuses[walletID] = 1;
                }

                logger.debug("picked wallet with ID: " + walletID);

                // open wallet
                walletsStatuses[walletID] = 2;

                // some dummy actions
                try{Thread.sleep(500);}catch(Exception e){};


                // close wallet
                walletsStatuses[walletID] = 1;

                if(statusMessageIterrationCounter % numOfIterrationsForStatusMessage == 0) {
                    logger.info("status msg #" + statusMessageIterrationCounter + " I'm alive.");
                }
            }

            logger.debug("finishing ...");
            try{Thread.sleep(2000);}catch(Exception e){};
        }
    }

    private class InfoWorker implements Runnable {

        boolean stopRunning = false;

        public void setStopRunning(boolean stopRunning) {
            this.stopRunning = stopRunning;
        }

        @Override
        public void run() {
            while(!stopRunning) {
                // go through wallet statuses and calculate statistics
                int notCreated = 0;
                int opened = 0;
                int closed = 0;

                for (int status : walletsStatuses) {
                    switch (status) {
                        case 0:
                            notCreated++;
                            break;
                        case 1:
                            closed++;
                            break;
                        case 2:
                            opened++;
                            break;
                    }
                }

                logger.info("Wallet stats: notCreated: " + notCreated + ", "
                        + "opened: " + opened + ", "
                        + "closed: " + closed + ", "
                        + "total: " + (notCreated+opened+closed));

                try{Thread.sleep(5000);}catch(Exception e){};

            }
        }
    }
}
