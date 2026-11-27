package com.ldjx.accountmanager

import android.app.Activity
import android.content.ClipData
import android.content.ClipDescription
import android.content.ClipboardManager
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.os.PersistableBundle
import android.provider.Settings
import androidx.core.content.FileProvider
import androidx.webkit.ProfileStore
import androidx.webkit.WebViewFeature
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Channel
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import java.io.ByteArrayOutputStream
import java.io.File
import java.io.FileInputStream
import java.io.FileOutputStream
import java.net.HttpURLConnection
import java.net.URL
import java.security.MessageDigest
import kotlin.concurrent.thread

@InvokeArg
class SubscribeEventsArgs {
    lateinit var channel: Channel
}

@InvokeArg
class OpenSessionArgs {
    var providerId: String = ""
    var providerName: String = ""
    var uuid: String = ""
    var alias: String = ""
    var autoAlias: Boolean = false
    var entryUrl: String = ""
    var allowedHosts: Array<String> = emptyArray()
    var proxyUrl: String? = null
    var timezone: String? = null
    var timezoneInitScript: String? = null
    var switchSeq: Long = 0L
    var viewInstanceId: Long = 0L
    var isColdStart: Boolean = false
}

@InvokeArg
class UpdateTitleArgs {
    var switchSeq: Long = 0L
    var title: String = ""
}

@InvokeArg
class CloseSessionArgs {
    var switchSeq: Long = 0L
}

@InvokeArg
class DeleteProfileArgs {
    var uuid: String = ""
}

@InvokeArg
class CopyProfileCookieArgs {
    var uuid: String = ""
    var url: String = ""
}

@InvokeArg
class ImportProfileCookiesArgs {
    var uuid: String = ""
    var url: String = ""
    var cookies: Array<String> = emptyArray()
}

@InvokeArg
class ExportProfileCookiesArgs {
    var uuid: String = ""
    var allowedHosts: Array<String> = emptyArray()
    var urls: Array<String> = emptyArray()
}

@InvokeArg
class OpenExternalUrlArgs {
    var url: String = ""
}

@InvokeArg
class FetchTextArgs {
    var url: String = ""
    var userAgent: String = ""
    var accept: String? = null
}

@InvokeArg
class HttpRequestArgs {
    var method: String = "GET"
    var url: String = ""
    var headers: Map<String, String>? = null
    var body: String? = null
}

@InvokeArg
class DownloadApkArgs {
    var url: String = ""
    var sha256: String? = null
    var filename: String = ""
    var userAgent: String = ""
}

@TauriPlugin
class AccountSessionPlugin(private val activity: Activity) : Plugin(activity) {
    @Volatile
    private var lastDownloadedUrl: String? = null

    @Volatile
    private var lastDownloadedPath: String? = null

    @Command
    fun subscribeEvents(invoke: Invoke) {
        val args = invoke.parseArgs(SubscribeEventsArgs::class.java)
        AccountSessionCoordinator.setEventChannel(args.channel)
        invoke.resolve(JSObject())
    }

    @Command
    fun checkCapabilities(invoke: Invoke) {
        activity.runOnUiThread {
            try {
                val supported = WebViewFeature.isFeatureSupported(WebViewFeature.MULTI_PROFILE)
                val profiles = JSArray()
                if (supported) {
                    for (name in ProfileStore.getInstance().allProfileNames) {
                        profiles.put(name)
                    }
                }
                val ret = JSObject().apply {
                    put("multiProfileSupported", supported)
                    put("profileNames", profiles)
                }
                invoke.resolve(ret)
            } catch (t: Throwable) {
                invoke.reject(t.message ?: "检测 Android WebView 能力异常")
            }
        }
    }

