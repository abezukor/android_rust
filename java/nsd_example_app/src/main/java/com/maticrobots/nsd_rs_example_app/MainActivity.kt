package com.maticrobots.nsd_rs_example_app

import android.os.Bundle
import android.util.Log
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.Close
import androidx.compose.material3.Icon
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import com.maticrobots.nsd_rs_example_app.ui.theme.Example_appTheme
import com.maticrobots.rust_android_utilities.RustArcBoxDynAny

class MainActivity : ComponentActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        RustNSDExample.init_logging();

        super.onCreate(savedInstanceState)
        //enableEdgeToEdge()
        setContent {
            Example_appTheme {
                Scaffold(modifier = Modifier.fillMaxSize()) { innerPadding ->
                    DoDiscovery(
                        modifier = Modifier
                            .padding(innerPadding)
                            .padding(top = 150.dp)
                    )
                }
            }
        }
    }

    @Composable
    fun DoDiscovery(modifier: Modifier = Modifier) {
        var manager: RustArcBoxDynAny? by remember { mutableStateOf(null) }

        Switch(
            modifier = modifier,
            checked = manager != null, onCheckedChange = {
                Log.i("MAIN", "Switch toggled " + it.toString());
                if (it) {
                    manager = RustNSDExample.start_discovery();
                } else {
                    manager?.close();
                    manager = null;
                }

                // Makes diagnosing destructors much easier
                System.gc();
            },
            enabled = true, thumbContent = {
                if (manager != null) {
                    Icon(
                        imageVector = Icons.Filled.Check,
                        contentDescription = "Checked Icon",
                        modifier = Modifier.size(SwitchDefaults.IconSize)
                    )
                } else {
                    Icon(
                        imageVector = Icons.Filled.Close,
                        contentDescription = "Unchecked Icon",
                        modifier = Modifier.size(SwitchDefaults.IconSize)
                    )
                }
            }
        )
    }

    @Preview
    @Composable
    fun DoDiscoveryPreview(modifier: Modifier = Modifier) {
        Example_appTheme {
            DoDiscovery()
        }
    }

}