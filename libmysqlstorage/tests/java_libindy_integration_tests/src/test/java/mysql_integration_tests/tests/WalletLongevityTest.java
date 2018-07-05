package mysql_integration_tests.tests;

import org.hyperledger.indy.sdk.IndyException;
import org.hyperledger.indy.sdk.non_secrets.WalletRecord;
import org.hyperledger.indy.sdk.non_secrets.WalletSearch;
import org.hyperledger.indy.sdk.wallet.Wallet;
import org.hyperledger.indy.sdk.wallet.WalletNotFoundException;
import org.json.JSONObject;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.testng.Assert;
import org.testng.annotations.*;

import java.util.Arrays;
import java.util.concurrent.ExecutionException;

public class WalletLongevityTest extends BaseTest {

    /** Test parameters **/
    private static Logger logger = LoggerFactory.getLogger(WalletLongevityTest.class);
    private static int numOfWalletsPerThread, printStatusFrequency, numberOfWallets;
    private static String walletNamePrefix;
    private static int maxNumOfKeyPerWallet;

    /** Test data **/
    private static int[] walletsStatuses;
    private static Thread[] threads;
    private static WalletWorker[] walletWorkers;
    private static long testDurationInMillis;

    @Parameters({ "testDurationInMillis", "testDurationInHours", "numberOfWallets", "numberOfThreads", "walletNamePrefix", "printStatusFrequency", "maxNumOfKeyPerWallet"})
    @BeforeClass (alwaysRun = true)
    public void prepareTestParameters(@Optional("") String testDurationInMillisParameter,
                                      long testDurationInHours,
                                      int numOfWallets,
                                      int numberOfThreads,
                                      String namePrefix,
                                      int numOfIterrationsForStatus,
                                      int maxNumOfKeys) throws Exception {

        // if optional param is empty, use duration in hours
        if(testDurationInMillisParameter.isEmpty()) {
            testDurationInMillis = testDurationInHours * 3600 * 1000;
        } else {
            testDurationInMillis = Long.parseLong(testDurationInMillisParameter);
        }

        printStatusFrequency = numOfIterrationsForStatus;
        walletNamePrefix = namePrefix;
        numberOfWallets = numOfWallets;
        maxNumOfKeyPerWallet = maxNumOfKeys;

        // TODO: replace deletion with direct DB delete query + wallet folders deletion
        // delete existing wallets (if any)
        for(int i=0; i<numberOfWallets; i++) {
            try {
                Wallet.deleteWallet(walletNamePrefix+i, CREDENTIALS).get();
            } catch (Exception e) {
                if (!(e.getCause() instanceof WalletNotFoundException))
                    throw e; // not expected exception -> re-throw
            }
        }

        // set test data
        Assert.assertTrue(numberOfWallets % numberOfThreads == 0, "Number of wallets must be dividable by number of threads.");
        numOfWalletsPerThread = numberOfWallets / numberOfThreads;

        walletsStatuses = new int[numberOfWallets];
        for(int i=0; i<walletsStatuses.length; i++) walletsStatuses[i] = -1; // initialise with status -1 (not created)

        threads = new Thread[numberOfThreads];
        walletWorkers = new WalletWorker[numberOfThreads];
    }

    @Parameters({"infoLogPeriodInSeconds"})
    @Test
    public void walletLongevityTest(int infoLogPeriodInSeconds) {

        // determine test end timestamp
        long testStartTimestamp = System.currentTimeMillis();
        long testEndTimestamp = System.currentTimeMillis() + testDurationInMillis;

        // prepare threads
        for(int i = 0; i< threads.length; i++) {
            // 0-9999, 1000-1999, 2000-2999 ...
            int minID = i*numOfWalletsPerThread;
            int maxID = (i+1)*numOfWalletsPerThread-1;
            walletWorkers[i] = new WalletWorker(minID, maxID, testEndTimestamp);
            threads[i] = new Thread(walletWorkers[i]);
            threads[i].setName(walletWorkers[i].getName());
        }

        // start threads
        for(Thread t : threads) {
            t.start();
        }

        InfoWorker infoWorker = new InfoWorker(infoLogPeriodInSeconds);
        Thread infoWorkerThread = new Thread(infoWorker);
        infoWorkerThread.setName("InfoWorker");
        infoWorkerThread.start();

        // wait for all threads to finish
        boolean normalExit = false;
        while(!normalExit) {
            for(Thread t : threads) {
                try{
                    t.join();
                } catch(InterruptedException e){
                    // some thread was interrupted -> reiterrate
                    break;
                }
            }

            // all joins finished without exceptions -> this is normal exit
            normalExit = true;
        }

        infoWorker.setStopRunning(true);
        try {
            infoWorkerThread.join();
        } catch (InterruptedException e) {
            logger.warn("InterruptedException when join is called for InfoWorker thread, exception message: " + e.getMessage());
        }

        logger.info("Actual test duration: " + (System.currentTimeMillis() - testStartTimestamp));
        Assert.assertTrue(System.currentTimeMillis() > testEndTimestamp, "Test ended after ");

    }

