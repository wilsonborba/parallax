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
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.util.LinkedHashMap
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
            val frameAssembler = FrameAssembler(logger)
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
                    val frameData = frameAssembler.onPacket(packet.data, packet.length) ?: continue
                    val inputIndex = decoder.dequeueInputBuffer(TIMEOUT_US)
                    if (inputIndex >= 0) {
                        val inputBuffer = decoder.getInputBuffer(inputIndex)
                        if (inputBuffer != null) {
                            inputBuffer.clear()
                            inputBuffer.put(frameData)
                            decoder.queueInputBuffer(
                                inputIndex,
                                0,
                                frameData.size,
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
        private const val MAGIC = "PRLX"
        private const val HEADER_LENGTH = 24
        private const val VERSION = 1
        private const val MAX_IN_FLIGHT_FRAMES = 60
    }

    private data class PacketHeader(
        val flags: Int,
        val frameId: Long,
        val packetId: Int,
        val packetCount: Int,
        val payloadLength: Int,
    )

    private class FrameAssembler(private val logger: Logger) {
        private val frames = object : LinkedHashMap<Long, FrameAssembly>(MAX_IN_FLIGHT_FRAMES, 0.75f, true) {
            override fun removeEldestEntry(eldest: MutableMap.MutableEntry<Long, FrameAssembly>?): Boolean {
                return size > MAX_IN_FLIGHT_FRAMES
            }
        }

        fun onPacket(bytes: ByteArray, length: Int): ByteArray? {
            if (length < HEADER_LENGTH) {
                logger.warn(TAG, "Dropping packet: too small", mapOf("length" to length))
                return null
            }
            val header = parseHeader(bytes) ?: return null
            if (header.payloadLength <= 0 || header.payloadLength > length - HEADER_LENGTH) {
                logger.warn(
                    TAG,
                    "Dropping packet: invalid payload length",
                    mapOf("payloadLength" to header.payloadLength, "length" to length),
                )
                return null
            }
            val payload = bytes.copyOfRange(HEADER_LENGTH, HEADER_LENGTH + header.payloadLength)
            val assembly = frames.getOrPut(header.frameId) {
                FrameAssembly(header.packetCount)
            }
            if (header.packetId >= assembly.packetCount) {
                logger.warn(
                    TAG,
                    "Dropping packet: packetId out of range",
                    mapOf("packetId" to header.packetId, "packetCount" to assembly.packetCount),
                )
                return null
            }
            assembly.packets[header.packetId] = payload
            if (assembly.isComplete()) {
                frames.remove(header.frameId)
                return assembly.assemble()
            }
            return null
        }

        private fun parseHeader(bytes: ByteArray): PacketHeader? {
            val magic = String(bytes, 0, 4)
            if (magic != MAGIC) {
                logger.warn(TAG, "Dropping packet: invalid magic", mapOf("magic" to magic))
                return null
            }
            val version = bytes[4].toInt() and 0xFF
            val headerLength = bytes[5].toInt() and 0xFF
            if (version != VERSION || headerLength != HEADER_LENGTH) {
                logger.warn(
                    TAG,
                    "Dropping packet: invalid header",
                    mapOf("version" to version, "headerLength" to headerLength),
                )
                return null
            }
            val buffer = ByteBuffer.wrap(bytes).order(ByteOrder.BIG_ENDIAN)
            val flags = buffer.getShort(6).toInt() and 0xFFFF
            val frameId = buffer.getInt(12).toLong() and 0xFFFF_FFFFL
            val packetId = buffer.getShort(16).toInt() and 0xFFFF
            val packetCount = buffer.getShort(18).toInt() and 0xFFFF
            val payloadLength = buffer.getShort(22).toInt() and 0xFFFF
            if (packetCount == 0) {
                logger.warn(TAG, "Dropping packet: packetCount=0", emptyMap())
                return null
            }
            return PacketHeader(flags, frameId, packetId, packetCount, payloadLength)
        }
    }

    private class FrameAssembly(val packetCount: Int) {
        val packets: Array<ByteArray?> = arrayOfNulls(packetCount)

        fun isComplete(): Boolean = packets.all { it != null }

        fun assemble(): ByteArray {
            val totalSize = packets.sumOf { it?.size ?: 0 }
            val output = ByteArray(totalSize)
            var offset = 0
            for (packet in packets) {
                if (packet != null) {
                    System.arraycopy(packet, 0, output, offset, packet.size)
                    offset += packet.size
                }
            }
            return output
        }
    }
}
