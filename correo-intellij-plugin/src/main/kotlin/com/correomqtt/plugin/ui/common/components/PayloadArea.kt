package com.correomqtt.plugin.ui.common.components

import com.intellij.openapi.editor.colors.EditorColors
import com.intellij.openapi.editor.colors.EditorColorsManager
import com.intellij.openapi.editor.colors.EditorColorsScheme
import com.intellij.openapi.editor.colors.EditorFontType
import org.fife.ui.autocomplete.AutoCompletion
import org.fife.ui.autocomplete.CompletionProvider
import org.fife.ui.autocomplete.DefaultCompletionProvider
import org.fife.ui.rsyntaxtextarea.RSyntaxTextArea
import org.fife.ui.rsyntaxtextarea.SyntaxConstants
import java.awt.Font

class PayloadArea {
    fun createJsonTextArea(): RSyntaxTextArea {
        val textArea = RSyntaxTextArea()
        textArea.syntaxEditingStyle = SyntaxConstants.SYNTAX_STYLE_JSON
        textArea.isCodeFoldingEnabled = true

        val provider = createCompletionProvider()
        val ac = AutoCompletion(provider)
        ac.install(textArea)

        applyIntelliJTheme(textArea)
        return textArea
    }

    private fun getEditorFont(colorsScheme: EditorColorsScheme): Font {
        // Retrieve the editor font from the current scheme
        val editorFont = colorsScheme.getFont(EditorFontType.PLAIN)
        return Font(editorFont.family, Font.PLAIN, editorFont.size)
    }

    private fun createCompletionProvider(): CompletionProvider {
        val provider = DefaultCompletionProvider()
        return provider
    }

    private fun applyIntelliJTheme(textArea: RSyntaxTextArea) {
        // Get the current editor colors scheme
        val colorsScheme = EditorColorsManager.getInstance().globalScheme

        // Apply colors and fonts from IntelliJ theme to RSyntaxTextArea
        textArea.background = colorsScheme.defaultBackground
        textArea.foreground = colorsScheme.defaultForeground
        textArea.font = getEditorFont(colorsScheme)

        // Apply caret color
        textArea.caretColor = colorsScheme.getColor(EditorColors.CARET_COLOR)

        // Apply selection color
        textArea.selectionColor = colorsScheme.getColor(EditorColors.SELECTION_BACKGROUND_COLOR)
        textArea.selectedTextColor = colorsScheme.getColor(EditorColors.SELECTION_FOREGROUND_COLOR)
        textArea.currentLineHighlightColor = colorsScheme.getColor(EditorColors.CARET_ROW_COLOR)
    }
}