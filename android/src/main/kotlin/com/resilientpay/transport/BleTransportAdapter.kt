package com.resilientpay.transport

class BleTransportAdapter : TransportAdapter {
    override fun send(payload: ByteArray): TransportResult {
        return TransportResult.Success(ByteArray(0))
    }

    override fun receive(): TransportResult {
        return TransportResult.Failure("BLE receive not implemented", false)
    }
}
