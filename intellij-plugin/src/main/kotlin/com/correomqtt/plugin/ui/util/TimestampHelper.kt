package com.correomqtt.plugin.ui.util

import org.apache.http.client.utils.DateUtils.formatDate
import java.text.DateFormat
import java.text.SimpleDateFormat
import java.time.LocalDateTime
import java.time.format.DateTimeFormatter
import java.util.*

/**
 * Utility class for timestamp specific methods.
 */
class TimestampHelper {
    companion object {
        private const val INPUT_FORMAT = "yyyy-MM-dd'T'HH:mm:ss.SSS"
        private const val US_FORMAT = "MM/dd/yyyy hh:mm:ss.SSS a"
        private const val DE_FORMAT = "dd.MM.yyyy HH:mm:ss.SSS"
        private val locale = Locale.getDefault()

        /**
         * Formats timestamp strings from format yyyy-MM-dd'T'HH:mm:ss.SSS
         * to a US or DE Format.
         */
        fun format(timestamp: String): String {
            // Trim the timestamp to keep only three milliseconds
            val trimmedTimestamp = timestamp.substring(0, 23) // yyyy-MM-ddTHH:mm:ss.SSS

            // Parse the input timestamp
            val inputFormatter = DateTimeFormatter.ofPattern(INPUT_FORMAT)
            val localDateTime = LocalDateTime.parse(trimmedTimestamp, inputFormatter)

            // Convert to Date
            val instant = localDateTime.atZone(java.time.ZoneId.systemDefault()).toInstant()
            val date = Date.from(instant)

            return if (locale == Locale.US) {
                formatDate(date, US_FORMAT)
            } else {
                formatDate(date, DE_FORMAT)
            }
        }
    }
}