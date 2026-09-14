#!/bin/bash
TARGET_DIR="/Users/shikhar/resilientpay-android/app/src/main/kotlin/com/resilientpay/app"
mkdir -p "$TARGET_DIR/ui/dashboard"
mkdir -p "$TARGET_DIR/ui/send"
mkdir -p "$TARGET_DIR/ui/receive"

cat << 'INNER_EOF' > "$TARGET_DIR/ui/dashboard/DashboardViewModel.kt"
package com.resilientpay.app.ui.dashboard

import androidx.lifecycle.ViewModel
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import javax.inject.Inject

@HiltViewModel
class DashboardViewModel @Inject constructor() : ViewModel() {
    private val _offlineBudget = MutableStateFlow(1500.0)
    val offlineBudget: StateFlow<Double> = _offlineBudget

    private val _offlineLimit = MutableStateFlow(5000.0)
    val offlineLimit: StateFlow<Double> = _offlineLimit
}
INNER_EOF

cat << 'INNER_EOF' > "$TARGET_DIR/ui/dashboard/DashboardScreen.kt"
package com.resilientpay.app.ui.dashboard

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel

@Composable
fun DashboardScreen(viewModel: DashboardViewModel = viewModel()) {
    val budget by viewModel.offlineBudget.collectAsState()
    val limit by viewModel.offlineLimit.collectAsState()
    
    Column(modifier = Modifier.padding(16.dp)) {
        Text(text = "Dashboard", style = MaterialTheme.typography.headlineMedium)
        Spacer(modifier = Modifier.height(16.dp))
        Text(text = "Offline Budget: \$${budget}")
        Spacer(modifier = Modifier.height(8.dp))
        Text(text = "Offline Limit: \$${limit}")
    }
}
INNER_EOF

cat << 'INNER_EOF' > "$TARGET_DIR/ui/send/SendViewModel.kt"
package com.resilientpay.app.ui.send

import androidx.lifecycle.ViewModel
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import javax.inject.Inject

@HiltViewModel
class SendViewModel @Inject constructor() : ViewModel() {
    private val _payload = MutableStateFlow("")
    val payload: StateFlow<String> = _payload

    fun generatePayload(amount: Double) {
        _payload.value = "PAYLOAD_QR_NFC_\$amount"
    }
}
INNER_EOF

cat << 'INNER_EOF' > "$TARGET_DIR/ui/send/SendPaymentScreen.kt"
package com.resilientpay.app.ui.send

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel

@Composable
fun SendPaymentScreen(viewModel: SendViewModel = viewModel()) {
    var amountText by remember { mutableStateOf("") }
    val payload by viewModel.payload.collectAsState()
    
    Column(modifier = Modifier.padding(16.dp)) {
        Text(text = "Send Payment", style = MaterialTheme.typography.headlineMedium)
        Spacer(modifier = Modifier.height(16.dp))
        OutlinedTextField(
            value = amountText,
            onValueChange = { amountText = it },
            label = { Text("Amount") }
        )
        Spacer(modifier = Modifier.height(16.dp))
        Button(onClick = { viewModel.generatePayload(amountText.toDoubleOrNull() ?: 0.0) }) {
            Text("Generate QR/NFC Payload")
        }
        if (payload.isNotEmpty()) {
            Spacer(modifier = Modifier.height(16.dp))
            Text(text = "Generated Payload: \$payload")
        }
    }
}
INNER_EOF

cat << 'INNER_EOF' > "$TARGET_DIR/ui/receive/ReceiveViewModel.kt"
package com.resilientpay.app.ui.receive

import androidx.lifecycle.ViewModel
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import javax.inject.Inject

@HiltViewModel
class ReceiveViewModel @Inject constructor() : ViewModel() {
    private val _receivedMessage = MutableStateFlow("Waiting for payload...")
    val receivedMessage: StateFlow<String> = _receivedMessage

    fun receivePayload(payload: String) {
        _receivedMessage.value = "Received: \$payload"
    }
}
INNER_EOF

cat << 'INNER_EOF' > "$TARGET_DIR/ui/receive/ReceivePaymentScreen.kt"
package com.resilientpay.app.ui.receive

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel

@Composable
fun ReceivePaymentScreen(viewModel: ReceiveViewModel = viewModel()) {
    val message by viewModel.receivedMessage.collectAsState()
    
    Column(modifier = Modifier.padding(16.dp)) {
        Text(text = "Receive Payment", style = MaterialTheme.typography.headlineMedium)
        Spacer(modifier = Modifier.height(16.dp))
        Text(text = message)
        Spacer(modifier = Modifier.height(16.dp))
        Button(onClick = { viewModel.receivePayload("DUMMY_PAYLOAD_SCANNED") }) {
            Text("Simulate Scan")
        }
    }
}
INNER_EOF

chmod +x /Users/shikhar/resilientpay/create_ui.sh
/Users/shikhar/resilientpay/create_ui.sh
