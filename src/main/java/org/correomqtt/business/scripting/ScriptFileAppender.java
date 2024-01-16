package org.correomqtt.business.scripting;

import ch.qos.logback.classic.spi.ILoggingEvent;
import ch.qos.logback.core.FileAppender;
import ch.qos.logback.core.recovery.ResilientFileOutputStream;

import java.io.IOException;
import java.nio.channels.FileChannel;
import java.nio.channels.FileLock;
import java.nio.charset.StandardCharsets;

public class ScriptFileAppender extends FileAppender<ILoggingEvent> {

    public void writePlainString(String msg) {
        safeWriteBytes(msg.getBytes(StandardCharsets.UTF_8));
    }

    // Copied private methods from FileAppender from here

    private void safeWriteBytes(byte[] byteArray) {
        ResilientFileOutputStream resilientFOS = (ResilientFileOutputStream) getOutputStream();
        FileChannel fileChannel = resilientFOS.getChannel();
        if (fileChannel == null) {
            return;
        }

        // Clear any current interrupt (see LOGBACK-875)
        boolean interrupted = Thread.interrupted();

        FileLock fileLock = null;
        try {
            fileLock = fileChannel.lock();
            long position = fileChannel.position();
            long size = fileChannel.size();
            if (size != position) {
                fileChannel.position(size);
            }
            writeByteArrayToOutputStreamWithPossibleFlush(byteArray);
        } catch (IOException e) {
            // Mainly to catch FileLockInterruptionExceptions (see LOGBACK-875)
            resilientFOS.postIOFailure(e);
        } finally {
            releaseFileLock(fileLock);

            // Re-interrupt if we started in an interrupted state (see LOGBACK-875)
            if (interrupted) {
                Thread.currentThread().interrupt();
            }
        }
    }

    private void releaseFileLock(FileLock fileLock) {
        if (fileLock != null && fileLock.isValid()) {
            try {
                fileLock.release();
            } catch (IOException e) {
                addError("failed to release lock", e);
            }
        }
    }

}
