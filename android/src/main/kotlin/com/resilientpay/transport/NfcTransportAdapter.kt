package com.resilientpay.transport

class NfcTransportAdapter : TransportAdapter {
    override fun send(payload: ByteArray): TransportResult {
        // Stub implementation
        return TransportResult.Success(ByteArray(0))
    }

    override fun receive(): TransportResult {
        // Stub implementation
        return TransportResult.Failure("NFC receive not implemented", false)
    }
}
