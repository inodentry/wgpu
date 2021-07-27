use crate::{
    binding_model::{CreateBindGroupLayoutError, CreatePipelineLayoutError},
    device::{DeviceError, MissingDownlevelFlags, MissingFeatures, RenderPassContext},
    hub::Resource,
    id::{DeviceId, PipelineLayoutId, ShaderModuleId},
    validation, Label, LifeGuard, Stored,
};
use std::borrow::Cow;
use thiserror::Error;

pub enum ShaderModuleSource<'a> {
    Wgsl(Cow<'a, str>),
    Naga(naga::Module),
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "trace", derive(serde::Serialize))]
#[cfg_attr(feature = "replay", derive(serde::Deserialize))]
pub struct ShaderModuleDescriptor<'a> {
    pub label: Label<'a>,
}

#[derive(Debug)]
pub struct ShaderModule<A: hal::Api> {
    pub(crate) raw: A::ShaderModule,
    pub(crate) device_id: Stored<DeviceId>,
    pub(crate) interface: Option<validation::Interface>,
    #[cfg(debug_assertions)]
    pub(crate) label: String,
}

impl<A: hal::Api> Resource for ShaderModule<A> {
    const TYPE: &'static str = "ShaderModule";

    fn life_guard(&self) -> &LifeGuard {
        unreachable!()
    }

    fn label(&self) -> &str {
        #[cfg(debug_assertions)]
        return &self.label;
        #[cfg(not(debug_assertions))]
        return "";
    }
}

