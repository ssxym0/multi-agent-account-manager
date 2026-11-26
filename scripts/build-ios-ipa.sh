#!/usr/bin/env bash
set -euo pipefail

unset HOMEBREW_ASK
export IPHONEOS_DEPLOYMENT_TARGET="17.0"

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

npm run build

if [ ! -f "src-tauri/gen/apple/project.yml" ]; then
  npm run tauri -- ios init --ci --skip-targets-install
fi

if [ -d "$ROOT_DIR/src-tauri/icons/ios" ] && [ -d "$ROOT_DIR/src-tauri/gen/apple/Assets.xcassets/AppIcon.appiconset" ]; then
  cp -f "$ROOT_DIR/src-tauri/icons/ios/"*.png "$ROOT_DIR/src-tauri/gen/apple/Assets.xcassets/AppIcon.appiconset/"
fi

cargo build --manifest-path src-tauri/Cargo.toml --target aarch64-apple-ios --release --lib --features tauri/custom-protocol

LIB_SRC="$ROOT_DIR/src-tauri/target/aarch64-apple-ios/release/libmulti_agent_account_manager_lib.a"
LIB_DST_DIR="$ROOT_DIR/src-tauri/gen/apple/Externals/arm64/release"
if [ ! -f "$LIB_SRC" ]; then
  echo "未找到编译后的 iOS 静态库: $LIB_SRC" >&2
  exit 1
fi
mkdir -p "$LIB_DST_DIR"
cp -f "$LIB_SRC" "$LIB_DST_DIR/libapp.a"

HAS_IOS_RUNTIME=0
if xcrun simctl list runtimes 2>/dev/null | grep -qi 'iOS'; then
  HAS_IOS_RUNTIME=1
fi

python3 - "$ROOT_DIR/src-tauri/gen/apple/project.yml" "$HAS_IOS_RUNTIME" <<'PY'
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
has_runtime = sys.argv[2] == "1"
text = path.read_text(encoding="utf-8")
orig = text

if "- sdk: Network.framework" not in text:
    anchor = "      - sdk: WebKit.framework\n"
    if anchor in text:
        text = text.replace(anchor, anchor + "      - sdk: Network.framework\n", 1)
if "- sdk: AVFoundation.framework" not in text:
    anchor = "      - sdk: Network.framework\n"
    if anchor in text:
        text = text.replace(anchor, anchor + "      - sdk: AVFoundation.framework\n", 1)

text = text.replace("      PRODUCT_NAME: 叠号\n", "      PRODUCT_NAME: AccountManager\n")
if "        CFBundleDisplayName:" not in text:
    text = text.replace(
        "        LSRequiresIPhoneOS: true\n",
        "        CFBundleDisplayName: 叠号\n        LSRequiresIPhoneOS: true\n",
        1,
    )
if "        NSLocalNetworkUsageDescription:" not in text:
    text = text.replace(
        "        LSRequiresIPhoneOS: true\n",
        "        LSRequiresIPhoneOS: true\n        NSLocalNetworkUsageDescription: 用于局域网多端加密同步账号与登录态\n",
        1,
    )
if "        NSCameraUsageDescription:" not in text:
    text = text.replace(
        "        NSLocalNetworkUsageDescription: 用于局域网多端加密同步账号与登录态\n",
        "        NSLocalNetworkUsageDescription: 用于局域网多端加密同步账号与登录态\n        NSCameraUsageDescription: 用于扫描局域网同步二维码，快速导入账号与登录态\n",
        1,
    )

text = text.replace("      - path: LaunchScreen.storyboard\n", "")
text = text.replace("        UILaunchStoryboardName: LaunchScreen\n", "        UILaunchScreen: {}\n")

old_script = '          npm run -- tauri ios xcode-script'
new_script = '          [ "${SKIP_TAURI_XCODE_SCRIPT:-0}" = "1" ] || npm run -- tauri ios xcode-script'
if new_script not in text and old_script in text:
    text = text.replace(old_script, new_script, 1)

