package com.parallax.receiver.core.qr

import java.net.URI

data class QrEndpoint(
    val host: String,
    val controlPort: Int,
)

object QrParser {
    fun parse(payload: String): QrEndpoint? {
        val trimmed = payload.trim()
        if (trimmed.isEmpty()) {
            return null
        }
        val uri = runCatching { URI(trimmed) }.getOrNull() ?: return null
        if (!uri.scheme.equals("prlx", ignoreCase = true)) {
            return null
        }
        val host = uri.host ?: return null
        val port = uri.port
        if (port <= 0) {
            return null
        }
        return QrEndpoint(host = host, controlPort = port)
    }
}
