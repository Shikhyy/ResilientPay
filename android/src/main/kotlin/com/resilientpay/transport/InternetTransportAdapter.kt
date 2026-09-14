package com.resilientpay.transport

class InternetTransportAdapter : TransportAdapter {
    override fun send(payload: ByteArray): TransportResult {
        return TransportResult.Success(ByteArray(0))
    }

    override fun receive(): TransportResult {
        return TransportResult.Failure("Internet receive not implemented", false)
    }
}
