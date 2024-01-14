package org.correomqtt.business.scripting;

import org.correomqtt.business.scripting.binding.AsyncLatch;
import org.correomqtt.business.scripting.binding.Client;
import org.correomqtt.business.scripting.binding.ClientFactory;
import org.correomqtt.business.scripting.binding.Queue;
import org.graalvm.polyglot.Context;
import org.graalvm.polyglot.Value;
import org.slf4j.Logger;

import java.io.PipedOutputStream;
import java.util.concurrent.BlockingQueue;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.LinkedBlockingDeque;
import java.util.function.Consumer;

public class JsContextBuilder {


    public static final String CORREO_SCRIPT_QUEUE = "queue";
    public static final String CORREO_CONNECTION_ID = "connectionId";
    public static final String CORREO_SCRIPT_LOGGER = "logger";

    public static final String CORREO_ASYNC_LATCH = "latch";
    private Context context;
    private PipedOutputStream out;
    private ExecutionDTO dto;
    private Logger scriptLogger;

    JsContextBuilder() {

    }

    JsContextBuilder out(PipedOutputStream out) {
        this.out = out;
        return this;
    }

    JsContextBuilder dto(ExecutionDTO dto) {
        this.dto = dto;
        return this;
    }

    JsContextBuilder logger(Logger scriptLogger) {
        this.scriptLogger = scriptLogger;
        return this;
    }

    Context build() {

        if (out == null) {
            throw new IllegalArgumentException("Out is missing.");
        }

        if (dto == null) {
            throw new IllegalArgumentException("DTO is missing.");
        }

        createContext();
        bindContext();

        return context;

    }

    private void createContext() {

        context = Context.newBuilder("js")
                .allowExperimentalOptions(true) // required for top level await
                .out(out)
                .err(out)
                .allowAllAccess(true)
                .option("js.esm-eval-returns-exports", "true")
                .option("js.ecmascript-version", "2023")
                .build();

    }

    private void bindContext() {

        Value binding = context.getBindings("js");

        Queue queue = new Queue();
        AsyncLatch joinLatch = new AsyncLatch();

        Value polyglotBindings = context.getPolyglotBindings();
        polyglotBindings.putMember(CORREO_CONNECTION_ID, dto.getConnectionId());
        polyglotBindings.putMember(CORREO_SCRIPT_LOGGER, scriptLogger);
        polyglotBindings.putMember(CORREO_SCRIPT_QUEUE, queue);
        polyglotBindings.putMember(CORREO_ASYNC_LATCH, joinLatch);

        binding.putMember("sleep", (Consumer<Integer>) t -> {
            try {
                Thread.sleep(t);
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
                throw new RuntimeException(e);
            }
        });
        binding.putMember("CorreoClient", Client.class);
        binding.putMember("ClientFactory", ClientFactory.class);
        binding.putMember(CORREO_SCRIPT_LOGGER, scriptLogger);
        binding.putMember("queue", queue);
        binding.putMember("join", (Runnable) () -> {
            try {
                joinLatch.join();
            } catch (InterruptedException e) {
                throw new RuntimeException(e);
            }
        });

    }

}
