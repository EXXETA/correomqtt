package org.correomqtt;

import org.correomqtt.core.eventbus.Event;

public record PreloadingProgressEvent(String msg) implements Event {
}
