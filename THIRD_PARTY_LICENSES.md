# Third-Party Licenses

IcedLens includes or links to the following third-party components.

## FFmpeg

**License:** LGPL 2.1 or later
**Website:** https://ffmpeg.org
**Source Code:** https://github.com/FFmpeg/FFmpeg

FFmpeg is a collection of libraries and tools to process multimedia content.
IcedLens uses FFmpeg for video decoding via dynamic linking (DLLs on Windows,
shared libraries on Linux).

Under the LGPL 2.1, you have the right to:
- Use this software freely
- Obtain the FFmpeg source code from the link above
- Replace the FFmpeg libraries with your own modified versions

The full LGPL 2.1 license text is available at:
https://www.gnu.org/licenses/old-licenses/lgpl-2.1.html

### FFmpeg License Header

```
FFmpeg is free software; you can redistribute it and/or modify
it under the terms of the GNU Lesser General Public License as published by
the Free Software Foundation; either version 2.1 of the License, or
(at your option) any later version.

FFmpeg is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
GNU Lesser General Public License for more details.
```

---

## DirectML

**License:** MIT License
**Copyright:** Microsoft Corporation
**Website:** https://github.com/microsoft/DirectML

DirectML is a high-performance, hardware-accelerated DirectX 12 library for
machine learning. IcedLens uses DirectML for AI inference acceleration on Windows.

```
MIT License

Copyright (c) Microsoft Corporation.

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

---

## ONNX Runtime

**License:** MIT License
**Copyright:** Microsoft Corporation
**Website:** https://onnxruntime.ai
**Source Code:** https://github.com/microsoft/onnxruntime

ONNX Runtime is a cross-platform inference engine for machine learning models.
IcedLens uses ONNX Runtime for AI-powered image enhancement (deblurring and upscaling).

```
MIT License

Copyright (c) Microsoft Corporation

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

---

## NAFNet

**License:** MIT License
**Copyright:** 2022 megvii-model
**Website:** https://github.com/megvii-research/NAFNet

NAFNet (Nonlinear Activation Free Network) is a neural network architecture for
image restoration. IcedLens downloads and uses the NAFNet ONNX model for
AI-powered image deblurring.

The full license text is available at:
https://github.com/megvii-research/NAFNet/blob/main/LICENSE

---

## Real-ESRGAN

**License:** BSD 3-Clause License
**Copyright:** 2021 Xintao Wang
**Website:** https://github.com/xinntao/Real-ESRGAN

Real-ESRGAN is a practical image restoration algorithm for general image/video
enhancement. IcedLens downloads and uses the Real-ESRGAN ONNX model for
AI-powered image upscaling.

The full license text is available at:
https://github.com/xinntao/Real-ESRGAN/blob/master/LICENSE

---

## Other Dependencies

IcedLens uses many other open-source Rust crates. Their licenses can be found in
the respective crate repositories linked from [crates.io](https://crates.io).
See [Cargo.toml](Cargo.toml) for the full list of dependencies.

Most Rust crates used by IcedLens are licensed under MIT or Apache 2.0.
