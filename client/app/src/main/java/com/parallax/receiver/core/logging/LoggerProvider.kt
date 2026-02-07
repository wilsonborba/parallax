package com.parallax.receiver.core.logging

object LoggerProvider {
    val logger: Logger by lazy { AndroidLogger() }
}
