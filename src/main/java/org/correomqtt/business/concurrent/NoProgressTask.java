package org.correomqtt.business.concurrent;

import org.correomqtt.business.eventbus.EventBus;
import org.correomqtt.business.utils.FrontendBinding;

import java.util.HashSet;
import java.util.Set;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CompletionException;

public abstract class NoProgressTask<T, E> extends Task<T, Void, E> {


}
