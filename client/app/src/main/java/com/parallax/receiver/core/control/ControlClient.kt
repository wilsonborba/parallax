package com.parallax.receiver.core.control

import com.parallax.receiver.core.logging.Logger
import com.parallax.receiver.core.logging.LoggerProvider
import java.io.EOFException
import java.io.InputStream
import java.io.OutputStream
import java.net.InetSocketAddress
import java.net.Socket
import javax.crypto.Mac
import javax.crypto.SecretKeyFactory
import javax.crypto.spec.PBEKeySpec
import javax.crypto.spec.SecretKeySpec

class ControlClient(
    private val pairingToken: String,
    private val logger: Logger = LoggerProvider.logger,
    private val connectTimeoutMillis: Int = DEFAULT_TIMEOUT_MS,
    private val readTimeoutMillis: Int = DEFAULT_TIMEOUT_MS,
) {
    data class DisplayInfo(
        val id: String,
        val name: String,
        val primary: Boolean,
        val connected: Boolean,
        val width: Int,
        val height: Int,
        val x: Int,
        val y: Int,
    )

    data class VirtualDisplayInfo(
        val id: String,
        val enabled: Boolean,
        val width: Int,
        val height: Int,
        val x: Int,
        val y: Int,
    )

    data class DisplaysSnapshot(
        val physical: List<DisplayInfo>,
        val virtual: List<VirtualDisplayInfo>,
    )

    data class StreamInfo(
        val streamId: Int,
        val displayId: String,
        val bindAddr: String,
        val targetAddr: String,
        val preferVaapi: Boolean,
        val running: Boolean,
        val width: Int,
        val height: Int,
        val fps: Float,
        val bitrateKbps: Int,
    )

    fun openSession(host: String, port: Int, streamPort: Int? = null): ControlSession {
        val socket = Socket()
        socket.tcpNoDelay = true
        socket.soTimeout = readTimeoutMillis
        socket.connect(InetSocketAddress(host, port), connectTimeoutMillis)
        logger.info(
            TAG,
            "Control socket connected",
            mapOf("host" to host, "port" to port, "streamPort" to streamPort),
        )
        val session = ControlSession(socket, pairingToken, streamPort, logger)
        return try {
            session.performHandshake()
            session
        } catch (e: Exception) {
            logger.error(
                TAG,
                "Control handshake failed",
                mapOf("host" to host, "port" to port, "streamPort" to streamPort, "error" to e.message),
            )
            socket.close()
            throw e
        }
    }

    class ControlSession(
        private val socket: Socket,
        private val pairingToken: String,
        private val streamPort: Int?,
        private val logger: Logger,
    ) {
        private val input: InputStream = socket.getInputStream()
        private val output: OutputStream = socket.getOutputStream()

        fun startStream(streamId: Int = 1) {
            writeFrame(MessageType.StartStream, "stream_id=$streamId".toByteArray())
            val response = readFrame()
            when (response.messageType) {
                MessageType.StreamStarted -> Unit
                MessageType.Error -> throw IllegalStateException(response.payload.decodeToString())
                else -> throw IllegalStateException("Unexpected response to start stream: ${response.messageType}")
            }
        }

        fun stopStream(streamId: Int = 1) {
            writeFrame(MessageType.StopStream, "stream_id=$streamId".toByteArray())
            val response = readFrame()
            when (response.messageType) {
                MessageType.StreamStopped -> Unit
                MessageType.Error -> throw IllegalStateException(response.payload.decodeToString())
                else -> throw IllegalStateException("Unexpected response to stop stream: ${response.messageType}")
            }
        }

        fun listDisplays(): DisplaysSnapshot {
            writeFrame(MessageType.ListDisplays, ByteArray(0))
            val response = readFrame()
            return when (response.messageType) {
                MessageType.Displays -> parseDisplaysPayload(response.payload.decodeToString())
                MessageType.Error -> throw IllegalStateException(response.payload.decodeToString())
                else -> throw IllegalStateException("Unexpected response to list displays: ${response.messageType}")
            }
        }

        fun addVirtualDisplay(id: String, width: Int, height: Int, x: Int, y: Int) {
            val payload = "$id,$width,$height,$x,$y".toByteArray()
            writeFrame(MessageType.AddVirtualDisplay, payload)
            val response = readFrame()
            when (response.messageType) {
                MessageType.DisplayOpAck -> Unit
                MessageType.Error -> throw IllegalStateException(response.payload.decodeToString())
                else -> throw IllegalStateException("Unexpected response to add virtual display: ${response.messageType}")
            }
        }

        fun removeVirtualDisplay(id: String) {
            writeFrame(MessageType.RemoveVirtualDisplay, id.toByteArray())
            val response = readFrame()
            when (response.messageType) {
                MessageType.DisplayOpAck -> Unit
                MessageType.Error -> throw IllegalStateException(response.payload.decodeToString())
                else -> throw IllegalStateException("Unexpected response to remove virtual display: ${response.messageType}")
            }
        }

        fun listStreams(): List<StreamInfo> {
            writeFrame(MessageType.ListStreams, ByteArray(0))
            val response = readFrame()
            return when (response.messageType) {
                MessageType.Streams -> parseStreamsPayload(response.payload.decodeToString())
                MessageType.Error -> throw IllegalStateException(response.payload.decodeToString())
                else -> throw IllegalStateException("Unexpected response to list streams: ${response.messageType}")
            }
        }

        fun setStreamConfig(
            streamId: Int,
            displayId: String? = null,
            bindAddr: String? = null,
            targetAddr: String? = null,
            preferVaapi: Boolean? = null,
        ) {
            val lines = mutableListOf("stream_id=$streamId")
            if (!displayId.isNullOrBlank()) lines.add("display=$displayId")
            if (!bindAddr.isNullOrBlank()) lines.add("bind_addr=$bindAddr")
            if (!targetAddr.isNullOrBlank()) lines.add("target_addr=$targetAddr")
            if (preferVaapi != null) lines.add("prefer_vaapi=$preferVaapi")
            writeFrame(MessageType.SetStreamConfig, lines.joinToString("\n").toByteArray())
            val response = readFrame()
            when (response.messageType) {
                MessageType.StreamConfigAck -> Unit
                MessageType.Error -> throw IllegalStateException(response.payload.decodeToString())
                else -> throw IllegalStateException("Unexpected response to set stream config: ${response.messageType}")
            }
        }

        fun close() {
            socket.close()
        }

        internal fun performHandshake() {
            logger.info(TAG, "Control handshake start", mapOf("streamPort" to streamPort))
            writeFrame(MessageType.Hello, ByteArray(0))
            val helloAck = readFrame()
            if (helloAck.messageType != MessageType.HelloAck) {
                logger.error(
                    TAG,
                    "Unexpected hello ack",
                    mapOf("messageType" to helloAck.messageType.name),
                )
                throw IllegalStateException("Expected hello ack, got ${helloAck.messageType}")
            }

            val pairingPayload = if (streamPort == null) {
                pairingToken.toByteArray()
            } else {
                "$pairingToken|$streamPort".toByteArray()
            }
            writeFrame(MessageType.PairRequest, pairingPayload)
            val pairingResponse = readFrame()
            when (pairingResponse.messageType) {
                MessageType.AuthChallenge -> {
                    logger.info(TAG, "Received auth challenge", mapOf("nonceLen" to pairingResponse.payload.size))
                    val nonce = pairingResponse.payload
                    val sessionKey = ControlClient.deriveSessionKey(pairingToken, nonce)
                    val hmac = ControlClient.hmacSha256(sessionKey, nonce)
                    writeFrame(MessageType.AuthResponse, hmac)
                    val authResponse = readFrame()
                    when (authResponse.messageType) {
                        MessageType.PairAccept -> logger.info(TAG, "Pairing accepted")
                        MessageType.PairReject -> throw IllegalStateException("Pairing rejected")
                        MessageType.Error -> throw IllegalStateException(authResponse.payload.decodeToString())
                        else -> throw IllegalStateException("Unexpected auth response: ${authResponse.messageType}")
                    }
                }
                MessageType.PairReject -> {
                    logger.warn(TAG, "Pairing rejected before auth")
                    throw IllegalStateException("Pairing rejected")
                }
                MessageType.Error -> throw IllegalStateException(pairingResponse.payload.decodeToString())
                else -> throw IllegalStateException("Unexpected pairing response: ${pairingResponse.messageType}")
            }
        }

        private fun readFrame(): Frame {
            val header = input.readExact(HEADER_LEN)
            val version = header[0]
            if (version != PROTOCOL_VERSION) {
                throw IllegalStateException("Unsupported protocol version: $version")
            }
            val messageType = MessageType.fromByte(header[1])
            val payloadLen = ((header[2].toInt() and 0xff) shl 8) or (header[3].toInt() and 0xff)
            val payload = if (payloadLen > 0) input.readExact(payloadLen) else ByteArray(0)
            return Frame(messageType, payload)
        }

        private fun writeFrame(messageType: MessageType, payload: ByteArray) {
            if (payload.size > MAX_PAYLOAD_LEN) {
                throw IllegalArgumentException("Payload too large: ${payload.size}")
            }
            val header = byteArrayOf(
                PROTOCOL_VERSION,
                messageType.value,
                (payload.size shr 8).toByte(),
                payload.size.toByte(),
            )
            output.write(header)
            if (payload.isNotEmpty()) {
                output.write(payload)
            }
            output.flush()
        }

        private fun parseDisplaysPayload(raw: String): DisplaysSnapshot {
            val physical = mutableListOf<DisplayInfo>()
            val virtual = mutableListOf<VirtualDisplayInfo>()
            var section: String? = null

            raw.lineSequence().forEach { lineRaw ->
                val line = lineRaw.trim()
                if (line.isEmpty() || line.startsWith("protocol=")) return@forEach
                if (line == "physical:") {
                    section = "physical"
                    return@forEach
                }
                if (line == "virtual:") {
                    section = "virtual"
                    return@forEach
                }

                val parts = line.split(",")
                when (section) {
                    "physical" -> if (parts.size >= 8) {
                        physical.add(
                            DisplayInfo(
                                id = parts[0],
                                name = parts[1],
                                primary = parts[2].toBooleanStrictOrNull() ?: false,
                                connected = parts[3].toBooleanStrictOrNull() ?: false,
                                width = parts[4].toIntOrNull() ?: 0,
                                height = parts[5].toIntOrNull() ?: 0,
                                x = parts[6].toIntOrNull() ?: 0,
                                y = parts[7].toIntOrNull() ?: 0,
                            ),
                        )
                    }
                    "virtual" -> if (parts.size >= 6) {
                        virtual.add(
                            VirtualDisplayInfo(
                                id = parts[0],
                                enabled = parts[1].toBooleanStrictOrNull() ?: false,
                                width = parts[2].toIntOrNull() ?: 0,
                                height = parts[3].toIntOrNull() ?: 0,
                                x = parts[4].toIntOrNull() ?: 0,
                                y = parts[5].toIntOrNull() ?: 0,
                            ),
                        )
                    }
                }
            }

            return DisplaysSnapshot(physical = physical, virtual = virtual)
        }

        private fun parseStreamsPayload(raw: String): List<StreamInfo> {
            val streams = mutableListOf<StreamInfo>()
            var inStreams = false

            raw.lineSequence().forEach { lineRaw ->
                val line = lineRaw.trim()
                if (line.isEmpty() || line.startsWith("protocol=")) return@forEach
                if (line == "streams:") {
                    inStreams = true
                    return@forEach
                }
                if (!inStreams) return@forEach

                val parts = line.split(",")
                if (parts.size < 6) return@forEach

                streams.add(
                    StreamInfo(
                        streamId = parts[0].toIntOrNull() ?: return@forEach,
                        displayId = parts[1],
                        bindAddr = parts[2],
                        targetAddr = parts[3],
                        preferVaapi = parts[4].toBooleanStrictOrNull() ?: false,
                        running = parts[5].toBooleanStrictOrNull() ?: false,
                        width = parts.getOrNull(6)?.toIntOrNull() ?: 0,
                        height = parts.getOrNull(7)?.toIntOrNull() ?: 0,
                        fps = parts.getOrNull(8)?.toFloatOrNull() ?: 0f,
                        bitrateKbps = parts.getOrNull(9)?.toIntOrNull() ?: 0,
                    ),
                )
            }

            return streams
        }
    }

    private companion object {
        private const val TAG = "ControlClient"
        private const val PROTOCOL_VERSION: Byte = 1
        private const val HEADER_LEN = 4
        private const val MAX_PAYLOAD_LEN = 0xFFFF
        private const val DEFAULT_TIMEOUT_MS = 5_000
        private const val PBKDF2_ITERATIONS = 100_000
        private const val KEY_LEN_BYTES = 32
        private const val HKDF_INFO = "parallax-control-auth"
        private val PBKDF2_SALT = "parallax-control".toByteArray()
        private fun deriveSessionKey(pairingToken: String, nonce: ByteArray): ByteArray {
            val masterKey = pbkdf2Sha256(pairingToken.toCharArray(), PBKDF2_SALT, PBKDF2_ITERATIONS, KEY_LEN_BYTES)
            return hkdfSha256(masterKey, nonce, HKDF_INFO.toByteArray(), KEY_LEN_BYTES)
        }

        private fun pbkdf2Sha256(password: CharArray, salt: ByteArray, iterations: Int, keyLenBytes: Int): ByteArray {
            val spec = PBEKeySpec(password, salt, iterations, keyLenBytes * 8)
            val factory = SecretKeyFactory.getInstance("PBKDF2WithHmacSHA256")
            return factory.generateSecret(spec).encoded
        }

        private fun hkdfSha256(ikm: ByteArray, salt: ByteArray, info: ByteArray, keyLenBytes: Int): ByteArray {
            val prk = hmacSha256(salt, ikm)
            val okm = hmacSha256(prk, info + 0x01.toByte())
            return okm.copyOfRange(0, keyLenBytes)
        }

        private fun hmacSha256(key: ByteArray, data: ByteArray): ByteArray {
            val mac = Mac.getInstance("HmacSHA256")
            mac.init(SecretKeySpec(key, "HmacSHA256"))
            return mac.doFinal(data)
        }
    }

    private data class Frame(
        val messageType: MessageType,
        val payload: ByteArray,
    )

    private enum class MessageType(val value: Byte) {
        Hello(0x01),
        HelloAck(0x02),
        PairRequest(0x03),
        PairAccept(0x04),
        PairReject(0x05),
        AuthChallenge(0x06),
        AuthResponse(0x07),
        StartStream(0x10),
        StopStream(0x11),
        StreamStarted(0x12),
        StreamStopped(0x13),
        ListStreams(0x14),
        Streams(0x15),
        SetStreamConfig(0x16),
        StreamConfigAck(0x17),
        Ping(0x20),
        Pong(0x21),
        ListDisplays(0x30),
        Displays(0x31),
        AddVirtualDisplay(0x32),
        RemoveVirtualDisplay(0x33),
        DisplayOpAck(0x34),
        Error(0x7f),
        ;

        companion object {
            fun fromByte(value: Byte): MessageType {
                return entries.firstOrNull { it.value == value }
                    ?: throw IllegalArgumentException("Unknown message type: $value")
            }
        }
    }
}

private fun InputStream.readExact(length: Int): ByteArray {
    val buffer = ByteArray(length)
    var offset = 0
    while (offset < length) {
        val read = read(buffer, offset, length - offset)
        if (read == -1) {
            throw EOFException("Unexpected end of stream")
        }
        offset += read
    }
    return buffer
}
