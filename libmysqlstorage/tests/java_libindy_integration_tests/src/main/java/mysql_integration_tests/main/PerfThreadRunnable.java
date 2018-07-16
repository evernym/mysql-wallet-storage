package mysql_integration_tests.main;

import mysql_integration_tests.main.utils.Utils;
import org.hyperledger.indy.sdk.non_secrets.WalletRecord;
import org.hyperledger.indy.sdk.non_secrets.WalletSearch;
import org.hyperledger.indy.sdk.wallet.Wallet;

import org.testng.Assert;

import java.time.Duration;
import java.time.Instant;
import java.util.HashMap;
import java.util.List;

import static org.hyperledger.indy.sdk.wallet.Wallet.createWallet;

public class PerfThreadRunnable implements Runnable{

    private static final String ITEM_TYPE = "TestType";
    private static final String GET_OPTIONS_ALL = "{\"retrieveTags\": true, \"retrieveValue\": true, \"retrieveType\": true}";
    private static final String SEARCH_OPTIONS_ALL =  "{\"retrieveTags\": true, \"retrieveValue\": true, \"retrieveType\": true, " + "\"retrieveTotalCount\": true, \"retrieveRecords\": true}";

    private String config;
    private String creds;
    private int threadsCnt;
    private int threadNum;
    private int totalWalletCnt;
    private int recordsPerWalletCnt;
    private String customTagsPerRecordData;
    private Action action;

    private volatile List<Duration> executionTimes;

    public PerfThreadRunnable(String config, String creds, int dbThreadsCnt, int threadNum, int totalWalletCnt, int recordsPerWalletCnt, String customTagsPerRecordData, Action action){
        this.config = config;
        this.creds = creds;
        this.threadsCnt = dbThreadsCnt;
        this.threadNum = threadNum;
        this.totalWalletCnt = totalWalletCnt;
        this.recordsPerWalletCnt = recordsPerWalletCnt;
        this.customTagsPerRecordData = customTagsPerRecordData;
        this.action = action;
    }

    public List<Duration> getExecutionTimes() {
        return this.executionTimes;
    }

    @Override
    public void run() {
        for (int walletNum = (threadNum -1) * (totalWalletCnt/ threadsCnt)+1; walletNum<threadNum*(totalWalletCnt/ threadsCnt)+1; walletNum++) {
            String walletName = "wallet_name" + walletNum;
            config = config.replace("\"id\": \"\"", "\"id\": \"" + walletName + "\"");
            Wallet wallet = null;

            if (action != Action.AddWallet || action != Action.DeleteWallet || action !=Action.OpenAndCloseWallet){
                try {
                    wallet = Wallet.openWallet(config, creds).get();
                } catch (Exception e) {
                    e.printStackTrace();
                }
            }
            Instant timeBeforeRequest = Instant.now();
            try {
                executeAction(walletNum, recordsPerWalletCnt, customTagsPerRecordData, action, wallet);

            } catch (Exception e) {
                e.printStackTrace();
            }
            Instant timeAfterRequest = Instant.now();
            Duration timeDiff = Duration.between(timeBeforeRequest, timeAfterRequest);
            executionTimes.add(timeDiff);
        }

    }


    public void executeAction(int walletNum, int recordsPerWalletCnt, String customTagsPerRecordData, Action action, Wallet wallet) throws Exception {
        String recordId;
        switch (action) {
            case AddWallet: {
                String walletName = "wallet_name_{}" + walletNum;
                config = config.replace("\"id\": \"\"", "\"id\": \"" + walletName + "\"");
                createWallet(config, creds).get();
            }
            case OpenAndCloseWallet: {
                String walletName = "wallet_name_{}" + walletNum;
                config = config.replace("\"id\": \"\"", "\"id\": \"" + walletName + "\"");
                wallet = Wallet.openWallet(config, creds).get();
                wallet.closeWallet();
            }
            case DeleteWallet: {
                String walletName = "wallet_name_{}" + walletNum;
                Wallet.deleteWallet(walletName, creds).get();

            }
            case AddRecord: {
                String recordValue = Utils.generateRandomRecordValue();
                for (int i = 1; i <= recordsPerWalletCnt; i++) {
                    recordId = "record_id_" + walletNum + "_" + i;
                    HashMap<String, String> tagsList = new HashMap<>();
                    if (customTagsPerRecordData != "") {
                        tagsList = Utils.getHashMapFromJsonString(customTagsPerRecordData);
                    }
                    String tags = Utils.getJsonStringFromHashMap(tagsList);
                    WalletRecord.add(wallet, ITEM_TYPE, recordId, recordValue, tags);
                }
            }
            case GetRecord: {
                for (int i = 1; i <= recordsPerWalletCnt; i++) {
                    recordId = "record_id_" + walletNum + "_" + i;
                    String recordJson = WalletRecord.get(wallet, ITEM_TYPE, recordId, GET_OPTIONS_ALL).get();
                    Assert.assertNotEquals(recordJson, "", "Get record api for record: " + recordId + " returned empty string");
                }
            }
            case DeleteRecord: {
                for (int i = 1; i <= recordsPerWalletCnt; i++) {
                    recordId = "record_id_" + walletNum + "_" + i;
                    WalletRecord.delete(wallet, ITEM_TYPE, recordId);
                }
            }
            case UpdatRecordValue: {
                String newRecordValue = Utils.generateRandomRecordValue();
                for (int i = 1; i <= recordsPerWalletCnt; i++) {
                    recordId = "record_id_" + walletNum + "_" + i;
                    WalletRecord.updateValue(wallet, ITEM_TYPE, recordId, newRecordValue);
                }
            }
            case AddRecordTags: {
                for (int i = 1; i <= recordsPerWalletCnt; i++) {
                    recordId = "record_id_" + walletNum + "_" + i;
                    WalletRecord.addTags(wallet, ITEM_TYPE, recordId, customTagsPerRecordData);
                }
            }
            case UpdateRecordTags: {
                for (int i = 1; i <= recordsPerWalletCnt; i++) {
                    recordId = "record_id_" + walletNum + "_" + i;
                    WalletRecord.updateTags(wallet, ITEM_TYPE, recordId, customTagsPerRecordData);
                }
            }
            case DeleteRecordTags: {
                for (int i = 1; i <= recordsPerWalletCnt; i++) {
                    recordId = "record_id_" + walletNum + "_" + i;
                    WalletRecord.deleteTags(wallet, ITEM_TYPE, recordId, customTagsPerRecordData);
                }
            }
            case SearchRecords: {
                WalletSearch.open(wallet, ITEM_TYPE, customTagsPerRecordData, SEARCH_OPTIONS_ALL).get();
            }

        }
    }

}

