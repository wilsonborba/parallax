package com.parallax.receiver.core.qr

import java.net.URI

data class PrlxQrEndpoint(
    val host: String,
    val controlPort: Int,
)

object PrlxQrParser {
    fun parse(payload: String): PrlxQrEndpoint? {
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
        return PrlxQrEndpoint(host = host, controlPort = port)
    }
}