    @AfterClass (alwaysRun = true)
    public void cleanupAfterClass() {
        // delete existing wallets (if any)
        for(int i=0; i<numberOfWallets; i++) {
            try {
                Wallet.deleteWallet(walletNamePrefix+i, CREDENTIALS).get();
            } catch (Exception e) {}
        }
    }

    private class WalletWorker implements Runnable {

        int minID;
        int maxID;
        long testEndTimestamp;
        long lastSeenOn;
        String name;
        long numOfItterations = 0;

        public WalletWorker(int minWalletID, int maxWalletID, long testEndTimestamp) {
            minID = minWalletID;
            maxID = maxWalletID;
            this.testEndTimestamp = testEndTimestamp;
            this.lastSeenOn = System.currentTimeMillis();
            name = "Wallets_" + minID + "-" + maxID;
        }

        @Override
        public void run() {

            logger.debug("starting ...");

            while(testEndTimestamp > System.currentTimeMillis()) {
                numOfItterations++;


                // check if I'm alive needs to be logged
                if(numOfItterations % printStatusFrequency == 0) {
                    logger.debug("status msg #" + numOfItterations + " I'm alive.");
                }

                // update last seen
                lastSeenOn = System.currentTimeMillis();

                Wallet wallet = null;

                // get random wallet ID and set wallet name
                int walletID = minID + (int)(Math.random()*(maxID-minID+1));
                String walletName = walletNamePrefix+walletID;
                logger.debug("picked wallet: " + walletName);

                if(walletsStatuses[walletID] < 0) {
                    // create wallet
                    try {
                        createWallet(walletName, walletID);
                    } catch (Exception e) {
                        logger.error("Wallet '" + walletName + "' not created, exception message is: " + e.getMessage());
                        continue; // start new iterration
                    }

                }

                // open wallet
                try {
                    wallet = openWallet(walletName);
                } catch(Exception e) {
                    logger.error("Wallet '" + walletName + "' not opened, exception message is: " + e.getMessage());
                    continue; // start new iterration
                }

                /**
                 * If wallet status is:
                 * [0-6]: add key
                 * [7-9]: 75% chance to add key, 25% chance to delete a key
                 * 10: delete a key
                 */
                if (walletsStatuses[walletID] >= 0 && walletsStatuses[walletID] <= 6) {
                    addRecord(wallet, walletID);
                } else if (walletsStatuses[walletID] >= 7 && walletsStatuses[walletID] <= 9) {
                    if (Math.random() < 0.75) {
                        addRecord(wallet, walletID);
                    } else {
                        deleteRecord(wallet, walletID);
                    }
                } else {
                    deleteRecord(wallet, walletID);
                }

                // get record
                getRecord(wallet, walletID);

                // update record
                updateRecordValue(wallet, walletID);

                // search wallet
                searchWallet(wallet, walletID);

                // close wallet
                try {
                    wallet.closeWallet().get();
                } catch(Exception e) {
                    logger.error("Wallet '" + walletName + "' not closed, exception message is: " + e.getMessage());
                    continue; // start new iterration
                }
            }

            logger.debug("finishing ...");
        }

        private void createWallet(String name, int walletID) throws IndyException, ExecutionException, InterruptedException {
            Wallet.createWallet(POOL, name, WALLET_TYPE, CONFIG, CREDENTIALS).get();
            walletsStatuses[walletID] = 0;
        }
        private Wallet openWallet(String walletName) throws IndyException, ExecutionException, InterruptedException {
            return Wallet.openWallet(walletName, null, CREDENTIALS).get();
        }

        private void addRecord(Wallet wallet, int walletID){

            // create ID to be added
            String itemID = RECORD_ID + (walletsStatuses[walletID] + 1);
            logger.trace("Adding a key to wallet with ID'" + walletID + "' of current status '" + walletsStatuses[walletID] + "'");

            try {
                WalletRecord.add(wallet, ITEM_TYPE, itemID, RECORD_VALUE, TAGS).get();
                walletsStatuses[walletID] += 1; // new key added
                logger.trace("Added a key to wallet with ID'" + walletID + "' of current status '" + walletsStatuses[walletID] + "'");
            } catch(Exception e) {
                logger.error("Adding key to wallet with ID '" + walletID + "' failed with message " + e.getMessage());
            }
        }

        private void deleteRecord(Wallet wallet, int walletID){

            // create ID to be deleted
            String itemID = RECORD_ID + (walletsStatuses[walletID]);
            logger.trace("Deleting a key from wallet with ID'" + walletID + "'");

            try {
                WalletRecord.delete(wallet, ITEM_TYPE, itemID).get();
                walletsStatuses[walletID] -= 1; // key deleted
            } catch(Exception e) {
                logger.error("Deleting key to wallet with ID '" + walletID + "' failed with message " + e.getMessage());
            }
        }

