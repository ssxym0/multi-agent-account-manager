import Foundation
import Tauri
import UIKit
import WebKit

private struct SubscribeEventsArgs: Decodable {
  let channel: Channel
}

private struct OpenSessionArgs: Decodable {
  let providerId: String
  let providerName: String
  let uuid: String
  let alias: String
  let autoAlias: Bool
  let entryUrl: String
  let allowedHosts: [String]
  let proxyUrl: String?
  let timezone: String?
  let timezoneInitScript: String?
  let switchSeq: UInt64
  let viewInstanceId: UInt64
  let isColdStart: Bool
}

private struct UpdateTitleArgs: Decodable {
  let switchSeq: UInt64
  let title: String
}

private struct CloseSessionArgs: Decodable {
  let switchSeq: UInt64
}

private struct DeleteProfileArgs: Decodable {
  let uuid: String
}

private struct CopyProfileCookieArgs: Decodable {
  let uuid: String
  let url: String
}

private struct ImportProfileCookiesArgs: Decodable {
  let uuid: String
  let url: String
  let cookies: [String]
}

private struct ExportProfileCookiesArgs: Decodable {
  let uuid: String
  let allowedHosts: [String]
  let urls: [String]?
}

private struct ExportedCookieItem: Encodable {
  let name: String
  let value: String
  let domain: String
  let path: String
  let secure: Bool
  let httpOnly: Bool
  let sameSite: String?
  let maxAgeSecs: Int64?
}

private struct ExportProfileCookiesResult: Encodable {
  let cookies: [ExportedCookieItem]
}

private struct OpenExternalUrlArgs: Decodable {
  let url: String
}

private struct FetchTextArgs: Decodable {
  let url: String
  let userAgent: String
  let accept: String?
}

private struct HttpRequestArgs: Decodable {
  let method: String
  let url: String
  let headers: [String: String]?
  let body: String?
}

private struct CapabilitiesResult: Encodable {
  let multiProfileSupported: Bool
  let profileNames: [String]
}

private struct FetchTextResult: Encodable {
  let status: UInt16
  let body: String
}

private struct ScanQrCodeResult: Encodable {
  let content: String
  let cancelled: Bool
}

@_cdecl("init_plugin_account_session")
func initPlugin() -> Plugin {
  return AccountSessionPlugin()
}

public class AccountSessionPlugin: Plugin {
  @objc public func subscribeEvents(_ invoke: Invoke) {
    do {
      let args = try invoke.parseArgs(SubscribeEventsArgs.self)
      AccountSessionCoordinator.shared.setEventChannel(args.channel)
      invoke.resolve()
    } catch {
      invoke.reject("订阅事件通道失败: \(error.localizedDescription)")
    }
  }

