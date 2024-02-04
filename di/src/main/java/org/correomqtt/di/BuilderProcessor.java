package org.correomqtt.di;

import javax.annotation.processing.AbstractProcessor;
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
import java.util.Set;
import java.util.concurrent.atomic.AtomicInteger;

@SupportedAnnotationTypes("org.correomqtt.di.*")
@SupportedSourceVersion(SourceVersion.RELEASE_17)
public class BuilderProcessor extends AbstractProcessor {

    @Override
    public boolean process(Set<? extends TypeElement> annotations, RoundEnvironment roundEnv) {
        for (TypeElement annotation : annotations) {
            if (!annotation.getSimpleName().toString().endsWith("Bean")) {
                continue;
            }
            info("Processing Beans @%s", getFqnByElement(annotation));
            AtomicInteger count = new AtomicInteger();
            Set<? extends Element> annotatedElements = roundEnv.getElementsAnnotatedWith(annotation);
            annotatedElements.forEach(classElement -> {
                new ClassProcessor(classElement, unwrapProcessingEnv(processingEnv)).process();
                count.getAndIncrement();
            });
            info("Produced %s assisted factories for @%s", count.get(), getFqnByElement(annotation));
        }
        return true;
    }

    private static ProcessingEnvironment unwrapProcessingEnv(ProcessingEnvironment wrapper) {
        ProcessingEnvironment unwrapped = null;
        try {
            final Class<?> apiWrappers = wrapper.getClass().getClassLoader().loadClass("org.jetbrains.jps.javac.APIWrappers");
            final Method unwrapMethod = apiWrappers.getDeclaredMethod("unwrap", Class.class, Object.class);
            unwrapped = (ProcessingEnvironment) unwrapMethod.invoke(null, ProcessingEnvironment.class, wrapper);
        } catch (Throwable ignored) {}
        return unwrapped != null ? unwrapped : wrapper;
    }

    static String getFqnByElement(Element element) {
        return ((PackageElement) element.getEnclosingElement()).getQualifiedName() + "." + element.getSimpleName();
    }

    public void info(String message, Object... args) {
        processingEnv.getMessager().printMessage(Diagnostic.Kind.NOTE, String.format(message, args));
    }


}
