package org.correomqtt.di;

import lombok.AllArgsConstructor;
import lombok.Getter;

@Getter
@AllArgsConstructor
public class ParameterInfo {

    private String name;
    private Class<?> type;
    private boolean assisted;
}
