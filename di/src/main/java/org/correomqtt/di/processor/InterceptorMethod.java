package org.correomqtt.di.processor;

import java.util.List;

public record InterceptorMethod(String name, String returnType, String visibility, List<InterceptorMethodParameters> parameters,
                                List<? extends javax.lang.model.element.AnnotationMirror> interceptorAnnotations,
                                List<? extends javax.lang.model.element.AnnotationMirror> annotationMirrors) {
}
