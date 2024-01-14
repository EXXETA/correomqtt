package org.correomqtt.business.log;

import ch.qos.logback.core.Appender;
import ch.qos.logback.core.UnsynchronizedAppenderBase;
import ch.qos.logback.core.spi.AppenderAttachable;
import ch.qos.logback.core.spi.AppenderAttachableImpl;

import java.util.Iterator;


// Idea from: https://tersesystems.com/blog/2019/05/27/application-logging-in-java-part-5/
public class CompositeAppender<E> extends UnsynchronizedAppenderBase<E> implements AppenderAttachable<E> {

    protected AppenderAttachableImpl<E> impl = new AppenderAttachableImpl<>();

    @Override
    public void addAppender(Appender<E> appender) {
        impl.addAppender(appender);
    }

    @Override
    public Iterator<Appender<E>> iteratorForAppenders() {
        return impl.iteratorForAppenders();
    }

    @Override
    public Appender<E> getAppender(String s) {
        return impl.getAppender(s);
    }

    @Override
    public boolean isAttached(Appender<E> appender) {
        return impl.isAttached(appender);
    }

    @Override
    public void detachAndStopAllAppenders() {
        impl.detachAndStopAllAppenders();
    }

    @Override
    public boolean detachAppender(Appender<E> appender) {
        return impl.detachAppender(appender);
    }

    @Override
    public boolean detachAppender(String s) {
        return impl.detachAppender(s);
    }

    @Override
    protected void append(E e) {
        impl.appendLoopOnAppenders(e);
    }
}
