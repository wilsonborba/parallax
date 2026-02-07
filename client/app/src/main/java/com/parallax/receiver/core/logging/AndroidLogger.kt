package com.parallax.receiver.core.logging

import android.util.Log

class AndroidLogger : Logger {
    override fun debug(tag: String, message: String, metadata: Map<String, Any?>) {
        Log.d(tag, formatMessage(message, metadata))
    }

    override fun info(tag: String, message: String, metadata: Map<String, Any?>) {
        Log.i(tag, formatMessage(message, metadata))
    }

    override fun warn(tag: String, message: String, metadata: Map<String, Any?>) {
        Log.w(tag, formatMessage(message, metadata))
    }

    override fun error(tag: String, message: String, metadata: Map<String, Any?>) {
        Log.e(tag, formatMessage(message, metadata))
    }

    private fun formatMessage(message: String, metadata: Map<String, Any?>): String {
        if (metadata.isEmpty()) {
            return message
        }
        return buildString {
            append(message)
            append(" | ")
            metadata.entries.joinTo(this, separator = ", ") { (key, value) ->
                "$key=$value"
            }
        }
    }
}
