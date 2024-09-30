# Shute

A library for easily running compute shaders.

The goal of this library is to enable a similar workflow for GPGPU programming as what CUDA offers. That is:
- Create a kernel (or in this library's case, a compute shader)
- Initialize buffers that will be passed between the CPU and GPU
- Send the buffers to the GPU and execute the kernel.
- Retrieve the buffers after the GPU has completed its work.

You can find examples in the [examples](./examples) directory.

Proper documentation coming soon.
