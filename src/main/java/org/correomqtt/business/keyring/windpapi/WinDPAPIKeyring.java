package org.correomqtt.business.keyring.windpapi;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.github.windpapi4j.InitializationFailedException;
import com.github.windpapi4j.WinAPICallFailedException;
import com.github.windpapi4j.WinDPAPI;
import org.correomqtt.plugin.spi.KeyringHook;
import org.correomqtt.business.keyring.BaseKeyring;
import org.correomqtt.business.keyring.KeyringException;
import org.correomqtt.business.fileprovider.SettingsProvider;
import org.pf4j.Extension;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.File;
import java.io.IOException;
import java.nio.charset.Charset;
import java.nio.charset.StandardCharsets;
import java.util.Base64;
import java.util.HashMap;
import java.util.Map;
import java.util.ResourceBundle;

@Extension
public class WinDPAPIKeyring extends BaseKeyring implements KeyringHook {

    private static final Logger LOGGER = LoggerFactory.getLogger(WinDPAPIKeyring.class);

    private static final Charset STD_CHAR_SET = StandardCharsets.UTF_8;
    public static final String FAILED_TO_UNPROTECT_DATA_WITH_WIN_DPAPI = "Failed to unprotect data with WinDPAPI.";
    public static final String FAILED_TO_PARSE_DATA_FROM_WIN_DPAPI = "Failed to parse data from WinDPAPI.";

    private ResourceBundle resources = ResourceBundle.getBundle("org.correomqtt.i18n", SettingsProvider.getInstance().getSettings().getCurrentLocale());

    @Override
    public boolean requiresUserinput() {
        return false;
    }

    @Override
    public String getName() {
        return resources.getString("windpapiName");
    }

    @Override
    public String getDescription() {
        return resources.getString("windpapiDescription");
    }

    @Override
    public String getPassword(String label) {
        Map<String, String> data = readData();
        return data.get(label);
    }

    @Override
    public void setPassword(String label, String password) {
        Map<String, String> data = readData();
        data.put(label, password);
        writeData(data);
    }

    private void writeData(Map<String, String> data) {
        try {
            File file = getFile();
            if(LOGGER.isDebugEnabled()) {
                LOGGER.debug("Write encoded string {}", Base64.getEncoder().encodeToString(protect(data)));
            }
            new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(file,
                    WinDPAPIKeyringDTO
                            .builder()
                            .data(Base64.getEncoder().encodeToString(protect(data)))
                            .build());
        } catch (JsonProcessingException e) {
            throw new KeyringException("Failed to write json data for WinDPAPI.", e);
        } catch (IOException e) {
            throw new KeyringException("Failed to write file with data from WinDPAPI.", e);
        }
    }

    private byte[] protect(Map<String, String> data) {
        try {
            String unprotectedData = new ObjectMapper().writerWithDefaultPrettyPrinter().writeValueAsString(data);
            WinDPAPI winDPAPI = WinDPAPI.newInstance(WinDPAPI.CryptProtectFlag.CRYPTPROTECT_UI_FORBIDDEN);
            return winDPAPI.protectData(unprotectedData.getBytes(STD_CHAR_SET));
        } catch (InitializationFailedException | WinAPICallFailedException e) {
            throw new KeyringException(FAILED_TO_UNPROTECT_DATA_WITH_WIN_DPAPI, e);
        } catch (JsonProcessingException e) {
            throw new KeyringException(FAILED_TO_PARSE_DATA_FROM_WIN_DPAPI, e);
        }

    }

    private Map<String, String> readData() {
        File file = getFile();
        if (file.exists()) {
            try {
                WinDPAPIKeyringDTO winDPAPIKeyringDTO = new ObjectMapper().readValue(file, WinDPAPIKeyringDTO.class);
                byte[] data = Base64.getDecoder().decode(winDPAPIKeyringDTO.getData().getBytes(STD_CHAR_SET));
                return unprotect(data);
            } catch (IOException e) {
                throw new KeyringException("Reading WinDPAPI file failed.", e);
            }
        } else {
            return new HashMap<>();
        }
    }

    private Map<String, String> unprotect(byte[] protectedData) {
        try {
            WinDPAPI winDPAPI = WinDPAPI.newInstance(WinDPAPI.CryptProtectFlag.CRYPTPROTECT_UI_FORBIDDEN);
            String unprotectedData = new String(winDPAPI.unprotectData(protectedData),STD_CHAR_SET);
            return new ObjectMapper().readValue(unprotectedData, new TypeReference<HashMap<String, String>>() {
            });
        } catch (InitializationFailedException | WinAPICallFailedException e) {
            throw new KeyringException(FAILED_TO_UNPROTECT_DATA_WITH_WIN_DPAPI, e);
        } catch (JsonProcessingException e) {
            throw new KeyringException(FAILED_TO_PARSE_DATA_FROM_WIN_DPAPI, e);
        }
    }

    private File getFile() {
        String windpapiPath = SettingsProvider.getInstance().getTargetDirectoryPath() + File.separator + "windpapi.json";
        return new File(windpapiPath);
    }

    @Override
    public boolean isSupported() {
        return WinDPAPI.isPlatformSupported();
    }

    @Override
    public String getIdentifier() {
        return "WinDPAPI";
    }

}
