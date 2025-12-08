// Shader uniforms structure (shared between legacy and multi-pass pipelines)
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShaderUniforms {
    pub time: f32,
    pub audio_bass: f32,
    pub audio_mid: f32,
    pub audio_high: f32,
    pub resolution: [f32; 2],
    pub gamma: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub _pad0: f32,  // Padding for alignment
}
