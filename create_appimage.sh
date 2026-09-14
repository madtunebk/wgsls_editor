#!/bin/bash
set -e

echo "==================================="
echo "Building WebShard Editor AppImage"
echo "==================================="

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------
APP_NAME="webshard_editor"                        # binary name / icon name
DEVELOPER_ID="io.github.madtunebk"                # reverse-DNS namespace
DEVELOPER_NAME="madtunebk"
HOMEPAGE="https://github.com/madtunebk/wgsls_editor"
APP_ID="${DEVELOPER_ID}.${APP_NAME}"              # io.github.madtunebk.webshard_editor
APPDIR="create_app/AppDir"
APPIMAGETOOL="create_app/appimagetool-x86_64.AppImage"

# Set to false to skip AppStream validation at build time.
# Validation contacts the network to check <url> reachability, so an offline
# or flaky connection can otherwise fail an otherwise-fine build.
VALIDATE_APPSTREAM=true

# Parse arguments
FORCE_BUILD=false
SKIP_BUILD=false
for arg in "$@"; do
    case $arg in
        --force|-f)
            FORCE_BUILD=true
            shift
            ;;
        --skip-build|-s)
            SKIP_BUILD=true
            shift
            ;;
        --no-validate)
            VALIDATE_APPSTREAM=false
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo "Options:"
            echo "  --force, -f       Force rebuild even if sources unchanged"
            echo "  --skip-build, -s  Skip cargo build, use existing binary"
            echo "  --no-validate     Skip AppStream validation (NO_APPSTREAM=1)"
            echo "  --help, -h        Show this help message"
            exit 0
            ;;
    esac
done

# ---------------------------------------------------------------------------
# Step 1: Smart build system
# ---------------------------------------------------------------------------
if [ "$SKIP_BUILD" = true ]; then
    echo -e "${YELLOW}[1/6]${NC} Skipping cargo build (--skip-build flag)"

    if [ ! -f "target/release/$APP_NAME" ]; then
        echo -e "${RED}Error: Binary not found and --skip-build specified!${NC}"
        echo "Run without --skip-build to build first"
        exit 1
    fi

    echo -e "${GREEN}✓ Using existing binary${NC}"
else
    echo -e "${YELLOW}[1/6]${NC} Checking if rebuild is needed..."

    NEEDS_BUILD=false

    # Check if binary exists
    if [ ! -f "target/release/$APP_NAME" ]; then
        echo -e "${YELLOW}→ Binary not found, build required${NC}"
        NEEDS_BUILD=true
    elif [ "$FORCE_BUILD" = true ]; then
        echo -e "${YELLOW}→ Force build requested (--force flag)${NC}"
        NEEDS_BUILD=true
    else
        # Check if any Rust source files are newer than the binary
        BINARY_TIME=$(stat -c %Y "target/release/$APP_NAME" 2>/dev/null || echo 0)

        # Find newest source file
        NEWEST_SRC=$(find src -type f \( -name "*.rs" -o -name "*.toml" \) -printf '%T@\n' 2>/dev/null | sort -rn | head -1)
        NEWEST_SRC=${NEWEST_SRC:-0}
        NEWEST_SRC_INT=${NEWEST_SRC%.*}  # Remove decimal part

        # Also check Cargo.toml
        CARGO_TOML_TIME=$(stat -c %Y "Cargo.toml" 2>/dev/null || echo 0)

        if [ "$NEWEST_SRC_INT" -gt "$BINARY_TIME" ] || [ "$CARGO_TOML_TIME" -gt "$BINARY_TIME" ]; then
            echo -e "${YELLOW}→ Source files modified, rebuild required${NC}"
            NEEDS_BUILD=true
        else
            echo -e "${GREEN}→ Binary is up to date, skipping rebuild${NC}"
            echo -e "${GREEN}   (use --force to rebuild anyway)${NC}"
            NEEDS_BUILD=false
        fi
    fi

    if [ "$NEEDS_BUILD" = true ]; then
        echo -e "${YELLOW}→ Building release binary...${NC}"
        cargo build --release

        if [ ! -f "target/release/$APP_NAME" ]; then
            echo -e "${RED}Error: Release binary not found after build!${NC}"
            exit 1
        fi

        echo -e "${GREEN}✓ Release binary built successfully${NC}"
    else
        echo -e "${GREEN}✓ Using existing binary (no changes detected)${NC}"
    fi
fi

# ---------------------------------------------------------------------------
# Step 2: Copy binary to AppDir
# ---------------------------------------------------------------------------
echo -e "${YELLOW}[2/6]${NC} Copying binary to AppDir..."
mkdir -p "$APPDIR/usr/bin"
cp "target/release/$APP_NAME" "$APPDIR/usr/bin/"
chmod +x "$APPDIR/usr/bin/$APP_NAME"
echo -e "${GREEN}✓ Binary copied${NC}"

