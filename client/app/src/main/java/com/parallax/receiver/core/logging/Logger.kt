package com.parallax.receiver.core.logging

interface Logger {
    fun debug(tag: String, message: String, metadata: Map<String, Any?> = emptyMap())
    fun info(tag: String, message: String, metadata: Map<String, Any?> = emptyMap())
    fun warn(tag: String, message: String, metadata: Map<String, Any?> = emptyMap())
    fun error(tag: String, message: String, metadata: Map<String, Any?> = emptyMap())
}
