import AVFoundation
import UIKit

final class QrScannerViewController: UIViewController, AVCaptureMetadataOutputObjectsDelegate {
  private let onFinish: (String?, Bool, String?) -> Void
  private var captureSession: AVCaptureSession?
  private var previewLayer: AVCaptureVideoPreviewLayer?
  private var handled = false
  private let reticleView = UIView()
  private let hintLabel = UILabel()

  init(onFinish: @escaping (String?, Bool, String?) -> Void) {
    self.onFinish = onFinish
    super.init(nibName: nil, bundle: nil)
    modalPresentationStyle = .fullScreen
  }

  @available(*, unavailable)
  required init?(coder: NSCoder) {
    fatalError("init(coder:) has not been implemented")
  }

  override func viewDidLoad() {
    super.viewDidLoad()
    view.backgroundColor = .black
    setupChrome()
    requestCameraAndStart()
  }

  override func viewDidLayoutSubviews() {
    super.viewDidLayoutSubviews()
    previewLayer?.frame = view.bounds
    let boxSize: CGFloat = min(view.bounds.width, view.bounds.height) * 0.64
    reticleView.frame = CGRect(
      x: (view.bounds.width - boxSize) / 2,
      y: (view.bounds.height - boxSize) / 2 - 20,
      width: boxSize,
      height: boxSize
    )
    hintLabel.frame = CGRect(
      x: 24,
      y: reticleView.frame.maxY + 22,
      width: view.bounds.width - 48,
      height: 44
    )
  }

  override func viewWillDisappear(_ animated: Bool) {
    super.viewWillDisappear(animated)
    stopSession()
  }

  private func setupChrome() {
    reticleView.backgroundColor = .clear
    reticleView.layer.borderColor = UIColor.systemBlue.cgColor
    reticleView.layer.borderWidth = 2.5
    reticleView.layer.cornerRadius = 16
    view.addSubview(reticleView)

    hintLabel.text = "将另一台设备的同步二维码放入框内"
    hintLabel.textColor = .white
    hintLabel.font = .systemFont(ofSize: 14, weight: .medium)
    hintLabel.textAlignment = .center
    hintLabel.numberOfLines = 2
    view.addSubview(hintLabel)

    let cancelBtn = UIButton(type: .system)
    cancelBtn.setTitle("取消", for: .normal)
    cancelBtn.setTitleColor(.white, for: .normal)
    cancelBtn.titleLabel?.font = .systemFont(ofSize: 16, weight: .semibold)
    cancelBtn.backgroundColor = UIColor.black.withAlphaComponent(0.55)
    cancelBtn.layer.cornerRadius = 18
    cancelBtn.translatesAutoresizingMaskIntoConstraints = false
    cancelBtn.addTarget(self, action: #selector(handleCancel), for: .touchUpInside)
    view.addSubview(cancelBtn)

    let titleLbl = UILabel()
    titleLbl.text = "扫码同步"
    titleLbl.textColor = .white
    titleLbl.font = .systemFont(ofSize: 17, weight: .semibold)
    titleLbl.translatesAutoresizingMaskIntoConstraints = false
    view.addSubview(titleLbl)

    NSLayoutConstraint.activate([
      cancelBtn.topAnchor.constraint(equalTo: view.safeAreaLayoutGuide.topAnchor, constant: 12),
      cancelBtn.leadingAnchor.constraint(equalTo: view.leadingAnchor, constant: 16),
      cancelBtn.widthAnchor.constraint(equalToConstant: 64),
      cancelBtn.heightAnchor.constraint(equalToConstant: 36),
      titleLbl.centerYAnchor.constraint(equalTo: cancelBtn.centerYAnchor),
      titleLbl.centerXAnchor.constraint(equalTo: view.centerXAnchor),
    ])
  }

  private func requestCameraAndStart() {
    switch AVCaptureDevice.authorizationStatus(for: .video) {
    case .authorized:
      startCapture()
    case .notDetermined:
      AVCaptureDevice.requestAccess(for: .video) { [weak self] granted in
        DispatchQueue.main.async {
          guard let self = self else { return }
          if granted {
            self.startCapture()
          } else {
            self.finish(content: nil, cancelled: false, error: "请在系统设置中允许访问相机后再扫码")
          }
        }
      }
    default:
      finish(content: nil, cancelled: false, error: "相机权限未开启，请在系统设置中开启后重试")
    }
  }

  private func startCapture() {
    guard let device = AVCaptureDevice.default(for: .video),
          let input = try? AVCaptureDeviceInput(device: device) else {
      finish(content: nil, cancelled: false, error: "无法打开设备摄像头")
      return
    }
    let session = AVCaptureSession()
    guard session.canAddInput(input) else {
      finish(content: nil, cancelled: false, error: "初始化相机输入失败")
      return
    }
    session.addInput(input)

    let output = AVCaptureMetadataOutput()
    guard session.canAddOutput(output) else {
      finish(content: nil, cancelled: false, error: "初始化二维码识别失败")
      return
    }
    session.addOutput(output)
    output.setMetadataObjectsDelegate(self, queue: .main)
    if output.availableMetadataObjectTypes.contains(.qr) {
      output.metadataObjectTypes = [.qr]
    }

    let layer = AVCaptureVideoPreviewLayer(session: session)
    layer.videoGravity = .resizeAspectFill
    layer.frame = view.bounds
    view.layer.insertSublayer(layer, at: 0)
    self.previewLayer = layer
    self.captureSession = session

    DispatchQueue.global(qos: .userInitiated).async {
      session.startRunning()
    }
  }

  private func stopSession() {
    if let session = captureSession, session.isRunning {
      DispatchQueue.global(qos: .userInitiated).async {
        session.stopRunning()
      }
    }
  }

  func metadataOutput(
    _ output: AVCaptureMetadataOutput,
    didOutput metadataObjects: [AVMetadataObject],
    from connection: AVCaptureConnection
  ) {
    guard !handled else { return }
    for obj in metadataObjects {
      if let qr = obj as? AVMetadataMachineReadableCodeObject,
         let val = qr.stringValue,
         !val.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
        handled = true
        let feedback = UINotificationFeedbackGenerator()
        feedback.notificationOccurred(.success)
        finish(content: val, cancelled: false, error: nil)
        return
      }
    }
  }

  @objc private func handleCancel() {
    guard !handled else { return }
    handled = true
    finish(content: nil, cancelled: true, error: nil)
  }

  private func finish(content: String?, cancelled: Bool, error: String?) {
    stopSession()
    dismiss(animated: true) { [onFinish] in
      onFinish(content, cancelled, error)
    }
  }
}
