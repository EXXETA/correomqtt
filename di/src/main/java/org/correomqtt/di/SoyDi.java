package org.correomqtt.di;

import io.github.classgraph.ClassGraph;
import io.github.classgraph.ClassInfo;
import io.github.classgraph.ScanResult;
import javassist.ClassPool;
import javassist.CtClass;
import javassist.CtConstructor;
import javassist.NotFoundException;
import javassist.bytecode.MethodInfo;
import lombok.extern.slf4j.Slf4j;

import java.lang.reflect.Constructor;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.ParameterizedType;
import java.lang.reflect.Type;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.stream.Collectors;

@Slf4j
public class SoyDi {

    private static final Map<Class<?>, BeanInfo> BEAN_INFO = new HashMap<>();
    private static final Map<Class<?>, Object> SINGLETON_INSTANCES = new HashMap<>();

    private SoyDi() {
    }

    public static synchronized void scan(String pkg) {
        log.info("Scanning {} for soy beans.", pkg);
        int count = 0;
        try (ScanResult scanResult =
                     new ClassGraph()
                             .verbose(log.isTraceEnabled())
                             .enableAllInfo()
                             .acceptPackages(pkg)
                             .scan()) {
            for (ClassInfo classInfo : scanResult.getClassesWithAnnotation(Bean.class)) {
                if (classInfo.isAnnotation()) {
                    continue;
                }
                log.info(classInfo.getName());
                try {
                    Class<?> clazz = Class.forName(classInfo.getName());
                    Constructor<?> constructor = findConstructor(clazz);
                    List<ParameterInfo> parameters = findConstructorParameter(constructor);
                    boolean singleton = clazz.getAnnotation(SingletonBean.class) != null;
                    BEAN_INFO.put(clazz, new BeanInfo(clazz, constructor, parameters, singleton));
                } catch (Exception e) {
                    throw new SoyDiException("Exception while scanning " + classInfo.getName() + " ", e);
                }
                //TODO step 2: automatic observes merken @Observes(autocreate = true)
                count++;
            }
        } catch (Exception e) {
            log.error("error", e);
        }
        log.info("Scanning {} for soy beans finished. Found {} soy beans.", pkg, count);
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

    private static synchronized <T> T inject(Class<T> clazz, List<Class<?>> chain) {
        if (clazz.isAssignableFrom(Lazy.class)) {

            Type[] genericInterfaces = clazz.getGenericInterfaces();
            for (Type genericInterface : genericInterfaces) {
                if (genericInterface instanceof ParameterizedType) {
                    ParameterizedType parameterizedType = (ParameterizedType) genericInterface;
                    Type[] genericTypes = parameterizedType.getActualTypeArguments();
                    for (Type type : genericTypes) {
                        System.out.println(type.getTypeName());
                    try {
                        return (T) getLazyFactory(Class.forName(type.getTypeName()));
                    } catch (ClassNotFoundException e) {
                        throw new RuntimeException(e);
                    }
                    }
                }
            }


            log.info("Lazy not found");
        }
        BeanInfo beanInfo = BEAN_INFO.get(clazz);
        ClassPool pool = ClassPool.getDefault();
        CtClass cc = null;
        try {
            cc = pool.get(clazz.getName());
        } catch (NotFoundException e) {
            throw new RuntimeException(e);
        }
        CtConstructor[] constructors = cc.getDeclaredConstructors();
        AtomicInteger lineNumber = new AtomicInteger();
        for (CtConstructor constructor : constructors) {
            MethodInfo methodInfo = constructor.getMethodInfo();
            lineNumber.set(methodInfo.getLineNumber(0));
        } //TODO find the correct constructor
        if (chain.contains(clazz)) {
            throw new SoyDiException("Detected dependency cycle: " + chain.stream().map(c -> c + ":" + lineNumber.get()).collect(Collectors.joining(" -> ")));
        }
        if (beanInfo == null) {
            throw new SoyDiException("Can not inject " + clazz + ", cause it was not scanned. " + chain.stream().map(c -> c + ":" + lineNumber.get()).collect(Collectors.joining(" -> ")));
        }
        if (beanInfo.isSingleton() && SINGLETON_INSTANCES.containsKey(clazz)) {
            return (T) SINGLETON_INSTANCES.get(clazz); // TODO weak reference
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
                SINGLETON_INSTANCES.put(clazz, instance);
            }
            return instance;
        } catch (InstantiationException | IllegalAccessException | InvocationTargetException e) {
            throw new SoyDiException("Can not inject " + clazz + ", caused by an exception. " + chain.stream().map(c -> c + ":" + lineNumber.get()).collect(Collectors.joining(" -> ")) + " ", e);
        }
    }

    public static synchronized <T> T inject(Class<T> clazz) {
        return inject(clazz, new ArrayList<>());
        // zyklen erkennen!
    }

    public static <T> T inject(TypeReference<T> reference) {
        // dependencies anhand der parameter holen
        // class erstellen
        return null;
    }

    public static <T> Lazy<T> getLazyFactory(Class<T> clazz) {
        return new Lazy<>(clazz);
    }

    public static <T> Lazy<T> getLazyFactory(TypeReference<T> reference) {
        return new Lazy<>(reference);
    }
}
