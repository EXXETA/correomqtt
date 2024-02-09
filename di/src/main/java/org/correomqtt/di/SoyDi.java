package org.correomqtt.di;

import io.github.classgraph.ClassGraph;
import io.github.classgraph.ClassInfo;
import io.github.classgraph.ScanResult;
import lombok.extern.slf4j.Slf4j;

import java.lang.reflect.Constructor;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.lang.reflect.ParameterizedType;
import java.lang.reflect.Type;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Set;
import java.util.stream.Collectors;

@Slf4j
public class SoyDi {

    private static final Map<Class<?>, BeanInfo> BEAN_INFO = new HashMap<>();
    private static final Map<Class<?>, Object> SINGLETON_INSTANCES = new HashMap<>();
    private static final Set<ClassLoader> CLASS_LOADER = new HashSet<>();
    private static boolean initialized = false;

    private SoyDi() {
    }

    private static synchronized void init() {
        if (initialized)
            return;
        CLASS_LOADER.add(SoyDi.class.getClassLoader());
        initialized = true;
    }

    public static void addClassLoader(ClassLoader classLoader) {
        init();
        CLASS_LOADER.add(classLoader);
    }

    public static synchronized void scan(String pkg) {
        init();
        log.info("SOY: Scanning {} for soy beans.", pkg);
        long startTime = System.currentTimeMillis();
        int count = 0;
        int singletonCount = 0;
        ClassGraph classGraph = new ClassGraph()
                .verbose(log.isInfoEnabled())
                .enableClassInfo()
                .enableMethodInfo()
                .enableAnnotationInfo()
                .acceptPackages(pkg);
        CLASS_LOADER.forEach(classGraph::addClassLoader);
        try (ScanResult scanResult = classGraph.scan()) {
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
            Class<?> clazz = loadClass(classInfo.getName());
            Constructor<?> constructor = findConstructor(clazz);
            List<ParameterInfo> parameters = findConstructorParameter(constructor);
            boolean singleton = clazz.getAnnotation(SingletonBean.class) != null;
            BeanInfo beanInfo = new BeanInfo(clazz, constructor, parameters, singleton);
            BEAN_INFO.put(clazz, beanInfo);
            EventBus.registerClass(clazz);
            return beanInfo;
        } catch (Exception e) {
            throw new SoyDiException("Exception while scanning " + classInfo.getName() + " ", e);
        }
    }

    private static Class<?> loadClass(String className) {
        return CLASS_LOADER.stream()
                .map(cl -> {
                    try {
                        return Class.forName(className, false, cl);
                    } catch (ClassNotFoundException e) {
                        return null;
                    }
                })
                .filter(Objects::nonNull)
                .findFirst()
                .orElseThrow(() -> new SoyDiException("Unable to load class via available classloaders: " + className));
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

    @SuppressWarnings("unchecked")
    private static synchronized <T> T inject(Class<T> clazz, List<Class<?>> chain) {
        BeanInfo beanInfo = BEAN_INFO.get(clazz);
        if (chain.contains(clazz)) {
            throw new SoyDiException("Detected dependency cycle: " + getChainLogMsg(chain));
        }
        if (beanInfo == null) {
            throw new SoyDiException("Can not inject " + clazz + ", cause it was not scanned. " + getChainLogMsg(chain));
        }
        if (beanInfo.isSingleton() && SINGLETON_INSTANCES.containsKey(clazz)) {
            return (T) SINGLETON_INSTANCES.get(clazz);
        }
        try {
            T instance;
            if (Factory.class.isAssignableFrom(clazz) || clazz.getName().startsWith("org.correomqtt.di")) {
                Object[] params = beanInfo.getConstructorParameters().stream()
                        .map(cp -> {
                            ArrayList<Class<?>> newChain = new ArrayList<>(chain);
                            newChain.add(clazz);
                            return inject(cp.getType(), newChain);
                        })
                        .toArray();
                instance = (T) beanInfo.getConstructor().newInstance(params);
            } else {
                Class<?> factoryClass = loadClass(clazz.getName() + "Factory");
                Object factory = inject(factoryClass);
                Method factoryMethod = factoryClass.getMethod("create");
                instance = (T) factoryMethod.invoke(factory);
            }
            if (beanInfo.isSingleton()) {
                SINGLETON_INSTANCES.put(clazz, instance);
            }
            return instance;
        } catch (IllegalAccessException | InvocationTargetException | NoSuchMethodException |
                 InstantiationException e) {
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
