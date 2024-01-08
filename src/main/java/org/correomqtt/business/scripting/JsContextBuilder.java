package org.correomqtt.business.scripting;

import org.correomqtt.business.scripting.binding.ClientConnect;
import org.graalvm.polyglot.Context;
import org.graalvm.polyglot.Engine;
import org.graalvm.polyglot.Value;

import java.io.PipedOutputStream;

public class JsContextBuilder {


    private Context context;
    private PipedOutputStream out;
    private ExecutionContextDTO dto;

    JsContextBuilder() {

    }

    JsContextBuilder out(PipedOutputStream out) {
        this.out = out;
        return this;
    }

    JsContextBuilder dto(ExecutionContextDTO dto) {
        this.dto = dto;
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
                .out(out)
                .allowAllAccess(true)
                .build();
    }

    private void bindContext() {
        if(!context.getEngine().getLanguages().containsKey("js")){
            throw new IllegalArgumentException("Javascript is not installed. If you use GraalVM try \"gu install js\"");
        }
        Value binding = context.getBindings("js");
        binding.putMember("correo", new CorreoJsBinding(dto));
        binding.putMember("connect", new ClientConnect(dto.getExecutionDTO().getConnectionId()));
    }

}
