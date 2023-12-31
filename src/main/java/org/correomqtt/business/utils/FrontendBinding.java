package org.correomqtt.business.utils;

import io.github.classgraph.ClassGraph;
import io.github.classgraph.ClassInfo;
import io.github.classgraph.ScanResult;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.lang.reflect.InvocationTargetException;

public class FrontendBinding {


    private static final Logger LOGGER = LoggerFactory.getLogger(FrontendBinding.class);

    private static TaskToFrontendPush executor;

    public static void pushToFrontend(Runnable runnable) {
        if (executor == null) {
            searchExecutor();
        }
        executor.pushToFrontend(runnable);
    }

    private static void searchExecutor() {
        String pkg = "org.correomqtt";
        try (ScanResult scanResult =
                     new ClassGraph()
                             .enableClassInfo()
                             .acceptPackages(pkg)
                             .scan()) {
            for (ClassInfo routeClassInfo : scanResult.getClassesImplementing(TaskToFrontendPush.class)) {
                if (executor != null) {
                    LOGGER.warn("Multiple implementations of {} found in {}.", TaskToFrontendPush.class, pkg);
                    return;
                }
                Class<TaskToFrontendPush> clazz = routeClassInfo.loadClass(TaskToFrontendPush.class);
                executor = clazz.getDeclaredConstructor().newInstance();
            }

            if (executor == null) {
                LOGGER.error("No implementation for {} found in {}. Please provide one. " +
                                "Otherwise background tasks won't work. Bye!",
                        TaskToFrontendPush.class, pkg);
                System.exit(1);
            }
        } catch (InvocationTargetException | InstantiationException | IllegalAccessException |
                 NoSuchMethodException e) {
            LOGGER.warn("Problem searching for {} in {}.", TaskToFrontendPush.class, pkg, e);
        }
    }

}
