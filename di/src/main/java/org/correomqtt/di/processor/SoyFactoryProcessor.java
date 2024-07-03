package org.correomqtt.di.processor;

import javax.annotation.processing.AbstractProcessor;
import javax.annotation.processing.Generated;
import javax.annotation.processing.ProcessingEnvironment;
import javax.annotation.processing.RoundEnvironment;
import javax.annotation.processing.SupportedAnnotationTypes;
import javax.annotation.processing.SupportedSourceVersion;
import javax.lang.model.SourceVersion;
import javax.lang.model.element.Element;
import javax.lang.model.element.PackageElement;
import javax.lang.model.element.TypeElement;
import javax.tools.Diagnostic;
import java.lang.reflect.Method;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import java.util.concurrent.atomic.AtomicInteger;

@SupportedAnnotationTypes("org.correomqtt.di.*")
@SupportedSourceVersion(SourceVersion.RELEASE_17)
public class SoyFactoryProcessor extends AbstractProcessor {

    private final Set<Element> classesToBeGenerated = new HashSet<>();
    private int totalCount = 0;
    private final AtomicInteger factoryCount = new AtomicInteger();
    private final AtomicInteger wrapperCount = new AtomicInteger();

    static String getFqnByElement(Element element) {
        return ((PackageElement) element.getEnclosingElement()).getQualifiedName() + "." + element.getSimpleName();
    }

    @Override
    public boolean process(Set<? extends TypeElement> annotations, RoundEnvironment roundEnv) {
        info("Processing soy beans...");
        for (TypeElement annotation : annotations) {
            if (!annotation.getSimpleName().toString().endsWith("Bean") && !annotation.getSimpleName().toString().endsWith("Thread")) {
                continue;
            }
            Set<? extends Element> annotatedElements = roundEnv.getElementsAnnotatedWith(annotation);
            classesToBeGenerated.addAll(annotatedElements);
        }
        List<Element> workList = new HashSet<>(classesToBeGenerated).stream()
                .filter(c -> c.getKind().isClass())
                .filter(c -> c.getAnnotation(Generated.class) == null)
                .toList();
        totalCount = Math.max(totalCount, workList.size());
        workList.forEach(classElement -> {
            try {
                ClassProcessResult result = new ClassProcessor(classElement, unwrapProcessingEnv(processingEnv)).process();
                factoryCount.getAndAdd(result.factoryCount());
                wrapperCount.getAndAdd(result.wrapperCount());
                classesToBeGenerated.remove(classElement);
            } catch (ProcessorRetryException e) {
                // Class is waiting for generated classes, so we have to wait for next round.
            }
        });
        if (factoryCount.get() > 0 && totalCount > 0) {
            info("Produced %s/%s factories and %s/%s wrapper", factoryCount.get(), totalCount, wrapperCount.get(), wrapperCount.get());
        }
        return true;
    }

    public void info(String message, Object... args) {
        processingEnv.getMessager().printMessage(Diagnostic.Kind.NOTE, String.format(message, args));
    }

    @SuppressWarnings("java:S1181")
    private ProcessingEnvironment unwrapProcessingEnv(ProcessingEnvironment wrapper) {
        ProcessingEnvironment unwrapped = null;
        try {
            final Class<?> apiWrappers = wrapper.getClass().getClassLoader().loadClass("org.jetbrains.jps.javac.APIWrappers");
            final Method unwrapMethod = apiWrappers.getDeclaredMethod("unwrap", Class.class, Object.class);
            unwrapped = (ProcessingEnvironment) unwrapMethod.invoke(null, ProcessingEnvironment.class, wrapper);
        } catch (Throwable e) {
            // This is only for intellij. Maven does not need the unwrapping.
        }
        return unwrapped != null ? unwrapped : wrapper;
    }
}
