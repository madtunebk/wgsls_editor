# Image Texture Usage Guide

## Overview

The shader editor now supports loading image textures that can be accessed by all shader buffers (BufferA, BufferB, BufferC, BufferD, and MainImage) via the `iChannel0` uniform, similar to ShaderToy.

## How to Load an Image

1. **Via UI**: 
   - Click the 🖼 icon in the bottom-right corner OR
   - Click the "⚙️ Settings" button and use the "Load Image" button in the properties window
   
2. **File Picker**: Select any common image format (PNG, JPG, BMP, etc.)

3. **Automatic Updates**: The shader will automatically recompile when a new image is loaded

## Using Images in Shaders

### Basic Usage

```wgsl
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    
    // Sample the loaded image at UV coordinates
    let color = textureSample(iChannel0, iChannel0Sampler, uv);
    
    return color;
}
```

### Available Uniforms

The following uniforms are automatically available in all shaders:

```wgsl
@group(1) @binding(8) var iChannel0: texture_2d<f32>;        // User-loaded image
@group(1) @binding(9) var iChannel0Sampler: sampler;         // Linear filtering sampler
```

### Example: Image with Effects

```wgsl
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    
    // Sample the image
    var color = textureSample(iChannel0, iChannel0Sampler, uv);
    
    // Apply effects based on time
    color.rgb *= 0.5 + 0.5 * sin(iTime);
    
    // Distort UV coordinates
    let distorted_uv = uv + 0.01 * vec2<f32>(
        sin(uv.y * 10.0 + iTime),
        cos(uv.x * 10.0 + iTime)
    );
    
    let distorted_color = textureSample(iChannel0, iChannel0Sampler, distorted_uv);
    
    return mix(color, distorted_color, 0.5);
}
```

## Multi-Pass Rendering

All buffers (A, B, C, D) can access the loaded image:

```wgsl
// In BufferA
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let img = textureSample(iChannel0, iChannel0Sampler, in.uv);
    // Process image and store in BufferA
    return img * vec4<f32>(1.0, 0.5, 0.5, 1.0); // Red tint
}

// In MainImage
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let buffer_a = textureSample(iBufferA, iBufferASampler, in.uv);
    let original = textureSample(iChannel0, iChannel0Sampler, in.uv);
    
    // Combine processed buffer with original image
    return mix(buffer_a, original, 0.5);
}
```

## Technical Details

- **Texture Format**: RGBA8 sRGB
- **Filtering**: Linear (smooth) filtering with repeat wrapping
- **Fallback**: If no image is loaded, `iChannel0` contains a 1x1 black texture
- **UV Coordinates**: (0,0) = top-left, (1,1) = bottom-right

## Implementation Notes

### WGPU API Version

This implementation uses `TexelCopyBufferLayout` (wgpu 27.x) for writing texture data:

```rust
queue.write_texture(
    texture.as_image_copy(),
    &rgba_data,
    TexelCopyBufferLayout {
        offset: 0,
        bytes_per_row: Some(4 * width),
        rows_per_image: Some(height),
    },
    texture_size,
);
```

### Bind Group Layout

The user image texture is bound to:
- **@group(1)** - Texture resources group
- **@binding(8)** - iChannel0 texture
- **@binding(9)** - iChannel0 sampler

This matches the ShaderToy convention where `iChannel0` is the first user-provided texture/buffer.