    @Command
    fun openAccountSession(invoke: Invoke) {
        val args = invoke.parseArgs(OpenSessionArgs::class.java)
        activity.runOnUiThread {
            if (!WebViewFeature.isFeatureSupported(WebViewFeature.MULTI_PROFILE)) {
                invoke.reject("系统 WebView 不支持 MULTI_PROFILE")
                return@runOnUiThread
            }
            if (args.isColdStart) {
                val allNames = ProfileStore.getInstance().allProfileNames
                if (!allNames.contains(args.uuid)) {
                    invoke.reject("已提交账号的 Profile 不存在")
                    return@runOnUiThread
                }
            }

            val spec = AndroidAccountSessionSpec(
                providerId = args.providerId,
                providerName = args.providerName,
                uuid = args.uuid,
                alias = args.alias,
                autoAlias = args.autoAlias,
                entryUrl = args.entryUrl,
                allowedHosts = args.allowedHosts.toList(),
                timezone = args.timezone,
                timezoneInitScript = args.timezoneInitScript,
                switchSeq = args.switchSeq,
                viewInstanceId = args.viewInstanceId,
                isColdStart = args.isColdStart,
            )

            val existing = AccountSessionCoordinator.beginSession(spec)
            if (!AccountSessionCoordinator.isCurrent(spec.switchSeq, spec.viewInstanceId)) {
                invoke.reject("切换请求已过期")
                return@runOnUiThread
            }
            if (existing != null && !existing.isFinishing && !existing.isDestroyed) {
                existing.bindSessionSpec(spec)
            } else {
                val intent = Intent(activity, AccountWebActivity::class.java).apply {
                    addFlags(Intent.FLAG_ACTIVITY_SINGLE_TOP)
                }
                activity.startActivity(intent)
            }
            invoke.resolve(JSObject())
        }
    }

    @Command
    fun updateAccountTitle(invoke: Invoke) {
        val args = invoke.parseArgs(UpdateTitleArgs::class.java)
        activity.runOnUiThread {
            AccountSessionCoordinator.markCommittedAndReveal(args.switchSeq, args.title)
            invoke.resolve(JSObject())
        }
    }

    @Command
    fun closeActiveSession(invoke: Invoke) {
        val args = invoke.parseArgs(CloseSessionArgs::class.java)
        activity.runOnUiThread {
            val previous = AccountSessionCoordinator.currentActiveSpec()
            val act = AccountSessionCoordinator.invalidateAndClose(args.switchSeq)
            act?.closeAndFinishFromManager(keepPage = previous != null && args.switchSeq > previous.switchSeq)
            invoke.resolve(JSObject())
        }
    }

    @Command
    fun deleteProfile(invoke: Invoke) {
        val args = invoke.parseArgs(DeleteProfileArgs::class.java)
        activity.runOnUiThread {
            try {
                if (!WebViewFeature.isFeatureSupported(WebViewFeature.MULTI_PROFILE)) {
                    invoke.resolve(JSObject())
                    return@runOnUiThread
                }
                if (AccountSessionCoordinator.isUuidActive(args.uuid)) {
                    invoke.reject("目标 Profile 视图尚未销毁")
                    return@runOnUiThread
                }
                RecentAccountPages.discardProfile(args.uuid)
                val store = ProfileStore.getInstance()
                if (store.allProfileNames.contains(args.uuid)) {
                    val deleted = store.deleteProfile(args.uuid)
                    if (!deleted) {
                        invoke.reject("系统 Profile 正在被占用，待下次启动重试")
                        return@runOnUiThread
                    }
                }
                invoke.resolve(JSObject())
            } catch (t: Throwable) {
                invoke.reject(t.message ?: "删除 Android Profile 失败")
            }
        }
    }

