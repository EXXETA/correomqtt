package org.correomqtt.di.processor;

import com.sun.source.tree.CompilationUnitTree;
import com.sun.source.tree.ImportTree;
import com.sun.source.util.TreePath;
import com.sun.source.util.Trees;
import lombok.AllArgsConstructor;
import lombok.Getter;
import org.correomqtt.di.Assisted;
import org.correomqtt.di.Inject;
import org.correomqtt.di.Interceptor;

import javax.annotation.processing.ProcessingEnvironment;
import javax.lang.model.element.AnnotationMirror;
import javax.lang.model.element.Element;
import javax.lang.model.element.ElementKind;
import javax.lang.model.element.ExecutableElement;
import javax.lang.model.element.Modifier;
import javax.lang.model.element.TypeElement;
import javax.lang.model.element.VariableElement;
import javax.lang.model.type.TypeMirror;
import javax.lang.model.type.TypeVariable;
import javax.tools.Diagnostic;
import javax.tools.JavaFileObject;
import java.io.IOException;
import java.io.PrintWriter;
import java.util.Collections;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Set;
import java.util.regex.Matcher;
import java.util.regex.Pattern;
import java.util.stream.Collectors;

import static javax.lang.model.element.ElementKind.METHOD;
import static javax.lang.model.element.Modifier.PRIVATE;
import static javax.lang.model.element.Modifier.PROTECTED;
import static javax.lang.model.element.Modifier.PUBLIC;
import static org.correomqtt.di.processor.SoyFactoryProcessor.getFqnByElement;

public class ClassProcessor {

    public static final String ORG_CORREOMQTT_DI_DEFAULT_BEAN = "org.correomqtt.di.DefaultBean";
    public static final String JAVAX_ANNOTATION_PROCESSING_GENERATED = "javax.annotation.processing.Generated";
    public static final String ORG_CORREOMQTT_DI_SOY_INTERCEPTOR = "org.correomqtt.di.SoyInterceptor";
    public static final String ORG_CORREOMQTT_DI_SOY_EVENTS = "org.correomqtt.di.SoyEvents";
    public static final String ORG_CORREOMQTT_DI_SOY_DI = "org.correomqtt.di.SoyDi";
    public static final String ORG_CORREOMQTT_DI_FACTORY = "org.correomqtt.di.Factory";
    public static final String ORG_CORREOMQTT_DI_TYPE_REFERENCE = "org.correomqtt.di.TypeReference";
    public static final String PARAMETER_JOIN_BREAK = ",\n    ";
    private static final Pattern TYPE_PATTERN = Pattern.compile("([A-Za-z0-9_.]+)");
    public static final String FACTORY_POSTFIX = "Factory";
    public static final String WRAPPER_POSTFIX = "Wrapper";
    private final Element classElement;
    private final ProcessingEnvironment processingEnv;
    private final Set<String> factoryImports = new HashSet<>();
    private final Set<String> wrapperImports = new HashSet<>();
    private final Set<String> shortTypes = new HashSet<>();
    private Element constructor;
    private List<Parameter> constructorParameters;
    private String className;
    private String longGenericsString;
    private String shortGenericsString;
    private String packageName;
    private String simpleClassName;
    private String factoryClassName;
    private String simpleFactoryClassName;
    private List<InterceptorMethod> interceptorMethods;
    private String wrapperClassName;
    private String simpleWrapperClassName;
    private List<? extends AnnotationMirror> constructorAnnotations;
    private String classReferenceString;
    private String constructorVisibility;
    private String classVisibility;

    @AllArgsConstructor
    @Getter
    private static class Parameter {
        private boolean assisted;
        private String type;
        private String name;
        private String reference;
    }

    public ClassProcessor(Element classElement,
                          ProcessingEnvironment processingEnv) {
        this.classElement = classElement;
        this.processingEnv = processingEnv;
    }

