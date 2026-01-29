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
    private val logger: Logger = LoggerProvider.logger,
    private val connectTimeoutMillis: Int = DEFAULT_TIMEOUT_MS,
    private val readTimeoutMillis: Int = DEFAULT_TIMEOUT_MS,
) {
    fun openSession(host: String, port: Int, pairingToken: String, streamPort: Int): ControlSession {
        val socket = Socket()
        socket.tcpNoDelay = true
        socket.soTimeout = readTimeoutMillis
        socket.connect(InetSocketAddress(host, port), connectTimeoutMillis)
        val session = ControlSession(socket, pairingToken, streamPort, logger)
        return try {
            session.performHandshake()
            session
        } catch (e: Exception) {
            socket.close()
            throw e
        }
    }

    class ControlSession(
        private val socket: Socket,
        private val pairingToken: String,
        private val streamPort: Int,
        private val logger: Logger,
    ) {
        private val input: InputStream = socket.getInputStream()
        private val output: OutputStream = socket.getOutputStream()

        fun startStream() {
            writeFrame(MessageType.StartStream, ByteArray(0))
            val response = readFrame()
            when (response.messageType) {
                MessageType.StreamStarted -> Unit
                MessageType.Error -> throw IllegalStateException(response.payload.decodeToString())
                else -> throw IllegalStateException("Unexpected response to start stream: ${response.messageType}")
            }
        }

        fun stopStream() {
            writeFrame(MessageType.StopStream, ByteArray(0))
            val response = readFrame()
            when (response.messageType) {
                MessageType.StreamStopped -> Unit
                MessageType.Error -> throw IllegalStateException(response.payload.decodeToString())
                else -> throw IllegalStateException("Unexpected response to stop stream: ${response.messageType}")
            }
        }

        fun close() {
            socket.close()
        }

        internal fun performHandshake() {
            writeFrame(MessageType.Hello, ByteArray(0))
            val helloAck = readFrame()
            if (helloAck.messageType != MessageType.HelloAck) {
                throw IllegalStateException("Expected hello ack, got ${helloAck.messageType}")
            }

            val pairingPayload = "$pairingToken|$streamPort".toByteArray()
            writeFrame(MessageType.PairRequest, pairingPayload)
            val pairingResponse = readFrame()
            when (pairingResponse.messageType) {
                MessageType.AuthChallenge -> {
                    val nonce = pairingResponse.payload
                    val sessionKey = ControlClient.deriveSessionKey(pairingToken, nonce)
                    val hmac = ControlClient.hmacSha256(sessionKey, nonce)
                    writeFrame(MessageType.AuthResponse, hmac)
                    val authResponse = readFrame()
                    when (authResponse.messageType) {
                        MessageType.PairAccept -> Unit
                        MessageType.PairReject -> throw IllegalStateException("Pairing rejected")
                        MessageType.Error -> throw IllegalStateException(authResponse.payload.decodeToString())
                        else -> throw IllegalStateException("Unexpected auth response: ${authResponse.messageType}")
                    }
                }
                MessageType.PairReject -> throw IllegalStateException("Pairing rejected")
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
    }

    private companion object {
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
        Ping(0x20),
        Pong(0x21),
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
