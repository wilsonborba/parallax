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
        stop()
        receiveJob = coroutineScope.launch {
            val buffer = ByteArray(MAX_PACKET_SIZE)
            val packet = DatagramPacket(buffer, buffer.size)
            socket = DatagramSocket(port).apply {
                reuseAddress = true
            }
            val mediaCodec = MediaCodec.createDecoderByType(MIME_TYPE)
            val format = MediaFormat.createVideoFormat(MIME_TYPE, width, height)
            mediaCodec.configure(format, surface, null, 0)
            mediaCodec.start()
            codec = mediaCodec
            val bufferInfo = MediaCodec.BufferInfo()
            try {
                while (isActive) {
                    socket?.receive(packet)
                    val inputIndex = mediaCodec.dequeueInputBuffer(TIMEOUT_US)
                    if (inputIndex >= 0) {
                        val inputBuffer = mediaCodec.getInputBuffer(inputIndex)
                        if (inputBuffer != null) {
                            inputBuffer.clear()
                            inputBuffer.put(packet.data, 0, packet.length)
                            mediaCodec.queueInputBuffer(
                                inputIndex,
                                0,
                                packet.length,
                                System.nanoTime() / 1_000L,
                                0,
                            )
                        } else {
                            mediaCodec.queueInputBuffer(inputIndex, 0, 0, 0L, 0)
                        }
                    }
                    var outputIndex = mediaCodec.dequeueOutputBuffer(bufferInfo, OUTPUT_TIMEOUT_US)
                    while (outputIndex >= 0) {
                        mediaCodec.releaseOutputBuffer(outputIndex, true)
                        outputIndex = mediaCodec.dequeueOutputBuffer(bufferInfo, OUTPUT_TIMEOUT_US)
                    }
                }
            } catch (e: Exception) {
                if (isActive) {
                    logger.error(TAG, "Streaming receiver failed", e)
                }
            } finally {
                mediaCodec.stop()
                mediaCodec.release()
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
        private const val TAG = "H264StreamReceiver"
        private const val MIME_TYPE = "video/avc"
        private const val MAX_PACKET_SIZE = 65_507
        private const val TIMEOUT_US = 10_000L
        private const val OUTPUT_TIMEOUT_US = 0L
    }
}
