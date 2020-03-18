package org.correomqtt.business.dispatcher;

public class LogDispatcher extends BaseDispatcher<LogObserver> {

    private static LogDispatcher instance;

    private LogDispatcher() {
        // private constructor
    }

    public static synchronized LogDispatcher getInstance() {
        if (instance == null) {
            instance = new LogDispatcher();
        }
        return instance;
    }

    void log(String logMsg) {
        observer.forEach(o -> o.updateLog(logMsg));
    }
}