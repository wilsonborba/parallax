package com.parallax.receiver.core.streaming

import android.media.MediaCodec
import android.media.MediaFormat
import android.view.Surface
import com.parallax.receiver.core.config.DEFAULT_REMOTE_HEIGHT
import com.parallax.receiver.core.config.DEFAULT_REMOTE_WIDTH
import com.parallax.receiver.core.logging.Logger
import com.parallax.receiver.core.logging.LoggerProvider
import com.parallax.receiver.domain.model.VideoDimensions
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
    private var onVideoDimensionsDetected: ((VideoDimensions) -> Unit)? = null
    private var lastDetectedDimensions: VideoDimensions? = null

    fun setOnVideoDimensionsDetected(callback: (VideoDimensions) -> Unit) {
        onVideoDimensionsDetected = callback
    }

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
            var pendingConfig: ByteArray? = null
            var configSent = false
            var pendingFrame: FramePayload? = null
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
                while (isActive) {
                    if (pendingFrame == null) {
                        socket?.receive(packet)
                        if (!loggedFirstPacket) {
                            logger.info(TAG, "First packet received", mapOf("bytes" to packet.length))
                            loggedFirstPacket = true
                        }
                        packetCount += 1
                        val framePayload = frameAssembler.onPacket(packet.data, packet.length) ?: continue
                        if ((framePayload.flags and FLAG_DISCONTINUITY) != 0) {
                            pendingConfig = null
                            configSent = false
                        }
                        if (framePayload.configData != null) {
                            pendingConfig = framePayload.configData
                            configSent = false
                        }
                        if ((framePayload.flags and FLAG_KEYFRAME) != 0 && pendingConfig != null) {
                            configSent = false
                        }
                        pendingFrame = framePayload
                    }
                    if (mediaCodec == null && (pendingConfig != null || pendingFrame != null)) {
                        val dimensions = pendingConfig?.let { parseDimensionsFromConfig(it) }
                        if (dimensions != null && dimensions != lastDetectedDimensions) {
                            lastDetectedDimensions = dimensions
                            onVideoDimensionsDetected?.invoke(dimensions)
                        }
                        val resolvedWidth = dimensions?.width ?: width
                        val resolvedHeight = dimensions?.height ?: height
                        val decoder = MediaCodec.createDecoderByType(MIME_TYPE)
                        mediaCodec = decoder
                        val format = MediaFormat.createVideoFormat(MIME_TYPE, resolvedWidth, resolvedHeight)
                        decoder.configure(format, surface, null, 0)
                        decoder.start()
                        codec = decoder
                        logger.info(
                            TAG,
                            "Decoder initialized",
                            mapOf("mimeType" to MIME_TYPE, "width" to resolvedWidth, "height" to resolvedHeight),
                        )
                    }
                    val frameToSend = pendingFrame
                    val decoder = mediaCodec
                    if (frameToSend != null && decoder != null) {
                        val inputIndex = decoder.dequeueInputBuffer(TIMEOUT_US)
                        if (inputIndex >= 0) {
                            val inputBuffer = decoder.getInputBuffer(inputIndex)
                            if (inputBuffer != null) {
                                inputBuffer.clear()
                                if (pendingConfig != null && !configSent) {
                                    inputBuffer.put(pendingConfig)
                                    decoder.queueInputBuffer(
                                        inputIndex,
                                        0,
                                        pendingConfig.size,
                                        System.nanoTime() / 1_000L,
                                        MediaCodec.BUFFER_FLAG_CODEC_CONFIG,
                                    )
                                    configSent = true
                                } else {
                                    inputBuffer.put(frameToSend.data)
                                    decoder.queueInputBuffer(
                                        inputIndex,
                                        0,
                                        frameToSend.data.size,
                                        System.nanoTime() / 1_000L,
                                        0,
                                    )
                                    pendingFrame = null
                                }
                            } else {
                                decoder.queueInputBuffer(inputIndex, 0, 0, 0L, 0)
                            }
                        }
                    }
                    if (decoder != null) {
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
        lastDetectedDimensions = null
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
        private const val STREAM_ID = 1L
        private const val PAYLOAD_TYPE_VIDEO = 0x01
        private const val FLAG_KEYFRAME = 1 shl 0
        private const val FLAG_CONFIG = 1 shl 1
        private const val FLAG_DISCONTINUITY = 1 shl 3
        private const val NAL_TYPE_SPS = 7
        private const val NAL_TYPE_PPS = 8
    }

    private data class PacketHeader(
        val flags: Int,
        val streamId: Long,
        val frameId: Long,
        val packetId: Int,
        val packetCount: Int,
        val payloadType: Int,
        val payloadLength: Int,
    )

    private data class FramePayload(
        val data: ByteArray,
        val flags: Int,
        val configData: ByteArray?,
    )

    private inner class FrameAssembler(private val logger: Logger) {
        private val frames = object : LinkedHashMap<Long, FrameAssembly>(MAX_IN_FLIGHT_FRAMES, 0.75f, true) {
            override fun removeEldestEntry(eldest: MutableMap.MutableEntry<Long, FrameAssembly>?): Boolean {
                return size > MAX_IN_FLIGHT_FRAMES
            }
        }

        fun onPacket(bytes: ByteArray, length: Int): FramePayload? {
            if (length < HEADER_LENGTH) {
                logger.warn(TAG, "Dropping packet: too small", mapOf("length" to length))
                return null
            }
            val header = parseHeader(bytes) ?: return null
            if (header.payloadType != PAYLOAD_TYPE_VIDEO) {
                logger.warn(
                    TAG,
                    "Dropping packet: unexpected payload type",
                    mapOf("payloadType" to header.payloadType),
                )
                return null
            }
            if (header.streamId != STREAM_ID) {
                logger.warn(
                    TAG,
                    "Dropping packet: unexpected stream id",
                    mapOf("streamId" to header.streamId),
                )
                return null
            }
            if (header.payloadLength <= 0 || header.payloadLength > length - HEADER_LENGTH) {
                logger.warn(
                    TAG,
                    "Dropping packet: invalid payload length",
                    mapOf("payloadLength" to header.payloadLength, "length" to length),
                )
                return null
            }
            val payload = bytes.copyOfRange(HEADER_LENGTH, HEADER_LENGTH + header.payloadLength)
            val assembly = frames.getOrPut(header.frameId) { FrameAssembly(header.packetCount) }
            if (header.packetId >= assembly.packetCount) {
                logger.warn(
                    TAG,
                    "Dropping packet: packetId out of range",
                    mapOf("packetId" to header.packetId, "packetCount" to assembly.packetCount),
                )
                return null
            }
            assembly.flags = assembly.flags or header.flags
            assembly.packets[header.packetId] = payload
            if (assembly.isComplete()) {
                frames.remove(header.frameId)
                val data = assembly.assemble()
                val configData = if (assembly.hasFlag(FLAG_CONFIG)) {
                    extractConfigNalUnits(data)
                } else {
                    null
                }
                return FramePayload(data, assembly.flags, configData)
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
            val streamId = buffer.getInt(8).toLong() and 0xFFFF_FFFFL
            val frameId = buffer.getInt(12).toLong() and 0xFFFF_FFFFL
            val packetId = buffer.getShort(16).toInt() and 0xFFFF
            val packetCount = buffer.getShort(18).toInt() and 0xFFFF
            val payloadType = bytes[20].toInt() and 0xFF
            val payloadLength = buffer.getShort(22).toInt() and 0xFFFF
            if (packetCount == 0) {
                logger.warn(TAG, "Dropping packet: packetCount=0", emptyMap())
                return null
            }
            return PacketHeader(flags, streamId, frameId, packetId, packetCount, payloadType, payloadLength)
        }
    }

    private fun parseDimensionsFromConfig(config: ByteArray): VideoDimensions? {
        val sps = extractNalUnit(config, NAL_TYPE_SPS) ?: return null
        return parseSpsDimensions(sps)
    }

    private fun extractNalUnit(data: ByteArray, targetNalType: Int): ByteArray? {
        var i = 0
        while (i + 3 < data.size) {
            val startCodeLength = when {
                data[i] == 0.toByte() && data[i + 1] == 0.toByte() && data[i + 2] == 1.toByte() -> 3
                i + 4 < data.size && data[i] == 0.toByte() && data[i + 1] == 0.toByte() &&
                    data[i + 2] == 0.toByte() && data[i + 3] == 1.toByte() -> 4
                else -> 0
            }
            if (startCodeLength == 0) {
                i += 1
                continue
            }
            val nalStart = i + startCodeLength
            var nalEnd = nalStart
            while (nalEnd + 3 < data.size && !isStartCode(data, nalEnd)) {
                nalEnd += 1
            }
            if (nalEnd >= data.size) {
                nalEnd = data.size
            }
            val nalType = data[nalStart].toInt() and 0x1F
            if (nalType == targetNalType) {
                return data.copyOfRange(nalStart, nalEnd)
            }
            i = nalEnd
        }
        return null
    }

    private fun isStartCode(data: ByteArray, index: Int): Boolean {
        return (data[index] == 0.toByte() && data[index + 1] == 0.toByte() &&
            data[index + 2] == 1.toByte()) ||
            (index + 3 < data.size && data[index] == 0.toByte() && data[index + 1] == 0.toByte() &&
                data[index + 2] == 0.toByte() && data[index + 3] == 1.toByte())
    }

    private fun parseSpsDimensions(nalUnit: ByteArray): VideoDimensions? {
        if (nalUnit.isEmpty()) return null
        val rbsp = removeEmulationPreventionBytes(nalUnit.copyOfRange(1, nalUnit.size))
        val reader = BitReader(rbsp)
        val profileIdc = reader.readBits(8)
        reader.readBits(8)
        reader.readBits(8)
        reader.readUnsignedExpGolomb()
        var chromaFormatIdc = 1
        if (profileIdc == 100 || profileIdc == 110 || profileIdc == 122 ||
            profileIdc == 244 || profileIdc == 44 || profileIdc == 83 ||
            profileIdc == 86 || profileIdc == 118 || profileIdc == 128 ||
            profileIdc == 138 || profileIdc == 144
        ) {
            chromaFormatIdc = reader.readUnsignedExpGolomb()
            if (chromaFormatIdc == 3) {
                reader.readBit()
            }
            reader.readUnsignedExpGolomb()
            reader.readUnsignedExpGolomb()
            reader.readBit()
            if (reader.readBit() == 1) {
                val scalingListCount = if (chromaFormatIdc == 3) 12 else 8
                repeat(scalingListCount) {
                    if (reader.readBit() == 1) {
                        skipScalingList(reader, if (it < 6) 16 else 64)
                    }
                }
            }
        }
        reader.readUnsignedExpGolomb()
        when (reader.readUnsignedExpGolomb()) {
            0 -> reader.readUnsignedExpGolomb()
            1 -> {
                reader.readBit()
                reader.readSignedExpGolomb()
                reader.readSignedExpGolomb()
                val cycle = reader.readUnsignedExpGolomb()
                repeat(cycle) { reader.readSignedExpGolomb() }
            }
        }
        reader.readUnsignedExpGolomb()
        reader.readBit()
        val picWidthInMbsMinus1 = reader.readUnsignedExpGolomb()
        val picHeightInMapUnitsMinus1 = reader.readUnsignedExpGolomb()
        val frameMbsOnlyFlag = reader.readBit() == 1
        if (!frameMbsOnlyFlag) {
            reader.readBit()
        }
        reader.readBit()
        val frameCroppingFlag = reader.readBit() == 1
        var frameCropLeft = 0
        var frameCropRight = 0
        var frameCropTop = 0
        var frameCropBottom = 0
        if (frameCroppingFlag) {
            frameCropLeft = reader.readUnsignedExpGolomb()
            frameCropRight = reader.readUnsignedExpGolomb()
            frameCropTop = reader.readUnsignedExpGolomb()
            frameCropBottom = reader.readUnsignedExpGolomb()
        }
        var width = (picWidthInMbsMinus1 + 1) * 16
        var height = (picHeightInMapUnitsMinus1 + 1) * 16
        if (!frameMbsOnlyFlag) {
            height *= 2
        }
        val cropUnitX = when (chromaFormatIdc) {
            0 -> 1
            3 -> 1
            else -> 2
        }
        val cropUnitY = when (chromaFormatIdc) {
            0 -> 2 - if (frameMbsOnlyFlag) 1 else 0
            3 -> 2 - if (frameMbsOnlyFlag) 1 else 0
            else -> 2 * (2 - if (frameMbsOnlyFlag) 1 else 0)
        }
        if (frameCroppingFlag) {
            width -= (frameCropLeft + frameCropRight) * cropUnitX
            height -= (frameCropTop + frameCropBottom) * cropUnitY
        }
        if (width <= 0 || height <= 0) return null
        return VideoDimensions(width, height)
    }

    private fun removeEmulationPreventionBytes(data: ByteArray): ByteArray {
        val output = ArrayList<Byte>(data.size)
        var i = 0
        while (i < data.size) {
            if (i + 2 < data.size && data[i] == 0.toByte() && data[i + 1] == 0.toByte() && data[i + 2] == 3.toByte()) {
                output.add(0)
                output.add(0)
                i += 3
            } else {
                output.add(data[i])
                i += 1
            }
        }
        return output.toByteArray()
    }

    private fun skipScalingList(reader: BitReader, size: Int) {
        var lastScale = 8
        var nextScale = 8
        for (i in 0 until size) {
            if (nextScale != 0) {
                val deltaScale = reader.readSignedExpGolomb()
                nextScale = (lastScale + deltaScale + 256) % 256
            }
            lastScale = if (nextScale == 0) lastScale else nextScale
        }
    }

    private class BitReader(private val data: ByteArray) {
        private var byteOffset = 0
        private var bitOffset = 0

        fun readBit(): Int = readBits(1)

        fun readBits(count: Int): Int {
            var remaining = count
            var value = 0
            while (remaining > 0) {
                if (byteOffset >= data.size) {
                    return value
                }
                val currentByte = data[byteOffset].toInt() and 0xFF
                val bitsLeft = 8 - bitOffset
                val bitsToRead = minOf(remaining, bitsLeft)
                val shift = bitsLeft - bitsToRead
                val mask = (0xFF shr (8 - bitsToRead)) shl shift
                value = (value shl bitsToRead) or ((currentByte and mask) shr shift)
                bitOffset += bitsToRead
                if (bitOffset == 8) {
                    bitOffset = 0
                    byteOffset += 1
                }
                remaining -= bitsToRead
            }
            return value
        }

        fun readUnsignedExpGolomb(): Int {
            var zeros = 0
            while (readBit() == 0 && byteOffset < data.size) {
                zeros += 1
            }
            if (zeros == 0) {
                return 0
            }
            val info = readBits(zeros)
            return (1 shl zeros) - 1 + info
        }

        fun readSignedExpGolomb(): Int {
            val value = readUnsignedExpGolomb()
            return if (value % 2 == 0) {
                -(value / 2)
            } else {
                (value + 1) / 2
            }
        }
    }

    private class FrameAssembly(val packetCount: Int) {
        val packets: Array<ByteArray?> = arrayOfNulls(packetCount)
        var flags: Int = 0

        fun isComplete(): Boolean = packets.all { it != null }
        fun hasFlag(flag: Int): Boolean = (flags and flag) != 0

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

    private fun extractConfigNalUnits(data: ByteArray): ByteArray? {
        val configUnits = mutableListOf<ByteArray>()
        var offset = 0
        while (true) {
            val start = findStartCode(data, offset) ?: break
            val nalStart = start.codeStart + start.codeLength
            if (nalStart >= data.size) {
                break
            }
            val nalType = data[nalStart].toInt() and 0x1F
            val next = findStartCode(data, nalStart)
            val nalEnd = next?.codeStart ?: data.size
            if (nalType == NAL_TYPE_SPS || nalType == NAL_TYPE_PPS) {
                configUnits.add(data.copyOfRange(start.codeStart, nalEnd))
            }
            offset = nalStart + 1
        }
        if (configUnits.isEmpty()) {
            return null
        }
        val totalSize = configUnits.sumOf { it.size }
        val output = ByteArray(totalSize)
        var offsetOut = 0
        for (unit in configUnits) {
            System.arraycopy(unit, 0, output, offsetOut, unit.size)
            offsetOut += unit.size
        }
        return output
    }

    private data class StartCode(val codeStart: Int, val codeLength: Int)

    private fun findStartCode(data: ByteArray, offset: Int): StartCode? {
        var i = offset
        while (i + 3 < data.size) {
            if (data[i] == 0.toByte() && data[i + 1] == 0.toByte()) {
                if (data[i + 2] == 1.toByte()) {
                    return StartCode(codeStart = i, codeLength = 3)
                }
                if (data[i + 2] == 0.toByte() && data[i + 3] == 1.toByte()) {
                    return StartCode(codeStart = i, codeLength = 4)
                }
            }
            i += 1
        }
        return null
    }
}
