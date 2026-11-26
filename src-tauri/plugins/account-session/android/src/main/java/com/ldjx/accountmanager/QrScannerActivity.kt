package com.ldjx.accountmanager

import android.Manifest
import android.content.pm.PackageManager
import android.graphics.Color
import android.os.Bundle
import android.view.Gravity
import android.view.ViewGroup
import android.widget.Button
import android.widget.FrameLayout
import android.widget.TextView
import androidx.activity.result.contract.ActivityResultContracts
import androidx.appcompat.app.AppCompatActivity
import androidx.core.content.ContextCompat
import com.journeyapps.barcodescanner.BarcodeCallback
import com.journeyapps.barcodescanner.BarcodeResult
import com.journeyapps.barcodescanner.DecoratedBarcodeView

object QrScannerBridge {
    @Volatile
    private var pendingCallback: ((String?, Boolean, String?) -> Unit)? = null

    fun setCallback(cb: (String?, Boolean, String?) -> Unit) {
        pendingCallback?.invoke(null, true, null)
        pendingCallback = cb
    }

    fun complete(content: String?, cancelled: Boolean, error: String?) {
        val cb = pendingCallback
        pendingCallback = null
        cb?.invoke(content, cancelled, error)
    }
}

class QrScannerActivity : AppCompatActivity() {
    private var barcodeView: DecoratedBarcodeView? = null
    private var finished = false

    private val cameraPermissionLauncher =
        registerForActivityResult(ActivityResultContracts.RequestPermission()) { granted ->
            if (granted) {
                startScanner()
            } else {
                finishWith(null, false, "请在系统设置中允许相机权限后再扫码")
            }
        }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val root = FrameLayout(this).apply {
            setBackgroundColor(Color.BLACK)
        }

        val scanner = DecoratedBarcodeView(this).apply {
            setStatusText("将另一台设备的同步二维码放入框内")
        }
        barcodeView = scanner
        root.addView(
            scanner,
            FrameLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT,
                ViewGroup.LayoutParams.MATCH_PARENT
            )
        )

        val cancelBtn = Button(this).apply {
            text = "取消"
            setTextColor(Color.WHITE)
            setBackgroundColor(Color.argb(140, 0, 0, 0))
            setOnClickListener {
                finishWith(null, true, null)
            }
        }
        val cancelLp = FrameLayout.LayoutParams(
            ViewGroup.LayoutParams.WRAP_CONTENT,
            ViewGroup.LayoutParams.WRAP_CONTENT,
            Gravity.TOP or Gravity.START
        ).apply {
            topMargin = 48
            marginStart = 32
        }
        root.addView(cancelBtn, cancelLp)

        val titleView = TextView(this).apply {
            text = "扫码同步"
            setTextColor(Color.WHITE)
            textSize = 17f
        }
        val titleLp = FrameLayout.LayoutParams(
            ViewGroup.LayoutParams.WRAP_CONTENT,
            ViewGroup.LayoutParams.WRAP_CONTENT,
            Gravity.TOP or Gravity.CENTER_HORIZONTAL
        ).apply {
            topMargin = 62
        }
        root.addView(titleView, titleLp)

        setContentView(root)

        if (ContextCompat.checkSelfPermission(this, Manifest.permission.CAMERA) ==
            PackageManager.PERMISSION_GRANTED
        ) {
            startScanner()
        } else {
            cameraPermissionLauncher.launch(Manifest.permission.CAMERA)
        }
    }

    private fun startScanner() {
        val view = barcodeView ?: return
        view.resume()
        view.decodeSingle(object : BarcodeCallback {
            override fun barcodeResult(result: BarcodeResult?) {
                val text = result?.text?.trim().orEmpty()
                if (text.isNotEmpty()) {
                    finishWith(text, false, null)
                }
            }
        })
    }

    override fun onResume() {
        super.onResume()
        if (ContextCompat.checkSelfPermission(this, Manifest.permission.CAMERA) ==
            PackageManager.PERMISSION_GRANTED
        ) {
            barcodeView?.resume()
        }
    }

    override fun onPause() {
        barcodeView?.pause()
        super.onPause()
    }

    @Deprecated("Deprecated in Java")
    override fun onBackPressed() {
        finishWith(null, true, null)
    }

    private fun finishWith(content: String?, cancelled: Boolean, error: String?) {
        if (finished) return
        finished = true
        QrScannerBridge.complete(content, cancelled, error)
        finish()
    }
}
