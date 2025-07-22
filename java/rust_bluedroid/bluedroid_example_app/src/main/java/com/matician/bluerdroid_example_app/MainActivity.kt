package com.matician.bluerdroid_example_app

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import com.matician.bluerdroid_example_app.ui.theme.JavaTheme
import com.maticrobots.rust_android_utilities.RustArcBoxDynAny


class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            JavaTheme {
                Scaffold(modifier = Modifier.fillMaxSize()) { innerPadding ->
                    BluetoothUI(
                        modifier = Modifier.padding(innerPadding)
                    )
                }
            }
        }
    }
}

@Composable
fun BluetoothUI(modifier: Modifier = Modifier) {
    // State to hold the text from the TextField
    var macAddress by remember { mutableStateOf("") }

    var device: RustArcBoxDynAny? = null

    // Column to arrange elements vertically
    Column(modifier = modifier.padding(75.dp)) {
        // Text field for user input
        OutlinedTextField(
            value = macAddress,
            onValueChange = { macAddress = it },
            label = { Text("Enter Mac Address") }
            // Removed modifiers for brevity
        )

        // Row to arrange buttons horizontally
        Row {
            // Button 1: Processes the text from the TextField
            Button(onClick = { device = onConnectToDevice(macAddress) }) {
                Text("Scan")
            }

            // Button 2: Performs an independent action
            Button(onClick = { test_gatt(device) }) {
                Text("Test Gatt")
            }
        }
    }
}
// 8C:B8:7E:C9:92:02

fun onConnectToDevice(mac_address: String): RustArcBoxDynAny? {
    return RustInitialization.rust_bluetooth_search(mac_address);
}

fun test_gatt(device: RustArcBoxDynAny?) {
    RustInitialization.test_gatt(device);
}

@Preview(showBackground = true)
@Composable
fun BluetoothUIPreview() {
    JavaTheme {
        BluetoothUI()
    }
}
