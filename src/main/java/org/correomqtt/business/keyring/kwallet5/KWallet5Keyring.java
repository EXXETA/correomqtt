package org.correomqtt.business.keyring.kwallet5;

import com.sun.jna.Platform;
import org.correomqtt.business.keyring.KeyringException;
import org.correomqtt.plugin.spi.KeyringHook;
import org.pf4j.Extension;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.nio.charset.Charset;
import java.nio.charset.StandardCharsets;
import java.util.Base64;

@Extension
public class KWallet5Keyring implements KeyringHook {

    private static final Logger LOGGER = LoggerFactory.getLogger(KWallet5Keyring.class);

    private static final String APP_NAME = "CorreoMQTT";
    private static final String WALLET_NAME = "kdewallet";
    private static final String QDBUS_BASE_CMD = "qdbus org.kde.kwalletd5 /modules/kwalletd5";

    private static final Charset STD_CHAR_SET = StandardCharsets.UTF_8;

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
        return Platform.isLinux() && isEnabled();
    }

    @Override
    public String getIdentifier() {
        return "KWallet5";
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
        } catch (InterruptedException | IOException e) {
            LOGGER.error("Cannot check KWallet", e);
            throw new KeyringException("Cannot check KWallet", e);
        }
    }

    private int openWallet() {
        try {
            // method int org.kde.KWallet.open(QString wallet, qlonglong wId, QString appid)
            String result = runQDBusCommand("open", WALLET_NAME, "0", APP_NAME);
            return Integer.parseInt(result);
        } catch (InterruptedException | IOException e) {
            LOGGER.error("Cannot open KWallet", e);
            throw new KeyringException("Cannot open KWallet", e);
        }
    }

    private void writePassword(int handler, String key, String password) {
        try {
            // method int org.kde.KWallet.writePassword(int handle, QString folder, QString key, QString value, QString appid)
            String result = runQDBusCommand("writePassword", String.valueOf(handler), APP_NAME, key, password, APP_NAME);
            if (Integer.parseInt(result) < 0) {
                LOGGER.error("Cannot store password in KWallet.");
                throw new KeyringException("Cannot store password in KWallet.");
            }
        } catch (InterruptedException | IOException e) {
            LOGGER.error("Cannot store password in KWallet", e);
            throw new KeyringException("Cannot store password in KWallet", e);
        }
    }


    private void removeEntry(int handler, String key) {
        try {
            // method int org.kde.KWallet.removeEntry(int handle, QString folder, QString key, QString appid)
            runQDBusCommand("removeEntry", String.valueOf(handler), APP_NAME, key, APP_NAME);
        } catch (InterruptedException | IOException e) {
            LOGGER.error("Cannot remove password from KWallet", e);
            throw new KeyringException("Cannot remove password from KWallet", e);
        }
    }

    private String readPassword(int handler, String key) {
        try {
            // method QString org.kde.KWallet.readPassword( int handle, QString folder, QString key, QString appid)
            return runQDBusCommand("readPassword", String.valueOf(handler), APP_NAME, key, APP_NAME);
        } catch (InterruptedException | IOException e) {
            LOGGER.error("Cannot get password from KWallet", e);
            throw new KeyringException("Cannot get password from KWallet", e);
        }
    }

    private boolean isEnabled() {
        try {
            return runQDBusCommand("isEnabled").equals("true");
        } catch (InterruptedException | IOException e) {
            return false;
        }
    }

    private String runQDBusCommand(String... parameter) throws IOException, InterruptedException {
        return runShellCommand(QDBUS_BASE_CMD + " " + String.join(" ", parameter));
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