  @objc public func checkCapabilities(_ invoke: Invoke) {
    DispatchQueue.main.async {
      if #available(iOS 17.0, *) {
        WKWebsiteDataStore.fetchAllDataStoreIdentifiers { uuids in
          let names = uuids.map { $0.uuidString.lowercased() }
          invoke.resolve(CapabilitiesResult(multiProfileSupported: true, profileNames: names))
        }
      } else {
        invoke.resolve(CapabilitiesResult(multiProfileSupported: false, profileNames: []))
      }
    }
  }

  @objc public func openAccountSession(_ invoke: Invoke) {
    do {
      let args = try invoke.parseArgs(OpenSessionArgs.self)
      let spec = MobileAccountSessionSpec(
        providerId: args.providerId,
        providerName: args.providerName,
        uuid: args.uuid.lowercased(),
        alias: args.alias,
        autoAlias: args.autoAlias,
        entryUrl: args.entryUrl,
        allowedHosts: args.allowedHosts,
        proxyUrl: args.proxyUrl,
        timezone: args.timezone,
        timezoneInitScript: args.timezoneInitScript,
        switchSeq: args.switchSeq,
        viewInstanceId: args.viewInstanceId,
        isColdStart: args.isColdStart
      )
      guard #available(iOS 17.0, *) else {
        invoke.reject("多账号隔离需要 iOS 17.0 及以上系统")
        return
      }
      guard let parsedUuid = UUID(uuidString: spec.uuid) else {
        invoke.reject("账号资料 UUID 无效")
        return
      }

      DispatchQueue.main.async {
        let proceedOpen = {
          let existing = AccountSessionCoordinator.shared.beginSession(spec)
          guard AccountSessionCoordinator.shared.isCurrent(spec.switchSeq, spec.viewInstanceId) else {
            invoke.reject("切换请求已过期")
            return
          }
          if let vc = existing, vc.presentingViewController != nil || vc.view.window != nil {
            vc.bindSessionSpec(spec)
            invoke.resolve()
            return
          }
          guard let hostVC = self.resolvePresenterViewController() else {
            invoke.reject("无法获取 iOS 根视图控制器")
            return
          }
          let vc = AccountWebViewController(initialSpec: spec)
          vc.modalPresentationStyle = .fullScreen
          AccountSessionCoordinator.shared.attachController(vc)
          hostVC.present(vc, animated: true) {
            invoke.resolve()
          }
        }

        if spec.isColdStart {
          WKWebsiteDataStore.fetchAllDataStoreIdentifiers { uuids in
            let found = uuids.contains { $0 == parsedUuid }
            if !found {
              invoke.reject("已提交账号的 Profile 不存在")
              return
            }
            proceedOpen()
          }
        } else {
          proceedOpen()
        }
      }
    } catch {
      invoke.reject("解析会话参数失败: \(error.localizedDescription)")
    }
  }

  @objc public func updateAccountTitle(_ invoke: Invoke) {
    do {
      let args = try invoke.parseArgs(UpdateTitleArgs.self)
      DispatchQueue.main.async {
        AccountSessionCoordinator.shared.markCommittedAndReveal(args.switchSeq, title: args.title)
        invoke.resolve()
      }
    } catch {
      invoke.reject("更新标题参数无效")
    }
  }

  @objc public func closeActiveSession(_ invoke: Invoke) {
    do {
      let args = try invoke.parseArgs(CloseSessionArgs.self)
      DispatchQueue.main.async {
        let controller = AccountSessionCoordinator.shared.invalidateAndClose(args.switchSeq)
        controller?.closeFromManager()
        invoke.resolve()
      }
    } catch {
      invoke.reject("关闭会话参数无效")
    }
  }

  @objc public func deleteProfile(_ invoke: Invoke) {
    do {
      let args = try invoke.parseArgs(DeleteProfileArgs.self)
      let normUuid = args.uuid.lowercased()
      guard #available(iOS 17.0, *) else {
        invoke.resolve()
        return
      }
      guard let parsedUuid = UUID(uuidString: normUuid) else {
        invoke.reject("账号资料 UUID 无效")
        return
      }
      DispatchQueue.main.async {
        if AccountSessionCoordinator.shared.isUuidActive(normUuid) {
          invoke.reject("目标 Profile 视图尚未销毁")
          return
        }
        WKWebsiteDataStore.fetchAllDataStoreIdentifiers { uuids in
          guard uuids.contains(parsedUuid) else {
            invoke.resolve()
            return
          }
          WKWebsiteDataStore.remove(forIdentifier: parsedUuid) { err in
            if let err = err {
              invoke.reject("删除 iOS 隔离资料失败: \(err.localizedDescription)")
            } else {
              invoke.resolve()
            }
          }
        }
      }
    } catch {
      invoke.reject("删除资料参数无效")
    }
  }

  @objc public func copyProfileCookie(_ invoke: Invoke) {
    do {
      let args = try invoke.parseArgs(CopyProfileCookieArgs.self)
      guard #available(iOS 17.0, *) else {
        invoke.reject("Cookie 隔离需要 iOS 17.0 及以上系统")
        return
      }
      guard let parsedUuid = UUID(uuidString: args.uuid),
            let entryUrl = URL(string: args.url),
            let entryHost = entryUrl.host?.lowercased() else {
        invoke.reject("账号资料标识或入口地址无效")
        return
      }

      DispatchQueue.main.async {
        let store = WKWebsiteDataStore(forIdentifier: parsedUuid)
        store.httpCookieStore.getAllCookies { cookies in
          let now = Date()
          var pairs: [String] = []
          for cookie in cookies {
            if let exp = cookie.expiresDate, exp <= now {
              continue
            }
            let domain = cookie.domain.trimmingCharacters(in: CharacterSet(charactersIn: ".")).lowercased()
            if domain.isEmpty || entryHost == domain || entryHost.hasSuffix("." + domain) {
              pairs.append("\(cookie.name)=\(cookie.value)")
            }
          }
          if pairs.isEmpty {
            invoke.reject("该账号暂无可复制的 Cookie，请先登录")
            return
          }
          let header = pairs.joined(separator: "; ")
          UIPasteboard.general.setItems(
            [["public.utf8-plain-text": header]],
            options: [.localOnly: true]
          )
          invoke.resolve()
        }
      }
    } catch {
      invoke.reject("复制 Cookie 失败: \(error.localizedDescription)")
    }
  }

  @objc public func importProfileCookies(_ invoke: Invoke) {
    do {
      let args = try invoke.parseArgs(ImportProfileCookiesArgs.self)
      guard #available(iOS 17.0, *) else {
        invoke.reject("导入 Cookie 需要 iOS 17.0 及以上系统")
        return
      }
      guard let parsedUuid = UUID(uuidString: args.uuid),
            let defaultUrl = URL(string: args.url) else {
        invoke.reject("账号资料标识或服务地址无效")
        return
      }

      DispatchQueue.main.async {
        let store = WKWebsiteDataStore(forIdentifier: parsedUuid)
        var parsedCookies: [HTTPCookie] = []
        for header in args.cookies {
          let trimmed = header.trimmingCharacters(in: .whitespacesAndNewlines)
          if trimmed.isEmpty { continue }
          let targetUrl = Self.resolveTargetUrl(fromSetCookieHeader: trimmed, fallback: defaultUrl)
          let list = HTTPCookie.cookies(withResponseHeaderFields: ["Set-Cookie": trimmed], for: targetUrl)
          parsedCookies.append(contentsOf: list)
        }
        if parsedCookies.isEmpty {
          invoke.reject("未生成可写入 iOS 容器的 Cookie，请检查粘贴格式")
          return
        }

        let group = DispatchGroup()
        for cookie in parsedCookies {
          group.enter()
          store.httpCookieStore.setCookie(cookie) {
            group.leave()
          }
        }
        group.notify(queue: .main) {
          invoke.resolve()
        }
      }
    } catch {
      invoke.reject("导入 Cookie 失败: \(error.localizedDescription)")
    }
  }

  @objc public func exportProfileCookies(_ invoke: Invoke) {
    do {
      let args = try invoke.parseArgs(ExportProfileCookiesArgs.self)
      guard #available(iOS 17.0, *) else {
        invoke.reject("导出 Cookie 需要 iOS 17.0 及以上系统")
        return
      }
      guard let parsedUuid = UUID(uuidString: args.uuid) else {
        invoke.reject("账号资料标识无效")
        return
      }
      let allowed = args.allowedHosts.map { $0.trimmingCharacters(in: CharacterSet(charactersIn: ".")).lowercased() }.filter { !$0.isEmpty }
      DispatchQueue.main.async {
        let store = WKWebsiteDataStore(forIdentifier: parsedUuid)
        store.httpCookieStore.getAllCookies { cookies in
          let now = Date()
          var items: [ExportedCookieItem] = []
          for cookie in cookies {
            if let exp = cookie.expiresDate, exp <= now {
              continue
            }
            let rawDomain = cookie.domain.lowercased()
            let normDomain = rawDomain.trimmingCharacters(in: CharacterSet(charactersIn: "."))
            if normDomain.isEmpty { continue }
            let matched = allowed.isEmpty || allowed.contains { ah in
              normDomain == ah || normDomain.hasSuffix("." + ah) || ah.hasSuffix("." + normDomain)
            }
            if !matched { continue }
            var maxAge: Int64? = nil
            if let exp = cookie.expiresDate {
              let secs = Int64(exp.timeIntervalSince(now))
              if secs > 0 {
                maxAge = min(secs, 365 * 86400)
              }
            }
            items.append(ExportedCookieItem(
              name: cookie.name,
              value: cookie.value,
              domain: rawDomain.hasPrefix(".") ? rawDomain : normDomain,
              path: cookie.path.isEmpty ? "/" : cookie.path,
              secure: cookie.isSecure,
              httpOnly: cookie.isHTTPOnly,
              sameSite: nil,
              maxAgeSecs: maxAge
            ))
          }
          invoke.resolve(ExportProfileCookiesResult(cookies: items))
        }
      }
    } catch {
      invoke.reject("导出 Cookie 失败: \(error.localizedDescription)")
    }
  }

  @objc public func openExternalUrl(_ invoke: Invoke) {
    do {
      let args = try invoke.parseArgs(OpenExternalUrlArgs.self)
      guard let url = URL(string: args.url),
            let scheme = url.scheme?.lowercased(),
            scheme == "http" || scheme == "https" else {
        invoke.reject("仅允许打开 http/https 外部链接")
        return
      }
      DispatchQueue.main.async {
        UIApplication.shared.open(url, options: [:]) { ok in
          if ok {
            invoke.resolve()
          } else {
            invoke.reject("无法调用 Safari 打开链接")
          }
        }
      }
    } catch {
      invoke.reject("无效的外部链接参数")
    }
  }

  @objc public func fetchText(_ invoke: Invoke) {
    do {
      let args = try invoke.parseArgs(FetchTextArgs.self)
      guard let url = URL(string: args.url),
            url.scheme?.lowercased() == "https" else {
        invoke.reject("仅允许 HTTPS 更新地址")
        return
      }
      var req = URLRequest(url: url, cachePolicy: .reloadIgnoringLocalCacheData, timeoutInterval: 20)
      if !args.userAgent.isEmpty {
        req.setValue(args.userAgent, forHTTPHeaderField: "User-Agent")
      }
      if let accept = args.accept, !accept.isEmpty {
        req.setValue(accept, forHTTPHeaderField: "Accept")
      }

      let delegate = HttpsOnlyRedirectDelegate()
      let session = URLSession(configuration: .ephemeral, delegate: delegate, delegateQueue: nil)
      let task = session.dataTask(with: req) { data, response, err in
        session.finishTasksAndInvalidate()
        if let err = err {
          invoke.reject("网络请求失败: \(err.localizedDescription)")
          return
        }
        guard let httpResp = response as? HTTPURLResponse,
              httpResp.url?.scheme?.lowercased() == "https" else {
          invoke.reject("更新响应重定向到了非 HTTPS 地址")
          return
        }
        let payload = data ?? Data()
        if payload.count > 2 * 1024 * 1024 {
          invoke.reject("响应内容过大")
          return
        }
        let body = String(data: payload, encoding: .utf8) ?? ""
        invoke.resolve(FetchTextResult(status: UInt16(httpResp.statusCode), body: body))
      }
      task.resume()
    } catch {
      invoke.reject("解析网络请求参数失败")
    }
  }

  @objc public func httpRequest(_ invoke: Invoke) {
    do {
      let args = try invoke.parseArgs(HttpRequestArgs.self)
      let method = args.method.uppercased()
      guard method == "GET" || method == "POST" || method == "DELETE" else {
        invoke.reject("不支持的 HTTP 方法")
        return
      }
      guard let url = URL(string: args.url),
            url.scheme?.lowercased() == "https" else {
        invoke.reject("仅允许 HTTPS 中转地址")
        return
      }
      var req = URLRequest(url: url, cachePolicy: .reloadIgnoringLocalCacheData, timeoutInterval: 25)
      req.httpMethod = method
      if let headers = args.headers {
        for (k, v) in headers {
          req.setValue(v, forHTTPHeaderField: k)
        }
      }
      if let body = args.body, method == "POST" {
        req.httpBody = body.data(using: .utf8)
      }

      let delegate = HttpsOnlyRedirectDelegate()
      let session = URLSession(configuration: .ephemeral, delegate: delegate, delegateQueue: nil)
      let task = session.dataTask(with: req) { data, response, err in
        session.finishTasksAndInvalidate()
        if let err = err {
          invoke.reject("网络请求失败: \(err.localizedDescription)")
          return
        }
        guard let httpResp = response as? HTTPURLResponse,
              httpResp.url?.scheme?.lowercased() == "https" else {
          invoke.reject("响应重定向到了非 HTTPS 地址")
          return
        }
        let payload = data ?? Data()
        if payload.count > 4 * 1024 * 1024 {
          invoke.reject("响应内容过大")
          return
        }
        let body = String(data: payload, encoding: .utf8) ?? ""
        invoke.resolve(FetchTextResult(status: UInt16(httpResp.statusCode), body: body))
      }
      task.resume()
    } catch {
      invoke.reject("解析网络请求参数失败")
    }
  }

  @objc public func scanQrCode(_ invoke: Invoke) {
    DispatchQueue.main.async { [weak self] in
      guard let self = self, let presenter = self.resolvePresenterViewController() else {
        invoke.reject("无法打开扫码界面")
        return
      }
      let vc = QrScannerViewController { content, cancelled, error in
        if let error = error {
          invoke.reject(error)
        } else if cancelled {
          invoke.resolve(ScanQrCodeResult(content: "", cancelled: true))
        } else {
          invoke.resolve(ScanQrCodeResult(content: content ?? "", cancelled: false))
        }
      }
      presenter.present(vc, animated: true)
    }
  }

  private func resolvePresenterViewController() -> UIViewController? {
    var topVC: UIViewController? = manager.viewController
    if topVC == nil {
      let scenes = UIApplication.shared.connectedScenes.compactMap { $0 as? UIWindowScene }
      topVC = scenes
        .flatMap { $0.windows }
        .first { $0.isKeyWindow }?
        .rootViewController
    }
    while let presented = topVC?.presentedViewController {
      topVC = presented
    }
    return topVC
  }

  private static func resolveTargetUrl(fromSetCookieHeader header: String, fallback: URL) -> URL {
    for part in header.split(separator: ";").dropFirst() {
      let kv = part.trimmingCharacters(in: .whitespacesAndNewlines)
      if kv.lowercased().hasPrefix("domain=") {
        let rawDomain = String(kv.dropFirst("domain=".count))
          .trimmingCharacters(in: CharacterSet(charactersIn: " .\""))
        if !rawDomain.isEmpty, let url = URL(string: "https://\(rawDomain)/") {
          return url
        }
      }
    }
    return fallback
  }
}

private final class HttpsOnlyRedirectDelegate: NSObject, URLSessionTaskDelegate {
  func urlSession(
    _ session: URLSession,
    task: URLSessionTask,
    willPerformHTTPRedirection response: HTTPURLResponse,
    newRequest request: URLRequest,
    completionHandler: @escaping (URLRequest?) -> Void
  ) {
    if request.url?.scheme?.lowercased() == "https" {
      completionHandler(request)
    } else {
      completionHandler(nil)
    }
  }
}
