package org.correomqtt.plugin.manager;

import lombok.Getter;
import lombok.RequiredArgsConstructor;

import java.util.List;

@Getter
@RequiredArgsConstructor
public class Task<T> {

    private final String id;
    private final List<T> tasks;

}
