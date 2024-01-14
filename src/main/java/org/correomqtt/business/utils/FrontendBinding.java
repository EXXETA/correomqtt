package org.correomqtt.business.utils;

import io.github.classgraph.ClassGraph;
import io.github.classgraph.ScanResult;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.lang.reflect.InvocationTargetException;
import java.util.Objects;

public class FrontendBinding {


    private static final Logger LOGGER = LoggerFactory.getLogger(FrontendBinding.class);

    private static TaskToFrontendPush executor;

    private FrontendBinding() {
        // private constructor
    }

    public static void pushToFrontend(Runnable runnable) {
        if (executor == null) {
            searchExecutor();
        }
        executor.pushToFrontend(runnable);
    }

    private static void searchExecutor() {
        String pkg = "org.correomqtt";
        try (ScanResult scanResult = new ClassGraph().enableClassInfo().acceptPackages(pkg).scan()) {
            executor = scanResult.getClassesImplementing(TaskToFrontendPush.class)
                    .stream()
                    .map(ci -> {
                        try {
                            return ci.loadClass(TaskToFrontendPush.class).getDeclaredConstructor().newInstance();
                        } catch (InstantiationException | IllegalAccessException | InvocationTargetException |
                                 NoSuchMethodException ex) {
                            LOGGER.warn("Problem searching for {} in {}.", TaskToFrontendPush.class, pkg, ex);
                            return null;
                        }
                    })
                    .filter(Objects::nonNull)
                    .findFirst()
                    .orElseGet(() -> {
                        LOGGER.error("No implementation for {} found in {}. Please provide one. " +
                                        "Otherwise background tasks won't work. Bye!",
                                TaskToFrontendPush.class, pkg);
                        System.exit(1);
                        return null;
                    });
        }
    }
}