    public ClassProcessResult process() {
        int factoryCount = 0;
        int wrapperCount = 0;
        className = getFqnByElement(classElement);
        try {
            factoryImports.add(ORG_CORREOMQTT_DI_DEFAULT_BEAN);
            factoryImports.add(JAVAX_ANNOTATION_PROCESSING_GENERATED);
            wrapperImports.add(JAVAX_ANNOTATION_PROCESSING_GENERATED);
            wrapperImports.add(ORG_CORREOMQTT_DI_SOY_INTERCEPTOR);
            factoryImports.add(ORG_CORREOMQTT_DI_SOY_EVENTS);
            factoryImports.add(ORG_CORREOMQTT_DI_SOY_DI);
            factoryImports.add(ORG_CORREOMQTT_DI_FACTORY);
            classVisibility = getVisibility(classElement);
            if (!findConstructor()) {
                return new ClassProcessResult(factoryCount, wrapperCount);
            }
            findConstructorParameter();
            findGenerics();
            findNames();
            findInterceptorMethods();
            factoryCount += writeFactoryFile();
            wrapperCount += writeWrapperFile();
        } catch (ProcessorRetryException e) {
            throw e;
        } catch (Exception e) {
            error("Exception parsing %s: %s", className, e.getMessage());
        }
        return new ClassProcessResult(factoryCount, wrapperCount);
    }

    private String getVisibility(Element element) {
        Set<Modifier> modifiers = element.getModifiers();
        if (modifiers.contains(PUBLIC)) {
            return "public";
        } else if (modifiers.contains(PROTECTED)) {
            return "protected";
        } else if (modifiers.contains(PRIVATE)) {
            return "private";
        } else {
            return "";
        }
    }

    private boolean findConstructor() {
        List<? extends Element> constructors = classElement.getEnclosedElements()
                .stream()
                .filter(el -> el.getKind() == ElementKind.CONSTRUCTOR)
                .toList();
        if (constructors.size() > 1) {
            List<? extends Element> annotatedConstructors = constructors.stream()
                    .filter(c -> c.asType().getAnnotation(Inject.class) != null)
                    .toList();
            if (annotatedConstructors.isEmpty()) {
                error("If more than one constructor exists. Exactly one must be annotated with @Inject: %s", getFqnByElement(classElement));
                return false;
            } else if (annotatedConstructors.size() > 1) {
                error("Only one constructor can be annotated with the @Inject annotation: %s", getFqnByElement(classElement));
                return false;
            }
        }
        constructor = constructors.get(0);
        constructorVisibility = getVisibility(constructor);
        constructorAnnotations = constructor.getAnnotationMirrors();
        return true;
    }

    private void findConstructorParameter() {
        List<? extends VariableElement> constructorParametersElements = ((ExecutableElement) constructor).getParameters();
        constructorParameters = constructorParametersElements.stream().map(p -> {
                    HashSet<Set<String>> imports = new HashSet<>();
                    imports.add(factoryImports);
                    imports.add(wrapperImports);
                    String typeString = extractType(classElement, p, imports);
                    String referenceString;
                    if (typeString.contains("<")) {
                        referenceString = "new TypeReference<" + typeString + ">(){}";
                        factoryImports.add(ORG_CORREOMQTT_DI_TYPE_REFERENCE);
                        wrapperImports.add(ORG_CORREOMQTT_DI_TYPE_REFERENCE);
                    } else {
                        referenceString = typeString + ".class";
                    }
                    boolean assisted = p.getAnnotation(Assisted.class) != null;
                    if (!assisted) {
                        factoryImports.add(ORG_CORREOMQTT_DI_SOY_DI);
                        wrapperImports.add(ORG_CORREOMQTT_DI_SOY_DI);
                    }
                    return new Parameter(assisted,
                            typeString,
                            p.getSimpleName().toString(),
                            referenceString);
                }
        ).toList();
    }

    private void findGenerics() {
        Map<String, String> upperBoundGenerics = new HashMap<>();
        ((TypeElement) classElement).getTypeParameters().forEach(tp -> {
            TypeMirror typeParamType = tp.asType();
            TypeVariable typeVariable = (TypeVariable) typeParamType;
            TypeMirror upperBound = typeVariable.getUpperBound();
            upperBoundGenerics.put(tp.getSimpleName().toString(), upperBound.toString());
        });
        longGenericsString = upperBoundGenerics.entrySet().stream()
                .map(e -> {
                    if (e.getValue() == null) {
                        return e.getKey();
                    } else {
                        return e.getKey() + " extends " + shortenType(e.getValue(), Collections.singleton(factoryImports));
                    }
                })
                .collect(Collectors.joining(", "));
        shortGenericsString = String.join(", ", upperBoundGenerics.keySet());
    }