# ---------------------------------------------------------------------------
# Step 3: Icon (root icon + hicolor + a REAL .DirIcon, never a symlink)
# ---------------------------------------------------------------------------
echo -e "${YELLOW}[3/6]${NC} Preparing icon..."
ICON_APPDIR="$APPDIR/${APP_NAME}.png"
ICON_HICOLOR="$APPDIR/usr/share/icons/hicolor/256x256/apps/${APP_NAME}.png"
mkdir -p "$APPDIR/usr/share/icons/hicolor/256x256/apps"

# Prefer logo.png, then APP_NAME.png
ICON_SRC=""
for candidate in "src/assets/logo.png" "src/assets/${APP_NAME}.png"; do
    if [ -f "$candidate" ]; then
        ICON_SRC="$candidate"
        break
    fi
done

if [ -n "$ICON_SRC" ]; then
    cp "$ICON_SRC" "$ICON_APPDIR"
    cp "$ICON_SRC" "$ICON_HICOLOR"
    echo -e "${GREEN}✓ Icon copied from $ICON_SRC${NC}"
elif [ ! -f "$ICON_APPDIR" ]; then
    echo -e "${YELLOW}⚠ Icon not found, generating placeholder with ImageMagick${NC}"
    convert -size 256x256 \
        xc:'#1a1a2e' \
        -fill '#e94560' -draw "circle 128,128 128,60" \
        -fill '#0f3460' -draw "circle 128,128 128,80" \
        -fill '#16213e' -draw "circle 128,128 128,100" \
        -fill '#e94560' -pointsize 48 -gravity center \
        -annotate 0 'WS' \
        "$ICON_APPDIR" 2>/dev/null || \
    convert -size 256x256 xc:'#1a1a2e' \
        -fill '#e94560' -draw "rectangle 40,40 216,216" \
        -fill '#16213e' -draw "rectangle 60,60 196,196" \
        "$ICON_APPDIR"
    cp "$ICON_APPDIR" "$ICON_HICOLOR"
    echo -e "${GREEN}✓ Placeholder icon generated${NC}"
else
    cp "$ICON_APPDIR" "$ICON_HICOLOR"
    echo -e "${GREEN}✓ Icon already in AppDir${NC}"
fi

# Guarantee .DirIcon is a real file, never a stale/broken symlink.
# (appimagetool would otherwise leave a symlink that confuses `file` and some tools.)
rm -f "$APPDIR/.DirIcon"
cp "$ICON_APPDIR" "$APPDIR/.DirIcon"
echo -e "${GREEN}✓ .DirIcon written as a real file${NC}"

# ---------------------------------------------------------------------------
# Step 4: Desktop file (reverse-DNS name; always regenerated; stale files removed)
# ---------------------------------------------------------------------------
echo -e "${YELLOW}[4/6]${NC} Writing desktop file..."
mkdir -p "$APPDIR/usr/share/applications"

# Remove any stale non-rdns desktop files from earlier builds so appimagetool
# never sees two .desktop files at the AppDir root.
rm -f "$APPDIR/${APP_NAME}.desktop" \
      "$APPDIR/usr/share/applications/${APP_NAME}.desktop"

# Always (re)write the canonical rdns desktop file so its contents stay correct
# (single main category => no "more than one main category" hint).
cat > "$APPDIR/${APP_ID}.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=WebShard Editor
Exec=$APP_NAME
Icon=$APP_NAME
Categories=Development;3DGraphics;
Terminal=false
EOF
cp "$APPDIR/${APP_ID}.desktop" "$APPDIR/usr/share/applications/"
echo -e "${GREEN}✓ Desktop file written: ${APP_ID}.desktop${NC}"

# AppRun wrapper: prefer host's loader paths if needed
cat > "$APPDIR/AppRun" <<'APPRUN'
#!/usr/bin/env sh
set -e
HERE="$(dirname "$(readlink -f "$0")")"
BINARY_PATH="$HERE/usr/bin/webshard_editor"
for loader in \
  /lib64/ld-linux-x86-64.so.2 \
  /lib/x86_64-linux-gnu/ld-linux-x86-64.so.2 \
  /lib/ld-linux-x86-64.so.2; do
  if [ -x "$loader" ]; then
    exec "$loader" --library-path "$HERE/usr/lib:$HERE/usr/lib64:$LD_LIBRARY_PATH" "$BINARY_PATH" "$@"
  fi
done
exec "$BINARY_PATH" "$@"
APPRUN
chmod +x "$APPDIR/AppRun"
echo -e "${GREEN}✓ AppRun created${NC}"

