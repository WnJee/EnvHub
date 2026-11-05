#!/usr/bin/env bash
# Generate release notes for a tag from the commits since the previous tag.
#
# Usage: scripts/generate-release-notes.sh <tag> [output-file]
#
# Requires full git history and tags (`actions/checkout` with fetch-depth: 0
# and fetch-tags: true), otherwise the previous tag cannot be resolved and every
# release would list the entire history.
set -euo pipefail

TAG="${1:?usage: generate-release-notes.sh <tag> [output-file]}"
OUT="${2:-/dev/stdout}"

# Release-note noise: version bumps are implied by the tag itself.
strip_noise() {
  sed -E \
    -e 's/,?[[:space:]]*(release|bump(ing)?( version)?( to)?|version bump to)[[:space:]]*v?[0-9]+\.[0-9]+\.[0-9]+.*$//I' \
    -e 's/[[:space:]]+(and|&|\+|,|:|-)[[:space:]]*$//I' \
    -e 's/[[:space:]]+$//'
}

previous_tag=""
if git rev-parse --verify --quiet "${TAG}" >/dev/null; then
  previous_tag=$(git describe --tags --abbrev=0 --match 'v*' "${TAG}^" 2>/dev/null || true)
elif git rev-parse --verify --quiet "HEAD" >/dev/null; then
  previous_tag=$(git describe --tags --abbrev=0 --match 'v*' HEAD 2>/dev/null || true)
fi

if [ -n "$previous_tag" ] && git rev-parse --verify --quiet "${TAG}" >/dev/null; then
  range="${previous_tag}..${TAG}"
elif [ -n "$previous_tag" ]; then
  range="${previous_tag}..HEAD"
else
  range="HEAD"
fi

# section key -> heading, in display order.
sections="feat fix perf refactor docs test chore other"

heading_for() {
  case "$1" in
    feat) echo "✨ 新功能" ;;
    fix) echo "🐛 修复" ;;
    perf) echo "⚡ 性能" ;;
    refactor) echo "♻️ 重构" ;;
    docs) echo "📝 文档" ;;
    test) echo "✅ 测试" ;;
    chore) echo "🔧 构建 / 杂项" ;;
    *) echo "📌 其它" ;;
  esac
}

classify() {
  case "$1" in
    feat*) echo "feat" ;;
    fix*) echo "fix" ;;
    perf*) echo "perf" ;;
    refactor*) echo "refactor" ;;
    docs*) echo "docs" ;;
    test*) echo "test" ;;
    chore*|build*|ci*|style*|revert*) echo "chore" ;;
    *) echo "other" ;;
  esac
}

# "fix(macos): do a thing" -> "do a thing";  "fix: do a thing" -> "do a thing"
clean_subject() {
  printf '%s' "$1" | sed -E 's/^[a-zA-Z]+(\([^)]*\))?!?:[[:space:]]*//'
}

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

git log --no-merges --reverse --max-count=200 --pretty=format:'%s%x1f%h' "$range" |
  while IFS=$'\x1f' read -r subject short || [ -n "$subject" ]; do
    [ -n "$subject" ] || continue
    cleaned=$(printf '%s' "$subject" | strip_noise)
    cleaned=$(clean_subject "$cleaned")
    # Skip pure version-bump commits that reduce to nothing useful.
    [ -n "$(printf '%s' "$cleaned" | tr -d '[:space:]')" ] || continue
    section=$(classify "$subject")
    printf '%s\t- %s (`%s`)\n' "$section" "$cleaned" "$short" >>"$tmp/entries"
  done

{
  echo "### 📝 更新内容"
  echo
  if [ -s "$tmp/entries" ]; then
    for section in $sections; do
      if grep -q "^${section}	" "$tmp/entries" 2>/dev/null; then
        echo "#### $(heading_for "$section")"
        echo
        grep "^${section}	" "$tmp/entries" | cut -f2-
        echo
      fi
    done
    echo "<details><summary>提交范围</summary>"
    echo
    echo "\`${range}\`"
    echo
    echo "</details>"
  else
    echo "_本版本没有可列出的提交记录。_"
  fi

  cat <<'FOOTER'

---

### 📦 下载对应系统安装包

| 平台 | 文件 |
| --- | --- |
| 🍏 macOS (Universal，同时兼容 Apple Silicon 与 Intel) | `*.dmg` |
| 🪟 Windows (x64) | `*-setup.exe` |
| 🐧 Linux (x64) | `*.AppImage` / `*.deb` |

### ⚠️ macOS 提示「已损坏，无法打开」

开源客户端未购买 Apple 商业签名证书，首次从浏览器下载安装后 Gatekeeper 会拦截。
把软件拖入「应用程序」后，在「终端」执行：

```bash
sudo xattr -rd com.apple.quarantine /Applications/EnvHub.app
```

### ⚠️ Windows SmartScreen 提示

安装包未做代码签名，首次运行可能出现「Windows 已保护你的电脑」，点「更多信息」→「仍要运行」即可。
FOOTER
} >"$OUT"

if [ "$OUT" != "/dev/stdout" ]; then
  echo "wrote release notes to $OUT (range: $range)" >&2
fi
