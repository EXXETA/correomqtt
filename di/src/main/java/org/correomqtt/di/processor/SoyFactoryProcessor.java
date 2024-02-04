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
    private final AtomicInteger processedCount = new AtomicInteger();
    private int totalCount = 0;

    static String getFqnByElement(Element element) {
        return ((PackageElement) element.getEnclosingElement()).getQualifiedName() + "." + element.getSimpleName();
    }

    @Override
    public boolean process(Set<? extends TypeElement> annotations, RoundEnvironment roundEnv) {
        info("Processing soy beans...");
        for (TypeElement annotation : annotations) {
            if (!annotation.getSimpleName().toString().endsWith("Bean")) {
                continue;
            }
            Set<? extends Element> annotatedElements = roundEnv.getElementsAnnotatedWith(annotation);
            classesToBeGenerated.addAll(annotatedElements);
        }
        List<Element> workList = new HashSet<>(classesToBeGenerated).stream()
                .filter(c -> c.getAnnotation(Generated.class) == null)
                .toList();
        totalCount = Math.max(totalCount, workList.size());
        workList.forEach(classElement -> {
            try {
                new ClassProcessor(classElement, unwrapProcessingEnv(processingEnv)).process();
                processedCount.getAndIncrement();
                classesToBeGenerated.remove(classElement);
            } catch (ProcessorRetryException e) {
                // Class is waiting for generated classes, so we have to wait for next round.
            }
        });
        if (processedCount.get() > 0 && totalCount > 0) {
            info("Produced %s/%s soy factories", processedCount.get(), totalCount);
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
            info("Unable to unwrap processing environment: %s", e.getMessage());
        }
        return unwrapped != null ? unwrapped : wrapper;
    }
}
