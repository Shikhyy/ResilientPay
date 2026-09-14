package com.resilientpay.transport

sealed class TransportResult {
    data class Success(val payload: ByteArray) : TransportResult() {
        override fun equals(other: Any?): Boolean {
            if (this === other) return true
            if (javaClass != other?.javaClass) return false
            other as Success
            return payload.contentEquals(other.payload)
        }
        override fun hashCode(): Int {
            return payload.contentHashCode()
        }
    }
    data class Failure(val reason: String, val isRecoverable: Boolean) : TransportResult()
}
