package org.correomqtt.di;

import io.github.classgraph.ClassGraph;
import io.github.classgraph.ClassInfo;
import io.github.classgraph.ScanResult;
import lombok.extern.slf4j.Slf4j;

import java.lang.ref.WeakReference;
import java.lang.reflect.Constructor;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.ParameterizedType;
import java.lang.reflect.Type;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.stream.Collectors;

@Slf4j
public class SoyDi {

    private static final Map<Class<?>, BeanInfo> BEAN_INFO = new HashMap<>();
    private static final Map<Class<?>, WeakReference<?>> SINGLETON_INSTANCES = new HashMap<>();

    private SoyDi() {
    }

    public static synchronized void scan(String pkg) {
        log.info("SOY: Scanning {} for soy beans.", pkg);
        long startTime = System.currentTimeMillis();
        int count = 0;
        int singletonCount = 0;
        try (ScanResult scanResult =
                     new ClassGraph()
                             .verbose(log.isTraceEnabled())
                             .disableJarScanning()
                             .disableNestedJarScanning()
                             .disableRuntimeInvisibleAnnotations()
                             .enableClassInfo()
                             .enableMethodInfo()
                             .enableAnnotationInfo()
                             .acceptPackages(pkg)
                             .scan()) {
            for (ClassInfo classInfo : scanResult.getClassesWithAnnotation(Bean.class)) {
                BeanInfo beanInfo = scanClass(classInfo);
                if (beanInfo != null) {
                    if (beanInfo.isSingleton()) {
                        singletonCount++;
                    }
                    count++;
                }
            }
        } catch (Exception e) {
            log.error("error", e);
        }
        if (log.isInfoEnabled()) {
            long endTime = System.currentTimeMillis();
            log.info("SOY: Found {} soy beans, containing {} singletons.", count, singletonCount);
            log.info("SOY: Scan of {} finished in {}ms.", pkg, endTime - startTime);
        }
    }

    private static BeanInfo scanClass(ClassInfo classInfo) {
        if (classInfo.isAnnotation()) {
            return null;
        }
        try {
            Class<?> clazz = Class.forName(classInfo.getName());
            Constructor<?> constructor = findConstructor(clazz);
            List<ParameterInfo> parameters = findConstructorParameter(constructor);
            boolean singleton = clazz.getAnnotation(SingletonBean.class) != null;
            BeanInfo beanInfo = new BeanInfo(clazz, constructor, parameters, singleton);
            BEAN_INFO.put(clazz, beanInfo);
            return beanInfo;
        } catch (Exception e) {
            throw new SoyDiException("Exception while scanning " + classInfo.getName() + " ", e);
        }
    }

    private static Constructor<?> findConstructor(Class<?> clazz) {
        List<Constructor<?>> constructors = Arrays.stream(clazz.getDeclaredConstructors()).toList();
        if (constructors.size() > 1) {
            List<Constructor<?>> annotatedConstructors = constructors.stream()
                    .filter(c -> c.getAnnotation(Inject.class) != null)
                    .toList();
            if (annotatedConstructors.isEmpty()) {
                throw new SoyDiException("If more than one constructor exists. Exactly one must be annotated with @Inject: " + clazz);
            } else if (annotatedConstructors.size() > 1) {
                throw new SoyDiException("Only one constructor can be annotated with the @Inject annotation: " + clazz);
            }
        }
        return constructors.get(0);
    }

    private static List<ParameterInfo> findConstructorParameter(Constructor<?> constructor) {
        return Arrays.stream(constructor.getParameters())
                .map(p -> new ParameterInfo(
                        p.getName(),
                        p.getType(),
                        p.getAnnotation(Assisted.class) != null))
                .toList();
    }

    public static <T> boolean isInjectable(Class<T> clazz) {
        return BEAN_INFO.containsKey(clazz);
    }

    @SuppressWarnings({"java:S3011", "unchecked"})
    // DI must ignore accessibility in order to allow accessibility for users.
    private static synchronized <T> T inject(Class<T> clazz, List<Class<?>> chain) {
        BeanInfo beanInfo = BEAN_INFO.get(clazz);
        if (chain.contains(clazz)) {
            throw new SoyDiException("Detected dependency cycle: " + getChainLogMsg(chain));
        }
        if (beanInfo == null) {
            throw new SoyDiException("Can not inject " + clazz + ", cause it was not scanned. " + getChainLogMsg(chain));
        }
        if (beanInfo.isSingleton() && SINGLETON_INSTANCES.containsKey(clazz)) {
            return (T) SINGLETON_INSTANCES.get(clazz).get();
        }
        Object[] params = beanInfo.getConstructorParameters().stream()
                .map(cp -> {
                    ArrayList<Class<?>> newChain = new ArrayList<>(chain);
                    newChain.add(clazz);
                    return inject(cp.getType(), newChain);
                })
                .toArray();
        try {
            beanInfo.getConstructor().setAccessible(true);
            T instance = (T) beanInfo.getConstructor().newInstance(params);
            if (beanInfo.isSingleton()) {
                SINGLETON_INSTANCES.put(clazz, new WeakReference<>(instance));
            }
            return instance;
        } catch (InstantiationException | IllegalAccessException | InvocationTargetException e) {
            throw new SoyDiException("Can not inject " + clazz + ", caused by an exception. " + getChainLogMsg(chain), e);
        }
    }

    private static String getChainLogMsg(List<Class<?>> chain) {
        return chain.stream()
                .map(Class::toString)
                .collect(Collectors.joining(" -> "));
    }

    public static synchronized <T> T inject(Class<T> clazz) {
        return inject(clazz, new ArrayList<>());
    }

    @SuppressWarnings("unchecked")
    public static <T> T inject(TypeReference<T> reference) {
        Type type = reference.getType();
        Class<T> rawType = (Class<T>) getRawType(type);
        return inject(rawType, new ArrayList<>());
    }

    private static Class<?> getRawType(Type type) {
        if (type instanceof Class) {
            return (Class<?>) type;
        } else if (type instanceof ParameterizedType) {
            return (Class<?>) ((ParameterizedType) type).getRawType();
        } else {
            throw new SoyDiException("Unable to cast TypeReference: " + type);
        }
    }
}
