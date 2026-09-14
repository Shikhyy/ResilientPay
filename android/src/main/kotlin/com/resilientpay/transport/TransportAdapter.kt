package com.resilientpay.transport

/**
 * TransportAdapter moves opaque bytes between devices or between a device and the backend.
 * Architecture Rule: Transport adapters MUST NOT contain any payment authorization logic.
 * They are purely pipes for bytes.
 */
interface TransportAdapter {
    fun send(payload: ByteArray): TransportResult
    fun receive(): TransportResult
}
