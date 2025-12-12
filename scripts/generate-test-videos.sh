#!/bin/bash
# SPDX-License-Identifier: MPL-2.0
# Generate test video files for integration tests
#
# This script uses FFmpeg to create minimal test videos in various formats.
# Generated files are placed in tests/data/ directory (videos gitignored).
#
# Usage: ./scripts/generate-test-videos.sh

set -e  # Exit on error

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}🎬 Generating test video files...${NC}\n"

# Check if FFmpeg is installed
if ! command -v ffmpeg &> /dev/null; then
    echo -e "${YELLOW}⚠️  FFmpeg not found. Please install FFmpeg:${NC}"
    echo "   Ubuntu/Debian: sudo apt-get install ffmpeg"
    echo "   Fedora:        sudo dnf install ffmpeg"
    echo "   macOS:         brew install ffmpeg"
    exit 1
fi

# Create tests/data directory if it doesn't exist
mkdir -p tests/data

# Generate 720p MP4 with audio (H.264 + AAC)
echo -e "${GREEN}[1/7]${NC} Generating sample.mp4 (720p, 10s, H.264+AAC)..."
ffmpeg -f lavfi -i testsrc=duration=10:size=1280x720:rate=30 \
       -f lavfi -i sine=frequency=440:duration=10 \
       -c:v libx264 -preset ultrafast -pix_fmt yuv420p \
       -c:a aac -b:a 128k \
       -y tests/data/sample.mp4 \
       -loglevel error

# Generate MP4 without audio
echo -e "${GREEN}[2/7]${NC} Generating sample_no_audio.mp4 (720p, 5s, H.264 only)..."
ffmpeg -f lavfi -i testsrc=duration=5:size=1280x720:rate=30 \
       -c:v libx264 -preset ultrafast -pix_fmt yuv420p \
       -y tests/data/sample_no_audio.mp4 \
       -loglevel error

# Generate MP4 with audio (for audio detection test)
echo -e "${GREEN}[3/7]${NC} Generating sample_with_audio.mp4 (720p, 5s, H.264+AAC)..."
ffmpeg -f lavfi -i testsrc=duration=5:size=1280x720:rate=30 \
       -f lavfi -i sine=frequency=880:duration=5 \
       -c:v libx264 -preset ultrafast -pix_fmt yuv420p \
       -c:a aac -b:a 128k \
       -y tests/data/sample_with_audio.mp4 \
       -loglevel error

# Generate AVI (MPEG-4)
echo -e "${GREEN}[4/7]${NC} Generating sample.avi (720p, 5s, MPEG-4)..."
ffmpeg -f lavfi -i testsrc=duration=5:size=1280x720:rate=30 \
       -c:v mpeg4 -q:v 5 \
       -y tests/data/sample.avi \
       -loglevel error

# Generate WebM (VP9 + Opus)
echo -e "${GREEN}[5/7]${NC} Generating sample.webm (720p, 5s, VP9+Opus)..."
ffmpeg -f lavfi -i testsrc=duration=5:size=1280x720:rate=30 \
       -f lavfi -i sine=frequency=660:duration=5 \
       -c:v libvpx-vp9 -b:v 1M \
       -c:a libopus -b:a 128k \
       -y tests/data/sample.webm \
       -loglevel error

# Generate MOV (H.264 + AAC in QuickTime container)
echo -e "${GREEN}[6/7]${NC} Generating sample.mov (720p, 5s, H.264+AAC)..."
ffmpeg -f lavfi -i testsrc=duration=5:size=1280x720:rate=30 \
       -f lavfi -i sine=frequency=550:duration=5 \
       -c:v libx264 -preset ultrafast -pix_fmt yuv420p \
       -c:a aac -b:a 128k \
       -y tests/data/sample.mov \
       -loglevel error

# Generate MKV (H.264 + AAC in Matroska container)
echo -e "${GREEN}[7/7]${NC} Generating sample.mkv (720p, 5s, H.264+AAC)..."
ffmpeg -f lavfi -i testsrc=duration=5:size=1280x720:rate=30 \
       -f lavfi -i sine=frequency=770:duration=5 \
       -c:v libx264 -preset ultrafast -pix_fmt yuv420p \
       -c:a aac -b:a 128k \
       -y tests/data/sample.mkv \
       -loglevel error

# Generate a small corrupted MP4 for error testing
echo -e "${GREEN}[Bonus]${NC} Generating corrupted.mp4 (invalid file)..."
dd if=/dev/urandom of=tests/data/corrupted.mp4 bs=1024 count=10 2>/dev/null

echo -e "\n${BLUE}✅ Test video generation complete!${NC}"
echo -e "\nGenerated files in tests/data/:"
ls -lh tests/data/*.{mp4,avi,webm,mov,mkv} 2>/dev/null || true

echo -e "\n${BLUE}📊 File details:${NC}"
for file in tests/data/sample.{mp4,avi,webm,mov,mkv}; do
    if [ -f "$file" ]; then
        echo -e "\n${GREEN}$(basename "$file"):${NC}"
        ffprobe -v error -show_entries format=duration,size,bit_rate \
                -show_entries stream=codec_name,width,height \
                -of default=noprint_wrappers=1 "$file" 2>/dev/null | grep -E '(codec_name|width|height|duration)' || true
    fi
done

echo -e "\n${BLUE}🧪 To run integration tests:${NC}"
echo "   cargo test --test video_integration -- --ignored --nocapture"
echo ""
