package com.resilientpay.sdk

/**
 * Result of a transport transmission.
 */
sealed class TransportResult {
    data class Success(val receiptBytes: ByteArray) : TransportResult()
    data class Error(val reason: String) : TransportResult()
}

/**
 * TransportAdapter moves protocol objects and does NOT decide payment validity.
 * Transport logic is completely isolated from the core SDK and state machine.
 */
interface TransportAdapter {
    /**
     * Sends the transaction envelope bytes to the merchant or backend.
     */
    fun send(txBytes: ByteArray): TransportResult
}

/**
 * Internet fallback adapter.
 */
class InternetAdapter : TransportAdapter {
    override fun send(txBytes: ByteArray): TransportResult {
        // TODO: Implement HTTP POST
        return TransportResult.Error("not implemented")
    }
}

/**
 * NFC transport adapter.
 */
class NfcAdapter : TransportAdapter {
    override fun send(txBytes: ByteArray): TransportResult {
        // TODO: Implement ISO-DEP / HCE
        return TransportResult.Error("not implemented")
    }
}

/**
 * Bluetooth Low Energy (BLE) transport adapter.
 */
class BleAdapter : TransportAdapter {
    override fun send(txBytes: ByteArray): TransportResult {
        // TODO: Implement BLE GATT
        return TransportResult.Error("not implemented")
    }
}

/**
 * The core ResilientPay Android protocol entrypoint.
 * Receives the transport adapter to enforce the architectural boundary.
 */
class ResilientPayClient(private val transport: TransportAdapter) {
    
    /**
     * Submits a signed transaction using the configured transport.
     */
    fun submitTransaction(txBytes: ByteArray): TransportResult {
        return transport.send(txBytes)
    }
}
