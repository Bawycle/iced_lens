# AI Deblurring Models Comparison

> Research document for evaluating AI models to enhance video frame capture in IcedLens.

## Overview

Two approaches exist for reducing motion blur:
- **Single-frame deblurring**: Analyzes one frame to remove blur
- **Multi-frame restoration (MFSR)**: Uses temporal information from neighboring frames

---

## Single-Frame Deblurring Models

### NAFNet

| Criteria | Details |
|----------|---------|
| **Type** | Single-frame deblurring |
| **Source** | [GitHub: megvii-research/NAFNet](https://github.com/megvii-research/NAFNet) |
| **HuggingFace** | [opencv/deblurring_nafnet](https://huggingface.co/opencv/deblurring_nafnet) |
| **License** | MIT |
| **Model Size** | ~30 MB (width32), ~65 MB (width64) |
| **ONNX Available** | Yes (official) |
| **Quantized Version** | Yes ([Quantized-NAFNet](https://github.com/AnonFox2/Quantized-NAFNet)) |

**Performance (GoPro benchmark):**
- PSNR: 32.87 dB (width32) / 33.71 dB (width64)
- Achieves SOTA with only 8.4% computational cost of previous best

**Rust Integration:**
- ONNX model available, works with `ort` crate
- OpenCV integration available (C++ demo provided)
- Quantized version: 75% fewer parameters, 16x faster inference

**Pros:**
- Lightweight and fast
- MIT license (permissive)
- Official ONNX support
- Good documentation

**Cons:**
- Single-frame only (no temporal info)

---

### Restormer

| Criteria | Details |
|----------|---------|
| **Type** | Single-frame deblurring (Transformer-based) |
| **Source** | [GitHub: swz30/Restormer](https://github.com/swz30/Restormer) |
| **HuggingFace** | [swz30/Restormer](https://huggingface.co/swz30/Restormer) |
| **License** | Apache 2.0 |
| **Model Size** | ~100 MB |
| **ONNX Available** | Community conversions |
| **Quantized Version** | No official |

**Performance (GoPro benchmark):**
- PSNR: 32.92 dB
- CVPR 2022 Oral - SOTA for motion deblurring, deraining, denoising

**Rust Integration:**
- Requires ONNX conversion (not officially provided)
- Transformer architecture may be slower on CPU

**Pros:**
- Excellent quality, especially for text recovery
- Versatile (deblur, denoise, derain)

**Cons:**
- Larger model
- No official ONNX
- Slower inference (Transformer overhead)

---

### DeblurGAN-v2

| Criteria | Details |
|----------|---------|
| **Type** | Single-frame deblurring (GAN-based) |
| **Source** | [GitHub: VITA-Group/DeblurGAN-v2](https://github.com/VITA-Group/DeblurGAN-v2) |
| **HuggingFace** | Limited availability |
| **License** | BSD |
| **Model Size** | ~60 MB (MobileNet backbone) |
| **ONNX Available** | Community conversions |
| **Quantized Version** | No |

**Performance (GoPro benchmark):**
- PSNR: 29.55 dB (MobileNet) / 30.40 dB (Inception-ResNet)
- 10-100x faster than competitors (2019)

**Rust Integration:**
- Requires manual ONNX conversion
- Lightweight backbones available

**Pros:**
- Very fast (real-time capable)
- Multiple backbone options (speed vs quality tradeoff)

**Cons:**
- Lower quality than newer models
- GAN artifacts possible
- Dated (2019)

---

### MIMO-UNet

| Criteria | Details |
|----------|---------|
| **Type** | Single-frame deblurring (Multi-scale) |
| **Source** | [GitHub: chosj95/MIMO-UNet](https://github.com/chosj95/MIMO-UNet) |
| **HuggingFace** | Not available |
| **License** | Apache 2.0 |
| **Model Size** | ~16 MB (MIMO-UNet), ~65 MB (MIMO-UNet+) |
| **ONNX Available** | Community conversions |
| **Quantized Version** | No |

**Performance (GoPro benchmark):**
- PSNR: 31.73 dB (MIMO-UNet) / 32.45 dB (MIMO-UNet+)
- Good at preserving fine details

**Rust Integration:**
- Requires manual ONNX conversion
- Smallest model size

**Pros:**
- Very lightweight
- Good detail preservation
- Simple architecture

**Cons:**
- No official ONNX
- Less documentation

---

## Multi-Frame Restoration Models (MFSR)

### BasicVSR++

| Criteria | Details |
|----------|---------|
| **Type** | Multi-frame video super-resolution/restoration |
| **Source** | [GitHub: ckkelvinchan/BasicVSR_PlusPlus](https://github.com/ckkelvinchan/BasicVSR_PlusPlus) |
| **Toolbox** | [BasicSR](https://github.com/XPixelGroup/BasicSR) |
| **License** | Apache 2.0 |
| **Model Size** | ~80 MB |
| **ONNX Available** | Community conversions |
| **Frames Used** | Bidirectional (past + future frames) |

**Performance:**
- NTIRE 2021: 3 champions, 1 runner-up
- Surpasses BasicVSR by 0.82 dB PSNR
- Works for super-resolution AND deblurring

**Rust Integration:**
- Part of BasicSR/MMagic ecosystem
- Requires sequence of frames as input
- More complex integration

**Pros:**
- Best quality for video
- Uses temporal information (exactly what we want)
- Well maintained

**Cons:**
- Requires multiple frames
- More complex pipeline
- Higher memory usage

---

### EDVR

| Criteria | Details |
|----------|---------|
| **Type** | Multi-frame video restoration |
| **Source** | [GitHub: xinntao/EDVR](https://github.com/xinntao/EDVR) |
| **Toolbox** | Merged into [BasicSR](https://github.com/XPixelGroup/BasicSR) |
| **License** | Apache 2.0 |
| **Model Size** | ~50 MB |
| **ONNX Available** | Community conversions |
| **Frames Used** | 5-7 frames typically |

**Performance:**
- NTIRE 2019 Winner (all 4 tracks)
- Excellent for large motion handling

**Rust Integration:**
- Similar to BasicVSR++
- Deformable convolutions may complicate ONNX conversion

**Pros:**
- Proven track record
- Handles large motions well
- Part of BasicSR ecosystem

**Cons:**
- Older than BasicVSR++ (2019)
- Deformable conv ONNX issues

---

### Real-ESRGAN

| Criteria | Details |
|----------|---------|
| **Type** | Single/Multi-frame super-resolution + restoration |
| **Source** | [GitHub: xinntao/Real-ESRGAN](https://github.com/xinntao/Real-ESRGAN) |
| **HuggingFace** | [Multiple models available](https://huggingface.co/models?search=real-esrgan) |
| **License** | BSD |
| **Model Size** | ~65 MB |
| **ONNX Available** | Yes (community + official ncnn) |
| **Video Support** | Yes (frame-by-frame or with temporal) |

**Performance:**
- PSNR: 24.97 dB, SSIM: 0.76 (facial images)
- Designed for real-world degradations

**Rust Integration:**
- Most popular, best community support
- ONNX models widely available
- [realesrgan-ncnn-vulkan](https://github.com/xinntao/Real-ESRGAN-ncnn-vulkan) for GPU

**Pros:**
- Most mature ecosystem
- Handles unknown/mixed degradations
- Good for real-world use (not just benchmarks)
- ncnn/Vulkan version for cross-platform GPU

**Cons:**
- Primarily upscaling, deblur is secondary
- May over-sharpen

---

## Comparison Summary

### Quality (GoPro Benchmark PSNR)

| Model | PSNR (dB) | Type |
|-------|-----------|------|
| NAFNet (width64) | 33.71 | Single-frame |
| Restormer | 32.92 | Single-frame |
| MIMO-UNet+ | 32.45 | Single-frame |
| BasicVSR++ | ~34+ | Multi-frame |
| EDVR | ~33 | Multi-frame |
| DeblurGAN-v2 | 30.40 | Single-frame |

### Speed (relative, CPU inference)

| Model | Speed | Notes |
|-------|-------|-------|
| DeblurGAN-v2 | Fastest | Real-time capable |
| MIMO-UNet | Very Fast | Smallest model |
| NAFNet (width32) | Fast | Good balance |
| NAFNet (width64) | Medium | Better quality |
| Real-ESRGAN | Medium | Depends on model |
| EDVR | Slow | Multi-frame overhead |
| BasicVSR++ | Slow | Multi-frame overhead |
| Restormer | Slowest | Transformer overhead |

### Rust Integration Difficulty

| Model | Difficulty | Notes |
|-------|------------|-------|
| NAFNet | Easy | Official ONNX, OpenCV support |
| Real-ESRGAN | Easy | Mature ecosystem, ncnn option |
| MIMO-UNet | Medium | Manual ONNX conversion |
| DeblurGAN-v2 | Medium | Manual ONNX conversion |
| Restormer | Hard | No official ONNX, Transformer ops |
| EDVR | Hard | Deformable convolutions |
| BasicVSR++ | Hard | Complex temporal pipeline |

### Resource Consumption

| Model | VRAM (GPU) | RAM (CPU) | Notes |
|-------|------------|-----------|-------|
| MIMO-UNet | ~1 GB | ~500 MB | Lightest |
| NAFNet (w32) | ~1.5 GB | ~800 MB | Light |
| DeblurGAN-v2 | ~2 GB | ~1 GB | Moderate |
| NAFNet (w64) | ~2.5 GB | ~1.2 GB | Moderate |
| Real-ESRGAN | ~3 GB | ~1.5 GB | Moderate |
| Restormer | ~4 GB | ~2 GB | Heavy |
| EDVR | ~6 GB | ~3 GB | Heavy (multi-frame) |
| BasicVSR++ | ~8 GB | ~4 GB | Heaviest |

---

## Recommendations for IcedLens

### Best Choice: NAFNet

**Why:**
1. Official ONNX support (easiest Rust integration via `ort` crate)
2. MIT license (most permissive)
3. Excellent quality/speed tradeoff
4. Quantized version available for resource-constrained systems
5. OpenCV integration if needed

### Alternative: Real-ESRGAN

**Why:**
1. Mature ecosystem with lots of community support
2. Handles real-world degradations well
3. ncnn/Vulkan version for cross-platform GPU acceleration
4. More versatile (upscaling + restoration)

### Future Consideration: BasicVSR++

**Why:**
1. Best quality using temporal information
2. Exactly matches the "analyze frames before/after" concept
3. Worth investigating once single-frame is working

---

## Implementation Path

### Phase 1: Proof of Concept
1. Download NAFNet ONNX model from HuggingFace
2. Test with `ort` crate in a standalone Rust binary
3. Measure performance on test images

### Phase 2: Integration
1. Add optional feature flag `--features ai-enhance`
2. Download model on first use (or bundle with AppImage)
3. Add "Enhance Frame" button in frame capture dialog

### Phase 3: Optimization
1. Test quantized NAFNet for faster inference
2. Consider GPU acceleration (CUDA/Vulkan)
3. Evaluate multi-frame models if quality insufficient

---

## References

- [NAFNet GitHub](https://github.com/megvii-research/NAFNet)
- [BasicSR Toolbox](https://github.com/XPixelGroup/BasicSR)
- [Real-ESRGAN GitHub](https://github.com/xinntao/Real-ESRGAN)
- [Papers With Code - Image Deblurring](https://paperswithcode.com/task/image-deblurring)
- [MMagic Documentation](https://mmagic.readthedocs.io/en/stable/model_zoo/deblurring.html)
- [Video Upscalers Benchmark](https://videoprocessing.ai/benchmarks/video-upscalers.html)
