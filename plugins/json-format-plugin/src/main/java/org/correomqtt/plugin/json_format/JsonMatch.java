package org.correomqtt.plugin.json_format;

import lombok.AllArgsConstructor;
import lombok.Data;

@Data
@AllArgsConstructor
public class JsonMatch {
    String type;
    int start;
    int end;
}
