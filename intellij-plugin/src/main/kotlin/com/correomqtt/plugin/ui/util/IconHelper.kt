package com.correomqtt.plugin.ui.util

import com.intellij.util.ui.UIUtil
import java.awt.AlphaComposite
import java.awt.image.BufferedImage
import javax.swing.Icon
import javax.swing.ImageIcon

/**
 * Utility class for icon specific helper methods.
 */
class IconHelper {
    companion object {

        /**
         * Creates greyed icons for ui components.
         */
        fun createGreyIcon(originalIcon: Icon): Icon {
            val image = BufferedImage(originalIcon.iconWidth, originalIcon.iconHeight, BufferedImage.TYPE_INT_ARGB)
            val g = image.createGraphics()

            // Draw the original icon
            originalIcon.paintIcon(null, g, 0, 0)

            // Apply a grey tint overlay
            g.composite = AlphaComposite.SrcAtop
            g.color = UIUtil.getInactiveTextColor()// Custom grey shade
            g.fillRect(0, 0, image.width, image.height)
            g.dispose()

            return ImageIcon(image)
        }
    }
}