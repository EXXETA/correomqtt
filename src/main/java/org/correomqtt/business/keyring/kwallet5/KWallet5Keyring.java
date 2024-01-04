package org.correomqtt.business.keyring.kwallet5;

import org.apache.commons.lang3.SystemUtils;
import org.correomqtt.plugin.spi.KeyringHook;
import org.correomqtt.business.keyring.BaseKeyring;
import org.correomqtt.business.keyring.KeyringException;
import org.correomqtt.business.fileprovider.SettingsProvider;
import org.pf4j.Extension;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.nio.charset.Charset;
import java.nio.charset.StandardCharsets;
import java.util.Base64;
import java.util.ResourceBundle;

@Extension
public class KWallet5Keyring extends BaseKeyring implements KeyringHook {

    private static final Logger LOGGER = LoggerFactory.getLogger(KWallet5Keyring.class);

    private static final String APP_NAME = "CorreoMQTT";
    private static final String WALLET_NAME = "kdewallet";
    private static final String QDBUS_BASE_CMD = "qdbus org.kde.kwalletd5 /modules/kwalletd5";

    private static final Charset STD_CHAR_SET = StandardCharsets.UTF_8;

    private final ResourceBundle resources = ResourceBundle.getBundle("org.correomqtt.i18n", SettingsProvider.getInstance().getSettings().getCurrentLocale());

    private Integer kwalletHandler = null;

    @Override
    public String getPassword(String label) {
        return new String(Base64.getDecoder().decode(readPassword(getKWalletHandler(), label).getBytes(STD_CHAR_SET)));
    }

    @Override
    public void setPassword(String label, String password) {
        if (password == null || password.isEmpty()) {
            removeEntry(getKWalletHandler(), label);
        } else {
            writePassword(getKWalletHandler(), label, Base64.getEncoder().encodeToString(password.getBytes(STD_CHAR_SET)));
        }
    }

    @Override
    public boolean isSupported() {
        try {
            return SystemUtils.IS_OS_LINUX && isEnabled();
        }catch(KeyringException e){
            LOGGER.debug("KWallet5 is not supported", e);
            return false;
        }
    }

    @Override
    public String getIdentifier() {
        return "KWallet5";
    }

    @Override
    public String getName() {
        return resources.getString("kwallet5Name");
    }

    @Override
    public String getDescription() {
        return resources.getString("kwallet5Description");
    }

    private synchronized int getKWalletHandler() {

        if (this.kwalletHandler == null || !isOpen(this.kwalletHandler)) {
            this.kwalletHandler = openWallet();
        }
        return this.kwalletHandler;
    }

    private boolean isOpen(Integer kwalletHandler) {
        try {
            // method bool org.kde.KWallet.isOpen(QString wallet)
            return runQDBusCommand("isOpen", String.valueOf(kwalletHandler)).equals("true");
        } catch (IOException e) {
            throw new KeyringException("Cannot check KWallet", e);
        }
    }

    private int openWallet() {
        try {
            // method int org.kde.KWallet.open(QString wallet, qlonglong wId, QString appid)
            String result = runQDBusCommand("open", WALLET_NAME, "0", APP_NAME);
            return Integer.parseInt(result);
        } catch (IOException e) {
            throw new KeyringException("Cannot open KWallet", e);
        }
    }

    private void writePassword(int handler, String key, String password) {
        try {
            // method int org.kde.KWallet.writePassword(int handle, QString folder, QString key, QString value, QString appid)
            String result = runQDBusCommand("writePassword", String.valueOf(handler), APP_NAME, key, password, APP_NAME);
            if (Integer.parseInt(result) < 0) {
                throw new KeyringException("Cannot store password in KWallet.");
            }
        } catch (IOException e) {
            throw new KeyringException("Cannot store password in KWallet", e);
        }
    }


    private void removeEntry(int handler, String key) {
        try {
            // method int org.kde.KWallet.removeEntry(int handle, QString folder, QString key, QString appid)
            runQDBusCommand("removeEntry", String.valueOf(handler), APP_NAME, key, APP_NAME);
        } catch (IOException e) {
            throw new KeyringException("Cannot remove password from KWallet", e);
        }
    }

    private String readPassword(int handler, String key) {
        try {
            // method QString org.kde.KWallet.readPassword( int handle, QString folder, QString key, QString appid)
            return runQDBusCommand("readPassword", String.valueOf(handler), APP_NAME, key, APP_NAME);
        } catch (IOException e) {
            throw new KeyringException("Cannot get password from KWallet", e);
        }
    }

    private boolean isEnabled() {

        try {
            return runQDBusCommand("isEnabled").equals("true");
        } catch (IOException e) {
            LOGGER.warn("QDBus Command failed.", e);
            return false;
        }
    }

    private String runQDBusCommand(String... parameter) throws IOException {
        try {
            return runShellCommand(QDBUS_BASE_CMD + " " + String.join(" ", parameter));
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            throw new IOException(e);
        }
    }

    private String runShellCommand(String cmd) throws IOException, InterruptedException {
        StringBuilder result = new StringBuilder();
        String s;
        Process p;

        p = Runtime.getRuntime().exec(cmd);
        BufferedReader br = new BufferedReader(
                new InputStreamReader(p.getInputStream()));
        while ((s = br.readLine()) != null)
            result.append(s);
        p.waitFor();
        if (p.exitValue() != 0) {
            LOGGER.error("Command failed with exit code {}", p.exitValue());
            throw new KeyringException("Command failed with exit code " + p.exitValue());
        }
        p.destroy();
        return result.toString();
    }
}