icons_block = (
    "        CFBundleIcons:\n"
    "          CFBundlePrimaryIcon:\n"
    "            CFBundleIconFiles:\n"
    "              - AppIcon60x60\n"
    "        CFBundleIcons~ipad:\n"
    "          CFBundlePrimaryIcon:\n"
    "            CFBundleIconFiles:\n"
    "              - AppIcon60x60\n"
    "              - AppIcon76x76\n"
    "              - AppIcon83.5x83.5\n"
)

if has_runtime:
    if "      - path: Assets.xcassets\n" not in text:
        sources_anchor = "      - path: Sources\n"
        if sources_anchor in text:
            text = text.replace(sources_anchor, sources_anchor + "      - path: Assets.xcassets\n", 1)
    text = text.replace(icons_block, "")
else:
    text = text.replace("      - path: Assets.xcassets\n", "")
    if "CFBundleIcons:" not in text:
        launch_anchor = "        UILaunchScreen: {}\n"
        if launch_anchor in text:
            text = text.replace(launch_anchor, launch_anchor + icons_block, 1)

if text != orig:
    path.write_text(text, encoding="utf-8")
PY

(
  cd "$ROOT_DIR/src-tauri/gen/apple"
  xcodegen generate --spec project.yml
)

OUT_DIR="$ROOT_DIR/src-tauri/target/ios"
BUILD_DIR="$OUT_DIR/Build"
SYM_DIR="$OUT_DIR/Sym"
OBJ_DIR="$OUT_DIR/Obj"
PAYLOAD_DIR="$OUT_DIR/Payload"
IPA_PATH="$OUT_DIR/account-manager-ios-unsigned.ipa"

rm -rf "$BUILD_DIR" "$PAYLOAD_DIR" "$IPA_PATH"
mkdir -p "$BUILD_DIR" "$SYM_DIR" "$OBJ_DIR"

SKIP_TAURI_XCODE_SCRIPT=1 xcodebuild \
  -project "$ROOT_DIR/src-tauri/gen/apple/multi-agent-account-manager.xcodeproj" \
  -target "multi-agent-account-manager_iOS" \
  -configuration release \
  -sdk iphoneos \
  CONFIGURATION_BUILD_DIR="$BUILD_DIR" \
  SYMROOT="$SYM_DIR" \
  OBJROOT="$OBJ_DIR" \
  ARCHS=arm64 \
  ONLY_ACTIVE_ARCH=YES \
  CODE_SIGNING_ALLOWED=NO \
  CODE_SIGNING_REQUIRED=NO \
  CODE_SIGN_IDENTITY="" \
  DEVELOPMENT_TEAM="" \
  IPHONEOS_DEPLOYMENT_TARGET=17.0 \
  build

APP_PATH="$(find "$BUILD_DIR" -maxdepth 1 -name '*.app' -print -quit)"
if [ -z "$APP_PATH" ] || [ ! -d "$APP_PATH" ]; then
  echo "未找到构建产物 .app" >&2
  exit 1
fi

if [ "$HAS_IOS_RUNTIME" = "0" ]; then
  ICON_SRC_DIR="$ROOT_DIR/src-tauri/gen/apple/Assets.xcassets/AppIcon.appiconset"
  cp -f "$ICON_SRC_DIR/AppIcon-60x60@2x.png" "$APP_PATH/AppIcon60x60@2x.png"
  cp -f "$ICON_SRC_DIR/AppIcon-60x60@3x.png" "$APP_PATH/AppIcon60x60@3x.png"
  cp -f "$ICON_SRC_DIR/AppIcon-76x76@2x.png" "$APP_PATH/AppIcon76x76@2x~ipad.png"
  cp -f "$ICON_SRC_DIR/AppIcon-83.5x83.5@2x.png" "$APP_PATH/AppIcon83.5x83.5@2x~ipad.png"
fi

mkdir -p "$PAYLOAD_DIR"
cp -R "$APP_PATH" "$PAYLOAD_DIR/"
APP_BASENAME="$(basename "$APP_PATH")"
rm -rf "$PAYLOAD_DIR/$APP_BASENAME/_CodeSignature"

(
  cd "$OUT_DIR"
  zip -qry "account-manager-ios-unsigned.ipa" "Payload"
)
rm -rf "$PAYLOAD_DIR"

echo "Generated unsigned IPA: $IPA_PATH"
ls -lh "$IPA_PATH"
