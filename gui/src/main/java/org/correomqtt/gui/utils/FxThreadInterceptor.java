package org.correomqtt.gui.utils;

import javafx.application.Platform;
import org.correomqtt.di.AroundInvoke;
import org.correomqtt.di.InterceptorBean;
import org.correomqtt.di.InvocationContext;

@InterceptorBean(FxThread.class)
public class FxThreadInterceptor{

    @AroundInvoke
    public Object aroundInvoke(InvocationContext ctx) {
        Platform.runLater(ctx::proceed);
        return null;
    }
}
