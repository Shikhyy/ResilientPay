package com.resilientpay.transport

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotEquals
import kotlin.test.assertTrue

class TransportResultTest {

    @Test
    fun testSuccessEquality() {
        val payload1 = byteArrayOf(1, 2, 3)
        val payload2 = byteArrayOf(1, 2, 3)
        val payload3 = byteArrayOf(1, 2, 4)

        val success1 = TransportResult.Success(payload1)
        val success2 = TransportResult.Success(payload2)
        val success3 = TransportResult.Success(payload3)

        assertEquals(success1, success2, "Success objects with same payload bytes should be equal")
        assertNotEquals(success1, success3, "Success objects with different payload bytes should not be equal")
    }

    @Test
    fun testFailureBehavior() {
        val failure = TransportResult.Failure("Connection lost", true)
        assertEquals("Connection lost", failure.reason)
        assertTrue(failure.isRecoverable)
    }
}
