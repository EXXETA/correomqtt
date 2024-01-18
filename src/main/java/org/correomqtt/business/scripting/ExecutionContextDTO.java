package org.correomqtt.business.scripting;

import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Getter;
import lombok.NoArgsConstructor;
import lombok.Setter;
import org.graalvm.polyglot.Context;

import java.io.PipedOutputStream;

@NoArgsConstructor
@AllArgsConstructor
@Getter
@Setter
@Builder
class ExecutionContextDTO {

    private Context context;
    private ExecutionDTO executionDTO;
    private PipedOutputStream out;

}