    private void findNames() {
        packageName = null;
        int lastDot = className.lastIndexOf('.');
        if (lastDot > 0) {
            packageName = className.substring(0, lastDot);
        }
        simpleClassName = className.substring(lastDot + 1);
        factoryClassName = className + FACTORY_POSTFIX;
        wrapperClassName = className + WRAPPER_POSTFIX;
        simpleFactoryClassName = factoryClassName.substring(lastDot + 1);
        simpleWrapperClassName = wrapperClassName.substring(lastDot + 1);
        if (!shortGenericsString.isEmpty()) {
            simpleClassName += "<" + shortGenericsString + ">";
        }
        if (!longGenericsString.isEmpty()) {
            simpleFactoryClassName += "<" + longGenericsString + ">";
            simpleWrapperClassName += "<" + simpleWrapperClassName + ">";
        }
        if (simpleClassName.contains("<")) {
            classReferenceString = "new TypeReference<" + simpleClassName + ">(){}";
            factoryImports.add(ORG_CORREOMQTT_DI_TYPE_REFERENCE);
        } else {
            classReferenceString = simpleClassName + ".class";
        }
    }

    private void findInterceptorMethods() {
        interceptorMethods = classElement.getEnclosedElements()
                .stream()
                .filter(m -> m.getKind().equals(METHOD))
                .filter(m -> m.getAnnotationMirrors()
                        .stream()
                        .anyMatch(am -> am.getAnnotationType()
                                .asElement()
                                .getAnnotationsByType(Interceptor.class).length != 0))
                .map(m -> {
                    List<? extends AnnotationMirror> interceptorAnnotations = m.getAnnotationMirrors()
                            .stream()
                            .filter(am -> am.getAnnotationType()
                                    .asElement()
                                    .getAnnotationsByType(Interceptor.class).length != 0)
                            .toList();
                    ExecutableElement ee = (ExecutableElement) m;
                    List<InterceptorMethodParameters> parameters = ee.getParameters().stream()
                            .map(p -> {
                                String shortType = shortenType(p.asType().toString(), Collections.singleton(wrapperImports));
                                return new InterceptorMethodParameters(p.getSimpleName().toString(), shortType);
                            })
                            .toList();
                    return new InterceptorMethod(m.getSimpleName().toString(),
                            ee.getReturnType().toString(),
                            getVisibility(m),
                            parameters,
                            interceptorAnnotations,
                            m.getAnnotationMirrors()
                    );
                })
                .toList();
    }

    private int writeFactoryFile() throws IOException {
        String createClassName;
        if (interceptorMethods.isEmpty()) {
            createClassName = simpleClassName;
        } else {
            createClassName = simpleWrapperClassName;
        }
        JavaFileObject builderFile = processingEnv.getFiler().createSourceFile(factoryClassName);
        try (PrintWriter out = new PrintWriter(builderFile.openWriter())) {
            printPackage(out);
            printImports(factoryImports, out);
            printGeneratedAnnotation(out);
            out.println("@DefaultBean");
            out.print("public class ");
            out.print(simpleFactoryClassName);
            out.print(" implements Factory");
            out.println(" {");
            out.println();
            out.print("  public ");
            out.print(simpleClassName);
            out.print(" create(");
            out.print(constructorParameters.stream()
                    .filter(p -> p.assisted)
                    .map(p -> p.type + " " + p.name)
                    .collect(Collectors.joining(", ")));
            out.println("){");
            out.println();
            if (!constructorParameters.isEmpty()) {
                out.println(constructorParameters.stream()
                        .filter(p -> !p.assisted)
                        .map(p -> "    " + p.type + " " + p.name + " = SoyDi.inject(" + p.reference + ");\n")
                        .collect(Collectors.joining()));
            }
            out.println("    try {");
            out.print("    " + simpleClassName + " instance = new " + createClassName + "(");
            out.print(constructorParameters.stream().map(p -> p.name).collect(Collectors.joining(",\n          ")));
            out.println(");");
            out.println("      SoyEvents.registerInstance(" + classReferenceString + ", instance);");
            out.println("      return instance;");
            out.println("    } catch(Exception e) {");
            out.println("      throw new IllegalStateException(\"Unable to instanciate class: " + simpleClassName + "\", e);");
            out.println("    }");
            out.println("  }");
            out.println("}");
        }
        return 1;
    }

