package com.exxeta.correomqtt.plugin.manager;

import lombok.Getter;
import lombok.RequiredArgsConstructor;

import java.util.List;

@Getter
@RequiredArgsConstructor
public class ProtocolTask {

    private final String id;
    private final List<ProtocolExtension> tasks;

}