    @Command
    fun copyProfileCookie(invoke: Invoke) {
        val args = invoke.parseArgs(CopyProfileCookieArgs::class.java)
        activity.runOnUiThread {
            try {
                if (!WebViewFeature.isFeatureSupported(WebViewFeature.MULTI_PROFILE)) {
                    invoke.reject("系统 WebView 不支持 MULTI_PROFILE")
                    return@runOnUiThread
                }
                val cookies = ProfileStore.getInstance().getProfile(args.uuid)
                    ?.cookieManager?.getCookie(args.url)
                if (cookies.isNullOrEmpty()) {
                    invoke.reject("该账号暂无可复制的 Cookie，请先登录")
                    return@runOnUiThread
                }
                val clipboard = activity.getSystemService(ClipboardManager::class.java)
                if (clipboard == null) {
                    invoke.reject("无法写入系统剪贴板，请重试")
                    return@runOnUiThread
                }
                val clip = ClipData.newPlainText("Cookie", cookies)
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                    clip.description.extras = PersistableBundle().apply {
                        putBoolean(ClipDescription.EXTRA_IS_SENSITIVE, true)
                    }
                }
                clipboard.setPrimaryClip(clip)
                invoke.resolve(JSObject())
            } catch (error: Exception) {
                invoke.reject("复制 Cookie 失败，请稍后重试")
            }
        }
    }

    @Command
    fun importProfileCookies(invoke: Invoke) {
        val args = invoke.parseArgs(ImportProfileCookiesArgs::class.java)
        activity.runOnUiThread {
            try {
                if (!WebViewFeature.isFeatureSupported(WebViewFeature.MULTI_PROFILE)) {
                    invoke.reject("系统 WebView 不支持 MULTI_PROFILE")
                    return@runOnUiThread
                }
                val cookieManager = ProfileStore.getInstance().getProfile(args.uuid)?.cookieManager
                if (cookieManager == null) {
                    invoke.reject("无法访问该账号的隔离 Cookie 存储")
                    return@runOnUiThread
                }
                cookieManager.setAcceptCookie(true)
                for (header in args.cookies) {
                    if (header.isNotBlank()) {
                        val targetUrl = resolveCookieTargetUrl(header, args.url)
                        cookieManager.setCookie(targetUrl, header)
                    }
                }
                cookieManager.flush()
                invoke.resolve(JSObject())
            } catch (error: Exception) {
                invoke.reject("导入 Cookie 失败，请检查格式后重试")
            }
        }
    }

    private fun resolveCookieTargetUrl(header: String, fallbackUrl: String): String {
        val parts = header.split(';')
        for (i in 1 until parts.size) {
            val kv = parts[i].trim()
            if (kv.startsWith("domain=", ignoreCase = true)) {
                val rawDomain = kv.substring(7).trim().trim('.', ' ', '"')
                if (rawDomain.isNotEmpty() && !rawDomain.contains('/') && !rawDomain.contains(' ')) {
                    return "https://$rawDomain/"
                }
            }
        }
        return fallbackUrl
    }

    @Command
    fun exportProfileCookies(invoke: Invoke) {
        val args = invoke.parseArgs(ExportProfileCookiesArgs::class.java)
        activity.runOnUiThread {
            try {
                if (!WebViewFeature.isFeatureSupported(WebViewFeature.MULTI_PROFILE)) {
                    invoke.reject("系统 WebView 不支持 MULTI_PROFILE")
                    return@runOnUiThread
                }
                val cookieManager = ProfileStore.getInstance().getProfile(args.uuid)?.cookieManager
                val arr = JSArray()
                if (cookieManager != null) {
                    val seen = HashSet<String>()
                    for (rawUrl in args.urls) {
                        val u = rawUrl.trim()
                        if (u.isEmpty()) continue
                        val host = try {
                            Uri.parse(u).host?.lowercase()?.trimStart('.') ?: ""
                        } catch (_: Exception) {
                            ""
                        }
                        if (host.isEmpty()) continue
                        val rawCookie = cookieManager.getCookie(u) ?: continue
                        for (pair in rawCookie.split(';')) {
                            val trimmed = pair.trim()
                            val eqIdx = trimmed.indexOf('=')
                            if (eqIdx <= 0) continue
                            val name = trimmed.substring(0, eqIdx).trim()
                            val value = trimmed.substring(eqIdx + 1).trim()
                            if (name.isEmpty()) continue
                            val dedupKey = "$host|$name"
                            if (!seen.add(dedupKey)) continue
                            val item = JSObject().apply {
                                put("name", name)
                                put("value", value)
                                put("domain", host)
                                put("path", "/")
                                put("secure", u.startsWith("https://", ignoreCase = true))
                                put("httpOnly", false)
                            }
                            arr.put(item)
                        }
                    }
                }
                val ret = JSObject().apply {
                    put("cookies", arr)
                }
                invoke.resolve(ret)
            } catch (error: Exception) {
                invoke.reject("导出 Cookie 失败，请稍后重试")
            }
        }
    }

    @Command
    fun openExternalUrl(invoke: Invoke) {
        val args = invoke.parseArgs(OpenExternalUrlArgs::class.java)
        activity.runOnUiThread {
            try {
                val uri = Uri.parse(args.url)
                val scheme = uri.scheme?.lowercase() ?: ""
                if (scheme != "http" && scheme != "https") {
                    invoke.reject("仅允许打开 http/https 外部链接")
                    return@runOnUiThread
                }
                val intent = Intent(Intent.ACTION_VIEW, uri).apply {
                    addCategory(Intent.CATEGORY_BROWSABLE)
                    addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                }
                activity.startActivity(intent)
                invoke.resolve(JSObject())
            } catch (t: Throwable) {
                invoke.reject(t.message ?: "无法调用系统浏览器打开链接")
            }
        }
    }

    @Command
    fun fetchText(invoke: Invoke) {
        val args = invoke.parseArgs(FetchTextArgs::class.java)
        thread(name = "maam-update-fetch") {
            var conn: HttpURLConnection? = null
            try {
                conn = openHttpConnection(args.url, args.userAgent, args.accept, 20_000)
                val status = conn.responseCode
                val stream = if (status in 200..299) conn.inputStream else conn.errorStream
                val body = if (stream != null) {
                    stream.use { input ->
                        val out = ByteArrayOutputStream()
                        val buf = ByteArray(8192)
                        var totalRead = 0
                        while (true) {
                            val n = input.read(buf)
                            if (n <= 0) break
                            totalRead += n
                            if (totalRead > 2 * 1024 * 1024) {
                                throw IllegalStateException("响应内容过大")
                            }
                            out.write(buf, 0, n)
                        }
                        out.toString(Charsets.UTF_8.name())
                    }
                } else {
                    ""
                }
                val ret = JSObject().apply {
                    put("status", status)
                    put("body", body)
                }
                invoke.resolve(ret)
            } catch (t: Throwable) {
                invoke.reject(t.message ?: "网络请求失败")
            } finally {
                conn?.disconnect()
            }
        }
    }

    @Command
    fun httpRequest(invoke: Invoke) {
        val args = invoke.parseArgs(HttpRequestArgs::class.java)
        thread(name = "maam-relay-http") {
            var conn: HttpURLConnection? = null
            try {
                val method = args.method.uppercase()
                if (method !in setOf("GET", "POST", "PUT", "DELETE", "HEAD", "OPTIONS")) {
                    throw IllegalArgumentException("不支持的 HTTP 方法")
                }
                val parsed = URL(args.url)
                val allowLocalHttp =
                    parsed.protocol.equals("http", ignoreCase = true) && isPrivateOrLocalHost(parsed.host ?: "")
                if (!parsed.protocol.equals("https", ignoreCase = true) && !allowLocalHttp) {
                    throw IllegalArgumentException("仅允许 HTTPS 或局域网内网 WebDAV 地址")
                }
                conn = (parsed.openConnection() as HttpURLConnection).apply {
                    instanceFollowRedirects = false
                    connectTimeout = 15_000
                    readTimeout = 30_000
                    requestMethod = method
                    useCaches = false
                    args.headers?.forEach { (k, v) ->
                        setRequestProperty(k, v)
                    }
                    if (args.body != null && (method == "POST" || method == "PUT")) {
                        doOutput = true
                        val bytes = args.body!!.toByteArray(Charsets.UTF_8)
                        setFixedLengthStreamingMode(bytes.size)
                        outputStream.use { it.write(bytes) }
                    }
                }
                val status = conn.responseCode
                val stream = if (status in 200..299) conn.inputStream else conn.errorStream
                val body = if (stream != null) {
                    stream.use { input ->
                        val out = ByteArrayOutputStream()
                        val buf = ByteArray(8192)
                        var totalRead = 0
                        while (true) {
                            val n = input.read(buf)
                            if (n <= 0) break
                            totalRead += n
                            if (totalRead > 16 * 1024 * 1024) {
                                throw IllegalStateException("响应内容过大")
                            }
                            out.write(buf, 0, n)
                        }
                        out.toString(Charsets.UTF_8.name())
                    }
                } else {
                    ""
                }
                val ret = JSObject().apply {
                    put("status", status)
                    put("body", body)
                }
                invoke.resolve(ret)
            } catch (t: Throwable) {
                invoke.reject(t.message ?: "网络请求失败")
            } finally {
                conn?.disconnect()
            }
        }
    }

    private fun isPrivateOrLocalHost(host: String): Boolean {
        val lower = host.trim().lowercase()
        if (lower == "localhost" || lower.endsWith(".local") || lower == "127.0.0.1" || lower == "::1" || lower == "[::1]") {
            return true
        }
        val parts = lower.split(".").mapNotNull { it.toIntOrNull() }
        if (parts.size != 4 || parts.any { it !in 0..255 }) {
            return false
        }
        if (parts[0] == 10 || parts[0] == 127) return true
        if (parts[0] == 192 && parts[1] == 168) return true
        if (parts[0] == 172 && parts[1] in 16..31) return true
        if (parts[0] == 169 && parts[1] == 254) return true
        return false
    }

    @Command
    fun downloadAndInstallApk(invoke: Invoke) {
        val args = invoke.parseArgs(DownloadApkArgs::class.java)
        thread(name = "maam-update-apk") {
            var conn: HttpURLConnection? = null
            val rawName = args.filename.substringAfterLast('/').substringAfterLast('\\').trim()
            val safeName = rawName
                .replace(Regex("""[^A-Za-z0-9._-]"""), "_")
                .trimStart('.')
                .ifEmpty { "update.apk" }
                .let { if (it.endsWith(".apk", ignoreCase = true)) it else "$it.apk" }
            val updatesDir = File(activity.cacheDir, "updates").apply { mkdirs() }
            val targetFile = File(updatesDir, safeName)
            val partFile = File(updatesDir, "$safeName.part")
            val expectedSha = args.sha256?.trim()?.lowercase()?.takeIf { it.isNotEmpty() }

            try {
                val canReuse = targetFile.exists() && targetFile.length() > 0L && (
                    (expectedSha != null && computeSha256Hex(targetFile) == expectedSha) ||
                        (expectedSha == null && lastDownloadedUrl == args.url && lastDownloadedPath == targetFile.absolutePath)
                )

                if (!canReuse) {
                    AccountSessionCoordinator.emitUpdateProgress(
                        stage = "downloading",
                        downloaded = 0L,
                        total = null,
                        percent = 0,
                        message = "正在连接下载服务器...",
                    )
                    conn = openHttpConnection(args.url, args.userAgent, null, 60_000)
                    val status = conn.responseCode
                    if (status !in 200..299) {
                        throw IllegalStateException("下载更新包失败（HTTP $status）")
                    }
                    val totalBytes = conn.contentLengthLong.takeIf { it > 0L }
                    val digest = MessageDigest.getInstance("SHA-256")
                    var downloaded = 0L
                    var lastEmitTime = 0L
                    var lastEmitPercent = -1

                    if (partFile.exists()) {
                        partFile.delete()
                    }
                    conn.inputStream.use { input ->
                        FileOutputStream(partFile).use { output ->
                            val buf = ByteArray(16 * 1024)
                            while (true) {
                                val n = input.read(buf)
                                if (n <= 0) break
                                output.write(buf, 0, n)
                                digest.update(buf, 0, n)
                                downloaded += n
                                val pct = totalBytes?.let { ((downloaded * 100L) / it).toInt().coerceIn(0, 100) }
                                val now = System.currentTimeMillis()
                                if (now - lastEmitTime >= 150L || (pct != null && pct != lastEmitPercent)) {
                                    lastEmitTime = now
                                    if (pct != null) lastEmitPercent = pct
                                    AccountSessionCoordinator.emitUpdateProgress(
                                        stage = "downloading",
                                        downloaded = downloaded,
                                        total = totalBytes,
                                        percent = pct,
                                        message = "正在下载安装包...",
                                    )
                                }
                            }
                            output.fd.sync()
                        }
                    }

                    AccountSessionCoordinator.emitUpdateProgress(
                        stage = "verifying",
                        downloaded = downloaded,
                        total = totalBytes,
                        percent = 100,
                        message = "正在校验安装包完整性...",
                    )
                    val actualSha = digest.digest().joinToString("") { "%02x".format(it) }
                    if (expectedSha != null && actualSha != expectedSha) {
                        partFile.delete()
                        throw IllegalStateException("安装包 SHA-256 校验失败，请重试")
                    }

                    if (targetFile.exists()) {
                        targetFile.delete()
                    }
                    if (!partFile.renameTo(targetFile)) {
                        FileInputStream(partFile).use { input ->
                            FileOutputStream(targetFile).use { output ->
                                input.copyTo(output)
                                output.fd.sync()
                            }
                        }
                        partFile.delete()
                    }

                    updatesDir.listFiles()?.forEach { f ->
                        if (f.name != targetFile.name) {
                            f.delete()
                        }
                    }
                    lastDownloadedUrl = args.url
                    lastDownloadedPath = targetFile.absolutePath
                }

                activity.runOnUiThread {
                    try {
                        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O && !activity.packageManager.canRequestPackageInstalls()) {
                            val permIntent = Intent(
                                Settings.ACTION_MANAGE_UNKNOWN_APP_SOURCES,
                                Uri.parse("package:${activity.packageName}"),
                            ).apply {
                                addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                            }
                            activity.startActivity(permIntent)
                            val msg = "请先开启“允许安装未知来源应用”，开启后返回再次点击安装"
                            AccountSessionCoordinator.emitUpdateProgress(
                                stage = "needs_permission",
                                downloaded = targetFile.length(),
                                total = targetFile.length(),
                                percent = 100,
                                message = msg,
                            )
                            val ret = JSObject().apply {
                                put("status", "needs_permission")
                                put("message", msg)
                            }
                            invoke.resolve(ret)
                            return@runOnUiThread
                        }

                        val apkUri = FileProvider.getUriForFile(
                            activity,
                            "${activity.packageName}.fileprovider",
                            targetFile,
                        )
                        val installIntent = Intent(Intent.ACTION_VIEW).apply {
                            setDataAndType(apkUri, "application/vnd.android.package-archive")
                            addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_GRANT_READ_URI_PERMISSION)
                        }
                        activity.startActivity(installIntent)
                        val msg = "已调起系统安装器，请按提示完成覆盖更新"
                        AccountSessionCoordinator.emitUpdateProgress(
                            stage = "ready",
                            downloaded = targetFile.length(),
                            total = targetFile.length(),
                            percent = 100,
                            message = msg,
                        )
                        val ret = JSObject().apply {
                            put("status", "installer_launched")
                            put("message", msg)
                        }
                        invoke.resolve(ret)
                    } catch (t: Throwable) {
                        val err = t.message ?: "调起系统安装器失败"
                        AccountSessionCoordinator.emitUpdateProgress("error", 0L, null, null, err)
                        invoke.reject(err)
                    }
                }
            } catch (t: Throwable) {
                if (partFile.exists()) {
                    partFile.delete()
                }
                val err = t.message ?: "下载更新包失败"
                AccountSessionCoordinator.emitUpdateProgress("error", 0L, null, null, err)
                invoke.reject(err)
            } finally {
                conn?.disconnect()
            }
        }
    }

    private fun openHttpConnection(
        rawUrl: String,
        userAgent: String,
        accept: String?,
        readTimeoutMs: Int,
    ): HttpURLConnection {
        var currentUrl = URL(rawUrl)
        if (!currentUrl.protocol.equals("https", ignoreCase = true)) {
            throw IllegalArgumentException("仅允许 HTTPS 更新地址")
        }
        for (hop in 0 until 10) {
            val conn = (currentUrl.openConnection() as HttpURLConnection).apply {
                connectTimeout = 15_000
                readTimeout = readTimeoutMs
                instanceFollowRedirects = false
                if (userAgent.isNotEmpty()) {
                    setRequestProperty("User-Agent", userAgent)
                }
                if (!accept.isNullOrEmpty()) {
                    setRequestProperty("Accept", accept)
                }
            }
            val code = conn.responseCode
            if (code == HttpURLConnection.HTTP_MOVED_PERM ||
                code == HttpURLConnection.HTTP_MOVED_TEMP ||
                code == HttpURLConnection.HTTP_SEE_OTHER ||
                code == 307 ||
                code == 308
            ) {
                val location = conn.getHeaderField("Location")
                conn.disconnect()
                if (location.isNullOrEmpty()) {
                    throw IllegalStateException("重定向缺少 Location 头")
                }
                val nextUrl = URL(currentUrl, location)
                if (!nextUrl.protocol.equals("https", ignoreCase = true)) {
                    throw IllegalStateException("重定向到非 HTTPS 地址已被拒绝")
                }
                currentUrl = nextUrl
                continue
            }
            return conn
        }
        throw IllegalStateException("网络重定向次数过多")
    }

    @Command
    fun scanQrCode(invoke: Invoke) {
        activity.runOnUiThread {
            try {
                QrScannerBridge.setCallback { content, cancelled, error ->
                    if (error != null) {
                        invoke.reject(error)
                    } else {
                        val ret = JSObject().apply {
                            put("content", content ?: "")
                            put("cancelled", cancelled)
                        }
                        invoke.resolve(ret)
                    }
                }
                val intent = Intent(activity, QrScannerActivity::class.java)
                activity.startActivity(intent)
            } catch (t: Throwable) {
                invoke.reject(t.message ?: "无法打开扫码界面")
            }
        }
    }

    private fun computeSha256Hex(file: File): String {
        val digest = MessageDigest.getInstance("SHA-256")
        FileInputStream(file).use { input ->
            val buf = ByteArray(16 * 1024)
            while (true) {
                val n = input.read(buf)
                if (n <= 0) break
                digest.update(buf, 0, n)
            }
        }
        return digest.digest().joinToString("") { "%02x".format(it) }
    }
}
