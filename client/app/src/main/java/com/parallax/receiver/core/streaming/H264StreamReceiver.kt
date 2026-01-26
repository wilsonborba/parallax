package com.parallax.receiver.core.streaming

import android.media.MediaCodec
import android.media.MediaFormat
import android.view.Surface
import com.parallax.receiver.core.config.DEFAULT_REMOTE_HEIGHT
import com.parallax.receiver.core.config.DEFAULT_REMOTE_WIDTH
import com.parallax.receiver.core.logging.Logger
import com.parallax.receiver.core.logging.LoggerProvider
import java.net.DatagramPacket
import java.net.DatagramSocket
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch

class H264StreamReceiver(
    private val coroutineScope: CoroutineScope = CoroutineScope(SupervisorJob() + Dispatchers.IO),
    private val logger: Logger = LoggerProvider.logger,
    private val defaultWidth: Int = DEFAULT_REMOTE_WIDTH,
    private val defaultHeight: Int = DEFAULT_REMOTE_HEIGHT,
) {
    private var receiveJob: Job? = null
    private var socket: DatagramSocket? = null
    private var codec: MediaCodec? = null

    fun start(
        port: Int,
        surface: Surface,
        width: Int = defaultWidth,
        height: Int = defaultHeight,
    ) {
        logger.info(TAG, "startStream invoked", mapOf("port" to port, "width" to width, "height" to height))
        stop()
        receiveJob = coroutineScope.launch {
            val buffer = ByteArray(MAX_PACKET_SIZE)
            val packet = DatagramPacket(buffer, buffer.size)
            var mediaCodec: MediaCodec? = null
            val bufferInfo = MediaCodec.BufferInfo()
            var packetCount = 0
            var frameCount = 0
            var lastLogMs = System.currentTimeMillis()
            var loggedFirstPacket = false
            var loggedFirstFrame = false
            try {
                socket = try {
                    DatagramSocket(port).apply {
                        reuseAddress = true
                    }.also {
                        logger.info(TAG, "Socket bound", mapOf("port" to port))
                    }
                } catch (e: Exception) {
                    logger.error(
                        TAG,
                        "Failed to bind socket",
                        mapOf("error" to e.message, "exception" to e),
                    )
                    throw e
                }
                val decoder = MediaCodec.createDecoderByType(MIME_TYPE)
                mediaCodec = decoder
                val format = MediaFormat.createVideoFormat(MIME_TYPE, width, height)
                decoder.configure(format, surface, null, 0)
                decoder.start()
                codec = decoder
                logger.info(
                    TAG,
                    "Decoder initialized",
                    mapOf("mimeType" to MIME_TYPE, "width" to width, "height" to height),
                )
                while (isActive) {
                    socket?.receive(packet)
                    if (!loggedFirstPacket) {
                        logger.info(TAG, "First packet received", mapOf("bytes" to packet.length))
                        loggedFirstPacket = true
                    }
                    packetCount += 1
                    val inputIndex = decoder.dequeueInputBuffer(TIMEOUT_US)
                    if (inputIndex >= 0) {
                        val inputBuffer = decoder.getInputBuffer(inputIndex)
                        if (inputBuffer != null) {
                            inputBuffer.clear()
                            inputBuffer.put(packet.data, 0, packet.length)
                            decoder.queueInputBuffer(
                                inputIndex,
                                0,
                                packet.length,
                                System.nanoTime() / 1_000L,
                                0,
                            )
                        } else {
                            decoder.queueInputBuffer(inputIndex, 0, 0, 0L, 0)
                        }
                    }
                    var outputIndex = decoder.dequeueOutputBuffer(bufferInfo, OUTPUT_TIMEOUT_US)
                    while (outputIndex >= 0) {
                        decoder.releaseOutputBuffer(outputIndex, true)
                        frameCount += 1
                        if (!loggedFirstFrame) {
                            logger.info(TAG, "First frame decoded", emptyMap())
                            loggedFirstFrame = true
                        }
                        outputIndex = decoder.dequeueOutputBuffer(bufferInfo, OUTPUT_TIMEOUT_US)
                    }
                    val nowMs = System.currentTimeMillis()
                    if (nowMs - lastLogMs >= PACKET_LOG_INTERVAL_MS) {
                        logger.debug(
                            TAG,
                            "Stream stats",
                            mapOf("packets" to packetCount, "frames" to frameCount),
                        )
                        packetCount = 0
                        frameCount = 0
                        lastLogMs = nowMs
                    }
                }
            } catch (e: Exception) {
                if (isActive) {
                    logger.error(
                        TAG,
                        "Streaming receiver failed",
                        mapOf("error" to e.message, "exception" to e),
                    )
                }
            } finally {
                mediaCodec?.stop()
                mediaCodec?.release()
                socket?.close()
                socket = null
                codec = null
            }
        }
    }

    fun stop() {
        receiveJob?.cancel()
        receiveJob = null
        socket?.close()
        socket = null
        codec?.stop()
        codec?.release()
        codec = null
    }

    fun isRunning(): Boolean = receiveJob?.isActive == true

    private companion object {
        private const val TAG = "StreamReceiver"
        private const val MIME_TYPE = "video/avc"
        private const val MAX_PACKET_SIZE = 65_507
        private const val TIMEOUT_US = 10_000L
        private const val OUTPUT_TIMEOUT_US = 0L
        private const val PACKET_LOG_INTERVAL_MS = 1_000L
    }
}
