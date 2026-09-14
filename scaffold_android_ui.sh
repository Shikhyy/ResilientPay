#!/bin/bash
TARGET_DIR="/Users/shikhar/resilientpay-android/app/src/main/kotlin/com/resilientpay/app"
mkdir -p "$TARGET_DIR/ui/dashboard"
mkdir -p "$TARGET_DIR/ui/send"
mkdir -p "$TARGET_DIR/ui/receive"

cat << 'INNER_EOF' > "$TARGET_DIR/ui/theme/Theme.kt"
package com.resilientpay.app.ui.theme

import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp

val DarkColors = darkColorScheme(
    primary = Color(0xFF1E88E5),
    background = Color(0xFF121212),
    surface = Color(0xFF1E1E1E)
)

val Shapes = Shapes(
    small = androidx.compose.foundation.shape.RoundedCornerShape(2.dp),
    medium = androidx.compose.foundation.shape.RoundedCornerShape(2.dp),
    large = androidx.compose.foundation.shape.RoundedCornerShape(0.dp)
)

@Composable
fun ResilientPayTheme(content: @Composable () -> Unit) {
    MaterialTheme(
        colorScheme = DarkColors,
        shapes = Shapes,
        content = content
    )
}
INNER_EOF

cat << 'INNER_EOF' > "$TARGET_DIR/ui/dashboard/DashboardViewModel.kt"
package com.resilientpay.app.ui.dashboard

import androidx.lifecycle.ViewModel
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import javax.inject.Inject

@HiltViewModel
class DashboardViewModel @Inject constructor() : ViewModel() {
    private val _connectivityState = MutableStateFlow("OFFLINE (BLE Available)")
    val connectivityState: StateFlow<String> = _connectivityState

    private val _pendingCount = MutableStateFlow(2)
    val pendingCount: StateFlow<Int> = _pendingCount
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
    val connectivity by viewModel.connectivityState.collectAsState()
    val pending by viewModel.pendingCount.collectAsState()
    
    Column(modifier = Modifier.padding(16.dp)) {
        Text(text = "System Status", style = MaterialTheme.typography.titleLarge)
        Spacer(modifier = Modifier.height(16.dp))
        Text(text = "Connectivity: \$connectivity")
        Spacer(modifier = Modifier.height(8.dp))
        Text(text = "Pending Reconciliation: \$pending transactions")
        Spacer(modifier = Modifier.height(24.dp))
        Button(
            onClick = { },
            shape = MaterialTheme.shapes.small
        ) {
            Text("Initiate Payment")
        }
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

    fun generatePayload(amount: String, merchant: String) {
        _payload.value = "TX: \$amount to \$merchant [NFC READY]"
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
    var merchantText by remember { mutableStateOf("") }
    val payload by viewModel.payload.collectAsState()
    
    Column(modifier = Modifier.padding(16.dp)) {
        Text(text = "Authorization Request", style = MaterialTheme.typography.titleLarge)
        Spacer(modifier = Modifier.height(16.dp))
        OutlinedTextField(
            value = amountText,
            onValueChange = { amountText = it },
            label = { Text("Amount (Minor Units)") }
        )
        Spacer(modifier = Modifier.height(8.dp))
        OutlinedTextField(
            value = merchantText,
            onValueChange = { merchantText = it },
            label = { Text("Merchant Identity") }
        )
        Spacer(modifier = Modifier.height(16.dp))
        Button(
            onClick = { viewModel.generatePayload(amountText, merchantText) },
            shape = MaterialTheme.shapes.small
        ) {
            Text("Generate NFC Payload")
        }
        if (payload.isNotEmpty()) {
            Spacer(modifier = Modifier.height(16.dp))
            Text(text = payload, style = MaterialTheme.typography.bodyMedium)
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
    private val _status = MutableStateFlow("WAITING FOR CONNECTIVITY")
    val status: StateFlow<String> = _status

    fun receivePayload() {
        _status.value = "AUTHORIZED LOCALLY\nStored securely. Reconciliation is pending."
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
    val status by viewModel.status.collectAsState()
    
    Column(modifier = Modifier.padding(16.dp)) {
        Text(text = "Local Authorization Result", style = MaterialTheme.typography.titleLarge)
        Spacer(modifier = Modifier.height(16.dp))
        Text(text = "Status: \$status", style = MaterialTheme.typography.bodyMedium)
        Spacer(modifier = Modifier.height(24.dp))
        Button(
            onClick = { viewModel.receivePayload() },
            shape = MaterialTheme.shapes.small
        ) {
            Text("Simulate Receive")
        }
    }
}
INNER_EOF

chmod +x /Users/shikhar/resilientpay/scaffold_android_ui.sh
/Users/shikhar/resilientpay/scaffold_android_ui.sh