#[derive(Clone, Debug, Error)]
pub enum CreateShaderModuleError {
    #[error("Failed to parse a shader")]
    Parsing,
    #[error("Failed to generate the backend-specific code")]
    Generation,
    #[error(transparent)]
    Device(#[from] DeviceError),
    #[error(transparent)]
    Validation(#[from] naga::valid::ValidationError),
    #[error(transparent)]
    MissingFeatures(#[from] MissingFeatures),
}

/// Describes a programmable pipeline stage.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "trace", derive(serde::Serialize))]
#[cfg_attr(feature = "replay", derive(serde::Deserialize))]
pub struct ProgrammableStageDescriptor<'a> {
    /// The compiled shader module for this stage.
    pub module: ShaderModuleId,
    /// The name of the entry point in the compiled shader. There must be a function that returns
    /// void with this name in the shader.
    pub entry_point: Cow<'a, str>,
}

/// Number of implicit bind groups derived at pipeline creation.
pub type ImplicitBindGroupCount = u8;

#[derive(Clone, Debug, Error)]
pub enum ImplicitLayoutError {
    #[error("missing IDs for deriving {0} bind groups")]
    MissingIds(ImplicitBindGroupCount),
    #[error("unable to reflect the shader {0:?} interface")]
    ReflectionError(wgt::ShaderStages),
    #[error(transparent)]
    BindGroup(#[from] CreateBindGroupLayoutError),
    #[error(transparent)]
    Pipeline(#[from] CreatePipelineLayoutError),
}

/// Describes a compute pipeline.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "trace", derive(serde::Serialize))]
#[cfg_attr(feature = "replay", derive(serde::Deserialize))]
pub struct ComputePipelineDescriptor<'a> {
    pub label: Label<'a>,
    /// The layout of bind groups for this pipeline.
    pub layout: Option<PipelineLayoutId>,
    /// The compiled compute stage and its entry point.
    pub stage: ProgrammableStageDescriptor<'a>,
}

#[derive(Clone, Debug, Error)]
pub enum CreateComputePipelineError {
    #[error(transparent)]
    Device(#[from] DeviceError),
    #[error("pipeline layout is invalid")]
    InvalidLayout,
    #[error("unable to derive an implicit layout")]
    Implicit(#[from] ImplicitLayoutError),
    #[error("error matching shader requirements against the pipeline")]
    Stage(#[from] validation::StageError),
    #[error("Internal error: {0}")]
    Internal(String),
    #[error(transparent)]
    MissingDownlevelFlags(#[from] MissingDownlevelFlags),
}

#[derive(Debug)]
pub struct ComputePipeline<A: hal::Api> {
    pub(crate) raw: A::ComputePipeline,
    pub(crate) layout_id: Stored<PipelineLayoutId>,
    pub(crate) device_id: Stored<DeviceId>,
    pub(crate) life_guard: LifeGuard,
}

impl<A: hal::Api> Resource for ComputePipeline<A> {
    const TYPE: &'static str = "ComputePipeline";

    fn life_guard(&self) -> &LifeGuard {
        &self.life_guard
    }
}

/// Describes how the vertex buffer is interpreted.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "trace", derive(serde::Serialize))]
#[cfg_attr(feature = "replay", derive(serde::Deserialize))]
pub struct VertexBufferLayout<'a> {
    /// The stride, in bytes, between elements of this buffer.
    pub array_stride: wgt::BufferAddress,
    /// How often this vertex buffer is "stepped" forward.
    pub step_mode: wgt::VertexStepMode,
    /// The list of attributes which comprise a single vertex.
    pub attributes: Cow<'a, [wgt::VertexAttribute]>,
}

/// Describes the vertex process in a render pipeline.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "trace", derive(serde::Serialize))]
#[cfg_attr(feature = "replay", derive(serde::Deserialize))]
pub struct VertexState<'a> {
    /// The compiled vertex stage and its entry point.
    pub stage: ProgrammableStageDescriptor<'a>,
    /// The format of any vertex buffers used with this pipeline.
    pub buffers: Cow<'a, [VertexBufferLayout<'a>]>,
}

/// Describes fragment processing in a render pipeline.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "trace", derive(serde::Serialize))]
#[cfg_attr(feature = "replay", derive(serde::Deserialize))]
pub struct FragmentState<'a> {
    /// The compiled fragment stage and its entry point.
    pub stage: ProgrammableStageDescriptor<'a>,
    /// The effect of draw calls on the color aspect of the output target.
    pub targets: Cow<'a, [wgt::ColorTargetState]>,
}

/// Describes a render (graphics) pipeline.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "trace", derive(serde::Serialize))]
#[cfg_attr(feature = "replay", derive(serde::Deserialize))]
pub struct RenderPipelineDescriptor<'a> {
    pub label: Label<'a>,
    /// The layout of bind groups for this pipeline.
    pub layout: Option<PipelineLayoutId>,
    /// The vertex processing state for this pipeline.
    pub vertex: VertexState<'a>,
    /// The properties of the pipeline at the primitive assembly and rasterization level.
    #[cfg_attr(any(feature = "replay", feature = "trace"), serde(default))]
    pub primitive: wgt::PrimitiveState,
    /// The effect of draw calls on the depth and stencil aspects of the output target, if any.
    #[cfg_attr(any(feature = "replay", feature = "trace"), serde(default))]
    pub depth_stencil: Option<wgt::DepthStencilState>,
    /// The multi-sampling properties of the pipeline.
    #[cfg_attr(any(feature = "replay", feature = "trace"), serde(default))]
    pub multisample: wgt::MultisampleState,
    /// The fragment processing state for this pipeline.
    pub fragment: Option<FragmentState<'a>>,
}

#[derive(Clone, Debug, Error)]
pub enum ColorStateError {
    #[error("output is missing")]
    Missing,
    #[error("format {0:?} is not renderable")]
    FormatNotRenderable(wgt::TextureFormat),
    #[error("format {0:?} is not blendable")]
    FormatNotBlendable(wgt::TextureFormat),
    #[error("format {0:?} does not have a color aspect")]
    FormatNotColor(wgt::TextureFormat),
    #[error("output format {pipeline} is incompatible with the shader {shader}")]
    IncompatibleFormat {
        pipeline: validation::NumericType,
        shader: validation::NumericType,
    },
    #[error("blend factors for {0:?} must be `One`")]
    InvalidMinMaxBlendFactors(wgt::BlendComponent),
}

#[derive(Clone, Debug, Error)]
pub enum DepthStencilStateError {
    #[error("format {0:?} is not renderable")]
    FormatNotRenderable(wgt::TextureFormat),
    #[error("format {0:?} does not have a depth aspect, but depth test/write is enabled")]
    FormatNotDepth(wgt::TextureFormat),
    #[error("format {0:?} does not have a stencil aspect, but stencil test/write is enabled")]
    FormatNotStencil(wgt::TextureFormat),
}

#[derive(Clone, Debug, Error)]
pub enum CreateRenderPipelineError {
    #[error(transparent)]
    Device(#[from] DeviceError),
    #[error("pipeline layout is invalid")]
    InvalidLayout,
    #[error("unable to derive an implicit layout")]
    Implicit(#[from] ImplicitLayoutError),
    #[error("color state [{0}] is invalid")]
    ColorState(u8, #[source] ColorStateError),
    #[error("depth/stencil state is invalid")]
    DepthStencilState(#[from] DepthStencilStateError),
    #[error("invalid sample count {0}")]
    InvalidSampleCount(u32),
    #[error("the number of vertex buffers {given} exceeds the limit {limit}")]
    TooManyVertexBuffers { given: u32, limit: u32 },
    #[error("the total number of vertex attributes {given} exceeds the limit {limit}")]
    TooManyVertexAttributes { given: u32, limit: u32 },
    #[error("vertex buffer {index} stride {given} exceeds the limit {limit}")]
    VertexStrideTooLarge { index: u32, given: u32, limit: u32 },
    #[error("vertex buffer {index} stride {stride} does not respect `VERTEX_STRIDE_ALIGNMENT`")]
    UnalignedVertexStride {
        index: u32,
        stride: wgt::BufferAddress,
    },
    #[error("vertex attribute at location {location} has invalid offset {offset}")]
    InvalidVertexAttributeOffset {
        location: wgt::ShaderLocation,
        offset: wgt::BufferAddress,
    },
    #[error("strip index format was not set to None but to {strip_index_format:?} while using the non-strip topology {topology:?}")]
    StripIndexFormatForNonStripTopology {
        strip_index_format: Option<wgt::IndexFormat>,
        topology: wgt::PrimitiveTopology,
    },
    #[error("Conservative Rasterization is only supported for wgt::PolygonMode::Fill")]
    ConservativeRasterizationNonFillPolygonMode,
    #[error(transparent)]
    MissingFeatures(#[from] MissingFeatures),
    #[error(transparent)]
    MissingDownlevelFlags(#[from] MissingDownlevelFlags),
    #[error("error matching {stage:?} shader requirements against the pipeline")]
    Stage {
        stage: wgt::ShaderStages,
        #[source]
        error: validation::StageError,
    },
    #[error("Internal error in {stage:?} shader: {error}")]
    Internal {
        stage: wgt::ShaderStages,
        error: String,
    },
}

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct PipelineFlags: u32 {
        const BLEND_CONSTANT = 1 << 0;
        const STENCIL_REFERENCE = 1 << 1;
        const WRITES_DEPTH_STENCIL = 1 << 2;
    }
}

#[derive(Debug)]
pub struct RenderPipeline<A: hal::Api> {
    pub(crate) raw: A::RenderPipeline,
    pub(crate) layout_id: Stored<PipelineLayoutId>,
    pub(crate) device_id: Stored<DeviceId>,
    pub(crate) pass_context: RenderPassContext,
    pub(crate) flags: PipelineFlags,
    pub(crate) strip_index_format: Option<wgt::IndexFormat>,
    pub(crate) vertex_strides: Vec<(wgt::BufferAddress, wgt::VertexStepMode)>,
    pub(crate) life_guard: LifeGuard,
}

impl<A: hal::Api> Resource for RenderPipeline<A> {
    const TYPE: &'static str = "RenderPipeline";

    fn life_guard(&self) -> &LifeGuard {
        &self.life_guard
    }
}
