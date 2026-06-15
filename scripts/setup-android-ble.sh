#!/usr/bin/env bash
# 构建 btleplug Android Java 库（AAR），供 Tauri Android 使用。
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
LIBS_DIR="$ROOT/src-tauri/gen/android/app/libs"
WORK="$ROOT/.cache/android-ble"
JNI_UTILS="$WORK/jni-utils-rs"
BTLE="$WORK/btleplug"

mkdir -p "$LIBS_DIR" "$WORK"

if [ ! -d "$JNI_UTILS/.git" ]; then
  git clone --depth 1 https://github.com/deviceplug/jni-utils-rs.git "$JNI_UTILS"
fi

if [ ! -d "$BTLE/.git" ]; then
  git clone --depth 1 https://github.com/deviceplug/btleplug.git "$BTLE"
fi

echo ">> 构建 jni-utils Java..."
(
  cd "$JNI_UTILS"
  cargo build --features=build-java-support
)

JNI_JAR="$(find "$JNI_UTILS/target" -name 'jni-utils-*.jar' | head -1)"
if [ -z "$JNI_JAR" ]; then
  echo "未找到 jni-utils jar" >&2
  exit 1
fi

echo ">> 构建 btleplug AAR..."
JAVA_DIR="$BTLE/src/droidplug/java"
GRADLE="$JAVA_DIR/build.gradle"

python3 - <<PY
from pathlib import Path
p = Path("$GRADLE")
text = p.read_text()
needle = "dependencies {"
if needle not in text:
    raise SystemExit("build.gradle 结构变化，请手动配置 jni-utils jar")
block = f'''dependencies {{
    implementation files('{JNI_JAR}')
'''
import re
text = re.sub(r'dependencies \{[^}]*\}', block.rstrip() + '\n}', text, count=1)
p.write_text(text)
PY

(
  cd "$JAVA_DIR"
  ./gradlew assembleRelease
)

AAR="$(find "$JAVA_DIR/build/outputs/aar" -name '*-release.aar' | head -1)"
if [ -z "$AAR" ]; then
  echo "未找到 btleplug AAR" >&2
  exit 1
fi

cp "$AAR" "$LIBS_DIR/btleplug-release.aar"
cp "$JNI_JAR" "$LIBS_DIR/"

echo ">> 完成。已复制到 $LIBS_DIR"
echo "   接下来: pnpm tauri android dev"