# ---------------------------------------------------------------------------
# Step 5: AppStream metainfo
# Filename == APP_ID so it satisfies BOTH:
#   - appstreamcli  (filename must match the component <id>)
#   - appimagetool  (derives usr/share/metainfo/<desktop-basename>.appdata.xml)
# Exactly one file is kept, so no cross-file validation surprises.
# ---------------------------------------------------------------------------
echo -e "${YELLOW}[5/6]${NC} Writing AppStream metainfo..."
METAINFO_DIR="$APPDIR/usr/share/metainfo"
mkdir -p "$METAINFO_DIR"

# Nuke every stale metainfo file (old nobus/*.metainfo.xml, mismatched names, etc.)
rm -f "$METAINFO_DIR"/*.xml

cat > "$METAINFO_DIR/${APP_ID}.appdata.xml" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<component type="desktop-application">
  <id>${APP_ID}</id>
  <metadata_license>MIT</metadata_license>
  <project_license>MIT</project_license>
  <name>WebShard Editor</name>
  <summary>Real-time WGSL shader editor</summary>
  <description>
    <p>
      A modern WGSL shader editor built with Rust and egui, featuring
      real-time multi-pass shader compilation, syntax highlighting, and
      audio-reactive capabilities.
    </p>
  </description>
  <launchable type="desktop-id">${APP_ID}.desktop</launchable>
  <url type="homepage">${HOMEPAGE}</url>
  <developer id="${DEVELOPER_ID}">
    <name>${DEVELOPER_NAME}</name>
  </developer>
  <content_rating type="oars-1.1" />
</component>
EOF
echo -e "${GREEN}✓ Metainfo written: ${APP_ID}.appdata.xml${NC}"

# ---------------------------------------------------------------------------
# Step 6: Create AppImage
# ---------------------------------------------------------------------------
echo -e "${YELLOW}[6/6]${NC} Creating AppImage..."
if [ ! -f "$APPIMAGETOOL" ]; then
    echo -e "${RED}Error: appimagetool not found!${NC}"
    echo "Download it from: https://github.com/AppImage/AppImageKit/releases"
    exit 1
fi

chmod +x "$APPIMAGETOOL"

# Remove old AppImage if exists
rm -f "${APP_NAME}-x86_64.AppImage"

# Create the AppImage (optionally skipping the network-dependent validation)
if [ "$VALIDATE_APPSTREAM" = true ]; then
    echo -e "${YELLOW}→ AppStream validation enabled (use --no-validate to skip)${NC}"
    ( cd create_app && ARCH=x86_64 ./appimagetool-x86_64.AppImage AppDir "../${APP_NAME}-x86_64.AppImage" )
else
    echo -e "${YELLOW}→ AppStream validation skipped (NO_APPSTREAM=1)${NC}"
    ( cd create_app && ARCH=x86_64 NO_APPSTREAM=1 ./appimagetool-x86_64.AppImage AppDir "../${APP_NAME}-x86_64.AppImage" )
fi

# Try to find the generated AppImage in likely locations
APPIMAGE_PATH=""
CANDIDATES=("./${APP_NAME}-x86_64.AppImage" "create_app/${APP_NAME}-x86_64.AppImage" "create_app/../${APP_NAME}-x86_64.AppImage")
for p in "${CANDIDATES[@]}"; do
  if [ -f "$p" ]; then
    APPIMAGE_PATH="$p"
    break
  fi
done

# Fallback: search for any webshard_editor*.AppImage in repo (maxdepth 2)
if [ -z "$APPIMAGE_PATH" ]; then
  found=$(find . -maxdepth 2 -type f -name "${APP_NAME}*.AppImage" -print -quit || true)
  if [ -n "$found" ]; then
    APPIMAGE_PATH="$found"
  fi
fi

if [ -n "$APPIMAGE_PATH" ]; then
  # Move to repo root if needed
  if [ "$(realpath "$APPIMAGE_PATH")" != "$(realpath "./${APP_NAME}-x86_64.AppImage")" ]; then
    mv "$APPIMAGE_PATH" "./${APP_NAME}-x86_64.AppImage"
    APPIMAGE_PATH="./${APP_NAME}-x86_64.AppImage"
  fi
  chmod +x "$APPIMAGE_PATH"
  echo -e "${GREEN}✓ AppImage created successfully!${NC}"
  echo ""
  echo "==================================="
  echo -e "${GREEN}✓ Build complete!${NC}"
  echo "==================================="
  echo ""
  echo "AppImage location: $(pwd)/${APP_NAME}-x86_64.AppImage"
  echo "Size: $(du -h "${APP_NAME}-x86_64.AppImage" | cut -f1)"
  echo ""
  echo "To run: ./${APP_NAME}-x86_64.AppImage"
else
  echo -e "${RED}Error: AppImage creation failed!${NC}"
  echo "appimagetool may have printed success but no .AppImage was found in expected locations."
  echo "Check create_app/ for output or run appimagetool manually to inspect its output."
  exit 1
fi