    private int writeWrapperFile() throws IOException {
        if (interceptorMethods.isEmpty()) {
            return 0;
        }
        JavaFileObject builderFile = processingEnv.getFiler().createSourceFile(wrapperClassName);
        try (PrintWriter out = new PrintWriter(builderFile.openWriter())) {
            printPackage(out);
            printImports(wrapperImports, out);
            printGeneratedAnnotation(out);
            out.println(classVisibility + " class " + simpleWrapperClassName + " extends " + simpleClassName + "{");
            constructorAnnotations.forEach(ca -> out.println("  " + ca));
            out.print("  " + constructorVisibility + " " + simpleWrapperClassName + "(");
            out.print(constructorParameters.stream()
                    .map(p -> p.type + " " + p.name)
                    .collect(Collectors.joining(PARAMETER_JOIN_BREAK)));
            out.println("){");
            out.print("    super(");
            out.print(constructorParameters.stream()
                    .map(p -> p.name)
                    .collect(Collectors.joining(",\n      ")));
            out.println(");");
            out.println("  }");
            out.println();
            interceptorMethods.forEach(m -> {
                m.annotationMirrors().forEach(ia -> out.println("  " + ia));
                out.print("  " + m.visibility() + " " + m.returnType() + " " + m.name() + "(");
                out.print(m.parameters().stream().map(p -> p.type() + " " + p.name()).collect(Collectors.joining(PARAMETER_JOIN_BREAK)));
                out.println("){");
                out.print("    ");
                if (!m.returnType().equals("void")) {
                    out.print("return (" + m.returnType() + ")");
                }
                out.print("SoyInterceptor.intercept(" + m.interceptorAnnotations().get(0).getAnnotationType() + ".class, () -> ");
                if (m.returnType().equals("void")) {
                    out.print("{ ");
                }
                out.print("super." + m.name() + "(");
                out.print(m.parameters().stream().map(InterceptorMethodParameters::name).collect(Collectors.joining(PARAMETER_JOIN_BREAK)));
                out.print(")");
                if (m.returnType().equals("void")) {
                    out.print("; return null; }");
                }
                out.println(");");
                out.println("  }");
                out.println();
            });
            out.println("}");
        }
        return 1;
    }

    private void error(String message, Object... args) {
        processingEnv.getMessager().printMessage(Diagnostic.Kind.ERROR, String.format(message, args));
    }

    private String extractType(Element clazzElement, Element element, Set<Set<String>> imports) {
        String typeString = element.asType().toString();
        String plainType;
        if (typeString.equals("<any>")) { // usage of generated classes -> try in next round
            throw new ProcessorRetryException();
        }
        if (typeString.contains("<")) {
            plainType = typeString.substring(0, typeString.indexOf("<"));
        } else {
            plainType = typeString;
        }
        if (!plainType.contains(".")) {
            imports.forEach(im -> im.add(getImportFix(clazzElement, plainType)));
        }
        return shortenType(typeString, imports);
    }

    private String shortenType(String typeString, Set<Set<String>> imports) {
        Matcher matcher = TYPE_PATTERN.matcher(typeString);
        while (matcher.find()) {
            String fqn = matcher.group();
            if (!fqn.contains(".")) {
                shortTypes.add(fqn);
            } else {
                String shortType = fqn.substring(fqn.lastIndexOf(".") + 1);
                if (!shortTypes.contains(shortType)) {
                    typeString = typeString.replace(fqn, shortType);
                    imports.forEach(im -> im.add(fqn));
                    shortTypes.add(shortType);
                }
            }
        }
        return typeString;
    }

    private void printPackage(PrintWriter out) {
        if (packageName != null) {
            out.print("package ");
            out.print(packageName);
            out.println(";");
            out.println();
        }
    }

    private void printImports(Set<String> imports, PrintWriter out) {
        if (!imports.isEmpty()) {
            out.print(imports.stream()
                    .filter(Objects::nonNull)
                    .map(i -> "import " + i + ";")
                    .sorted()
                    .collect(Collectors.joining("\n")));
            out.println();
            out.println();
        }
    }

    private static void printGeneratedAnnotation(PrintWriter out) {
        out.println("@Generated(\"org.correomqtt.di.processor.SoyFactoryProcessor\")");
    }

    private String getImportFix(Element element, String simpleClassName) {
        CompilationUnitTree compilationUnit = getCompilationUnit(element);
        List<? extends ImportTree> actualImports = compilationUnit.getImports();
        for (ImportTree importTree : actualImports) {
            String importString = importTree.getQualifiedIdentifier().toString();
            if (importString.endsWith("." + simpleClassName)) {
                return importString;
            }
        }
        return null; // maybe in same package
    }

    private CompilationUnitTree getCompilationUnit(Element element) {
        Trees trees = Trees.instance(processingEnv);
        TreePath path = trees.getPath(element);
        return path.getCompilationUnit();
    }

    public void info(String message, Object... args) {
        processingEnv.getMessager().printMessage(Diagnostic.Kind.NOTE, String.format(message, args));
    }
}
