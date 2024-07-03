package org.correomqtt.core.scripting;

import org.correomqtt.di.DefaultBean;
import org.correomqtt.core.scripting.binding.AsyncLatch;
import org.correomqtt.core.scripting.binding.ClientFactory;
import org.correomqtt.core.scripting.binding.Queue;
import org.graalvm.polyglot.Context;
import org.graalvm.polyglot.Value;
import org.slf4j.Logger;
import org.slf4j.Marker;

import org.correomqtt.di.Inject;
import java.io.PipedOutputStream;
import java.util.function.Consumer;

@DefaultBean
public class JsContextBuilder {

    public static final String CORREO_SCRIPT_QUEUE = "queue";
    public static final String CORREO_CONNECTION_ID = "connectionId";
    public static final String CORREO_SCRIPT_LOGGER = "logger";
    public static final String CORREO_SCRIPT_MARKER = "marker";
    public static final String CORREO_ASYNC_LATCH = "latch";
    private final ClientFactory clientFactory;
    private Context context;
    private PipedOutputStream out;
    private ExecutionDTO dto;
    private Logger scriptLogger;
    private Marker marker;

    @Inject
    public JsContextBuilder(ClientFactory clientFactory){
        this.clientFactory = clientFactory;
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

    JsContextBuilder marker(Marker marker) {
        this.marker = marker;
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
        Queue queue = new Queue(scriptLogger);
        AsyncLatch asyncLatch = new AsyncLatch(marker);

        Value polyglotBindings = context.getPolyglotBindings();
        polyglotBindings.putMember(CORREO_CONNECTION_ID, dto.getConnectionId());
        polyglotBindings.putMember(CORREO_SCRIPT_LOGGER, scriptLogger);
        polyglotBindings.putMember(CORREO_SCRIPT_MARKER, marker);
        polyglotBindings.putMember(CORREO_SCRIPT_QUEUE, queue);
        polyglotBindings.putMember(CORREO_ASYNC_LATCH, asyncLatch);

        clientFactory.setContext(context);

        binding.putMember("sleep", (Consumer<Integer>) this::sleepCmd);
        binding.putMember("clientFactory", clientFactory);
        binding.putMember(CORREO_SCRIPT_LOGGER, scriptLogger);
        binding.putMember(CORREO_SCRIPT_MARKER, marker);
        binding.putMember(CORREO_SCRIPT_QUEUE, queue);
        binding.putMember("join", (Runnable) () -> joinAsyncLatch(asyncLatch));

    }

    private void joinAsyncLatch(AsyncLatch asyncLatch) {
        try {
            asyncLatch.join();
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            throw new IllegalStateException(e);
        }
    }

    private void sleepCmd(Integer timeInMillis) {
        try {
            Thread.sleep(timeInMillis);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            throw new IllegalStateException(e);
        }
    }
}
