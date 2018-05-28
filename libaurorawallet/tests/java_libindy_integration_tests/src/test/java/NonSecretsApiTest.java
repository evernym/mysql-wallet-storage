import org.hyperledger.indy.sdk.IndyException;
import org.hyperledger.indy.sdk.wallet.Wallet;
import org.testng.annotations.AfterMethod;
import org.testng.annotations.Test;

import java.util.concurrent.ExecutionException;

public class NonSecretsApiTest extends BaseTest {

    private String walletName = "testWallet" + System.currentTimeMillis();
    private String type = "aurora";

    @Test
    public void tempTest() throws IndyException, ExecutionException, InterruptedException {
        AuroraPluggableStorage.api.aurora_storage_init();

        // create and open wallet
        Wallet.createWallet(POOL, walletName, type, CONFIG, CREDENTIALS).get();
        Wallet wallet = Wallet.openWallet(walletName, null, CREDENTIALS).get();
    }

    @AfterMethod (alwaysRun = true)
    public void afterMethod() throws IndyException, InterruptedException, ExecutionException {
        Wallet.deleteWallet(walletName, CREDENTIALS).get();
    }
}
