package com.parallax.receiver.core.qr

import java.net.URI
import java.net.URLDecoder
import java.nio.charset.StandardCharsets

data class QrEndpoint(
    val host: String,
    val controlPort: Int,
    val streamPort: Int?,
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
        val queryParams = parseQueryParams(uri.query)
        val streamPort = queryParams["streamPort"]?.toIntOrNull()?.takeIf { it > 0 }
        return QrEndpoint(
            host = host,
            controlPort = port,
            streamPort = streamPort,
        )
    }

    private fun parseQueryParams(query: String?): Map<String, String> {
        if (query.isNullOrBlank()) {
            return emptyMap()
        }
        return query.split("&")
            .mapNotNull { part ->
                if (part.isBlank()) {
                    return@mapNotNull null
                }
                val (key, value) = part.split("=", limit = 2).let {
                    it[0] to it.getOrElse(1) { "" }
                }
                if (key.isBlank()) {
                    return@mapNotNull null
                }
                val decodedKey = URLDecoder.decode(key, StandardCharsets.UTF_8.name())
                val decodedValue = URLDecoder.decode(value, StandardCharsets.UTF_8.name())
                decodedKey to decodedValue
            }
            .toMap()
    }
}
