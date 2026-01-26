package com.parallax.receiver.domain.model

data class StreamState(
    val status: Status,
    val message: String? = null,
) {
    enum class Status {
        Idle,
        Connecting,
        Streaming,
        Error,
    }
}