        private void getRecord(Wallet wallet, int walletID) {
            int numOfKeysInWallet = walletsStatuses[walletID];

            if(numOfKeysInWallet < 1) {
                logger.warn("getRecord: Wallet with id '" + walletID + "' does not have keys, its status is '" + numOfKeysInWallet + "'");
                return;
            }

            // pick one random key from the wallet: random() => [0,N), then +1 => [1,N]
            int keyID = 1 + (int) Math.random()*numOfKeysInWallet;

            try {
                WalletRecord.get(wallet, ITEM_TYPE, RECORD_ID + keyID, GET_OPTIONS_ALL).get();

            } catch(Exception e) {
                logger.error("Exception when getting a record '" + (RECORD_ID + keyID) + "' from wallet with ID '" + walletID + "', exception message is: " + e.getMessage());
            }
        }

        private void searchWallet(Wallet wallet, int walletID) {

            JSONObject searchRecords;
            try {
                WalletSearch search = WalletSearch.open(wallet, ITEM_TYPE, QUERY_EMPTY, SEARCH_OPTIONS_ALL).get();
                String searchRecordsJson = search.fetchNextRecords(wallet, 20).get();
                searchRecords = new JSONObject(searchRecordsJson);
            } catch (Exception e) {
                logger.error("Exception when getting a record from wallet with ID '" + walletID + "', exception message is: " + e.getMessage());
                return;
            }

            try {
                long expected = walletsStatuses[walletID], actual = searchRecords.getLong("totalCount");
                if(expected != actual) {
                    logger.warn("Actual number of records [" + actual + "] in wallet with ID '" + walletID + "' " +
                            "do not match expected number of records [" + expected + "]");
                }
            } catch (Exception e) {
                logger.warn("Error while parsing search results for wallet with ID'" + walletID + "'");
            }
        }

        private void updateRecordValue(Wallet wallet, int walletID) {
            int numOfKeysInWallet = walletsStatuses[walletID];

            if(numOfKeysInWallet < 1) {
                logger.warn("updateRecord: Wallet with id '" + walletID + "' does not have keys, its status is '" + numOfKeysInWallet + "'");
                return;
            }

            // pick one random key from the wallet: random() => [0,N), then +1 => [1,N]
            int keyID = 1 + (int) Math.random()*numOfKeysInWallet;

            try {
                WalletRecord.updateValue(wallet, ITEM_TYPE, RECORD_ID + keyID, ""+System.currentTimeMillis()).get();
            } catch (Exception e) {
                logger.error("Exception when updating a record '" + (RECORD_ID + keyID) + "' from wallet with ID '" + walletID + "', exception message is: " + e.getMessage());
            }
        }

        public long getLastSeenOn() {
            return lastSeenOn;
        }

        public String getName() {return name;}
        public long getNumOfItterations(){return numOfItterations;}
    }

    private class InfoWorker implements Runnable {

        boolean stopRunning = false;
        int logPeriodInSeconds = 30;
        long lastSeenThresholdInSeconds = 60; // 1 minutes allowed for wallet worker inactivity

        public InfoWorker(int logPeriodInSeconds) {
            this.logPeriodInSeconds = logPeriodInSeconds;
        }

        public void setStopRunning(boolean stopRunning) {
            this.stopRunning = stopRunning;
        }

        @Override
        public void run() {
            while(!stopRunning) {
                // go through wallet statuses and calculate statistics
                int walletTotals[] = new int[maxNumOfKeyPerWallet+1]; //
                int notCreated = 0;
                int numOfIrregular = 0;

                for (int status : walletsStatuses) {
                    if (status == -1) {
                        notCreated++;
                    } else if(status > maxNumOfKeyPerWallet) {
                        // this is not regular
                        numOfIrregular++;
                    } else {
                        walletTotals[status] += 1;
                    }
                }

                logger.info("Wallet stats: notCreated: " + notCreated + ", "
                        + "numOfIrregular: " + numOfIrregular + ", "
                        + "regular: " + Arrays.toString(walletTotals));

                String numOfIterratiosPerThread = "";
                for(WalletWorker w : walletWorkers) {
                    numOfIterratiosPerThread += " ," + w.getNumOfItterations();
                }
                logger.info("Number of itterations per worker: " + numOfIterratiosPerThread.substring(2));

                for (WalletWorker w : walletWorkers) {
                    long diff = System.currentTimeMillis() - w.getLastSeenOn();
                    if(diff >= lastSeenThresholdInSeconds * 1000)
                        logger.warn("Thread '" + w.getName() + "' not seen more than " + lastSeenThresholdInSeconds + " seconds");
                }

                try{Thread.sleep(logPeriodInSeconds * 1000);}catch(Exception e){};

            }
        }
    }
}
