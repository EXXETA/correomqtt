package org.correomqtt.di;

import com.sun.source.tree.CompilationUnitTree;
import com.sun.source.tree.ImportTree;
import com.sun.source.util.TreePath;
import com.sun.source.util.Trees;
import lombok.AllArgsConstructor;
import lombok.Getter;

import javax.annotation.processing.ProcessingEnvironment;
import javax.lang.model.element.Element;
import javax.lang.model.element.ElementKind;
import javax.lang.model.element.ExecutableElement;
import javax.lang.model.element.TypeElement;
import javax.lang.model.element.VariableElement;
import javax.lang.model.type.TypeMirror;
import javax.lang.model.type.TypeVariable;
import javax.tools.Diagnostic;
import javax.tools.JavaFileObject;
import java.io.IOException;
import java.io.PrintWriter;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Set;
import java.util.regex.Matcher;
import java.util.regex.Pattern;
import java.util.stream.Collectors;

import static org.correomqtt.di.BuilderProcessor.getFqnByElement;

public class ClassProcessor {

    private static final Pattern TYPE_PATTERN = Pattern.compile("([A-Za-z0-9_.]+)");
    private final Element classElement;
    private final ProcessingEnvironment processingEnv;
    private final Set<String> imports = new HashSet<>();
    private Element constructor;
    private List<Parameter> constructorParameters;
    private String className;
    private String longGenericsString;
    private String shortGenericsString;
    private String packageName;
    private String simpleClassName;
    private String factoryClassName;
    private String simpleFactoryClassName;

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

    public void process() {
        className = getFqnByElement(classElement);
        try {
            if (!findConstructor()) {
                return;
            }
            if (!findConstructorParameter()) {
                return;
            }
            findGenerics();
            findNames();
            writeBuilderFile();
        } catch (Exception e) {
            error("Exception parsing %s: %s", className, e.getMessage());
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
        return true;
    }

    private boolean findConstructorParameter() {
        List<? extends VariableElement> constructorParametersElements = ((ExecutableElement) constructor).getParameters();
        if (constructorParametersElements.stream().noneMatch(p -> p.getAnnotation(Assisted.class) != null)) {
            return false;
        }
        constructorParameters = constructorParametersElements.stream().map(p -> {
                    String typeString = extractType(classElement, p, imports);
                    String referenceString;
                    if (typeString.contains("<")) {
                        referenceString = "new TypeReference<" + typeString + ">(){}";
                        imports.add("org.correomqtt.di.TypeReference");
                    } else {
                        referenceString = typeString + ".class";
                    }
                    boolean assisted = p.getAnnotation(Assisted.class) != null;
                    if (!assisted) {
                        imports.add("org.correomqtt.di.SoyDi");
                    }
                    return new Parameter(assisted,
                            typeString,
                            p.getSimpleName().toString(),
                            referenceString);
                }
        ).toList();
        return true;
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
                        return e.getKey() + " extends " + shortenType(e.getValue(), imports);
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
        factoryClassName = className + "Factory";
        simpleFactoryClassName = factoryClassName.substring(lastDot + 1);
        if (!shortGenericsString.isEmpty()) {
            simpleClassName += "<" + shortGenericsString + ">";
        }
        if (!longGenericsString.isEmpty()) {
            simpleFactoryClassName += "<" + longGenericsString + ">";
        }
    }

    private void writeBuilderFile() throws IOException {
        imports.add("org.correomqtt.di.DefaultBean");
        JavaFileObject builderFile = processingEnv.getFiler().createSourceFile(factoryClassName);
        try (PrintWriter out = new PrintWriter(builderFile.openWriter())) {
            if (packageName != null) {
                out.print("package ");
                out.print(packageName);
                out.println(";");
                out.println();
            }
            if (!imports.isEmpty()) {
                out.print(imports.stream()
                        .filter(Objects::nonNull)
                        .map(i -> "import " + i + ";")
                        .collect(Collectors.joining("\n")));
                out.println();
                out.println();
            }
            out.println("// Generated with SoyDi");
            out.println("@DefaultBean");
            out.print("public class ");
            out.print(simpleFactoryClassName);
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
            out.print("    return new " + simpleClassName + "(");
            out.print(constructorParameters.stream().map(p -> p.name).collect(Collectors.joining(",\n          ")));
            out.println(");");
            out.println("  }");
            out.println("}");
        }
    }

    private void error(String message, Object... args) {
        processingEnv.getMessager().printMessage(Diagnostic.Kind.ERROR, String.format(message, args));
    }

    private String extractType(Element clazzElement, Element element, Set<String> imports) {
        String typeString = element.asType().toString();
        String plainType;
        if (typeString.contains("<")) {
            plainType = typeString.substring(0, typeString.indexOf("<"));
        } else {
            plainType = typeString;
        }
        if (!plainType.contains(".")) {
            imports.add(getImportFix(clazzElement, plainType));
        }
        // Shorten -> TODO handle duplicates
        return shortenType(typeString, imports);
    }

    private String shortenType(String typeString, Set<String> imports) {
        Matcher matcher = TYPE_PATTERN.matcher(typeString);
        while (matcher.find()) {
            String fqn = matcher.group();
            if (!fqn.contains(".")) {
                continue;
            }
            imports.add(fqn);
            typeString = typeString.replace(fqn, fqn.substring(fqn.lastIndexOf(".") + 1));
        }
        return typeString;
    }

    private String getImportFix(Element element, String simpleClassName) {
        CompilationUnitTree compilationUnit = getCompilationUnit(element);
        List<? extends ImportTree> imports = compilationUnit.getImports();
        for (ImportTree importTree : imports) {
            String importString = importTree.getQualifiedIdentifier().toString();
            if (importString.endsWith(simpleClassName)) {
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
}
