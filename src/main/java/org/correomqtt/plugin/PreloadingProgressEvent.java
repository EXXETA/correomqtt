package org.correomqtt.plugin;

import org.correomqtt.business.eventbus.Event;

public record PreloadingProgressEvent(String msg) implements Event {
}
