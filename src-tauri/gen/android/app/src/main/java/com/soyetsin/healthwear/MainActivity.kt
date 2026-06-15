package com.soyetsin.healthwear

import android.Manifest
import android.content.pm.PackageManager
import android.os.Build
import android.os.Bundle
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts
import androidx.core.content.ContextCompat

class MainActivity : TauriActivity() {
  private val permissionLauncher =
      registerForActivityResult(ActivityResultContracts.RequestMultiplePermissions()) { _ ->
        // 权限结果由 Rust btleplug 层在扫描/连接时反馈
      }

  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
    requestBlePermissionsIfNeeded()
  }

  private fun requestBlePermissionsIfNeeded() {
    val required = requiredBlePermissions().filter {
      ContextCompat.checkSelfPermission(this, it) != PackageManager.PERMISSION_GRANTED
    }
    if (required.isNotEmpty()) {
      permissionLauncher.launch(required.toTypedArray())
    }
  }

  private fun requiredBlePermissions(): List<String> {
    return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
      listOf(
          Manifest.permission.BLUETOOTH_SCAN,
          Manifest.permission.BLUETOOTH_CONNECT,
      )
    } else {
      listOf(
          Manifest.permission.ACCESS_FINE_LOCATION,
          Manifest.permission.BLUETOOTH,
          Manifest.permission.BLUETOOTH_ADMIN,
      )
    }
  }
}
