use bevy::{
    core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    prelude::*,
    render::{
        render_resource::{
            BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
            CachedRenderPipelineId, ColorTargetState, ColorWrites, FragmentState, MultisampleState,
            PipelineCache, PrimitiveState, RenderPipelineDescriptor, SamplerBindingType,
            ShaderStages, TextureFormat, TextureSampleType, TextureViewDimension,
        },
        renderer::RenderDevice,
        texture::BevyDefault,
    },
};

use crate::{BLIT_SHADER_HANDLE, FXAA_SHADER_HANDLE};

#[derive(Resource)]
pub struct FXAAPipeline {
    pub hdr_texture_bind_group: BindGroupLayout,
    pub fxaa_pipeline_id: CachedRenderPipelineId,
    pub blit_pipeline_id: CachedRenderPipelineId,
}

impl FromWorld for FXAAPipeline {
    fn from_world(render_world: &mut World) -> Self {
        let fxaa_texture_bind_group = render_world
            .resource::<RenderDevice>()
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("fxaa_texture_bind_group_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let fxaa_descriptor = RenderPipelineDescriptor {
            label: Some("fxaa pipeline".into()),
            layout: Some(vec![fxaa_texture_bind_group.clone()]),
            vertex: fullscreen_shader_vertex_state(),
            fragment: Some(FragmentState {
                shader: FXAA_SHADER_HANDLE.typed(),
                shader_defs: vec![],
                entry_point: "fs_main".into(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
        };

        let blit_descriptor = RenderPipelineDescriptor {
            label: Some("blit pipeline".into()),
            layout: Some(vec![fxaa_texture_bind_group.clone()]),
            vertex: fullscreen_shader_vertex_state(),
            fragment: Some(FragmentState {
                shader: BLIT_SHADER_HANDLE.typed(),
                shader_defs: vec![],
                entry_point: "fs_main".into(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
        };
        let mut cache = render_world.resource_mut::<PipelineCache>();
        FXAAPipeline {
            hdr_texture_bind_group: fxaa_texture_bind_group,
            fxaa_pipeline_id: cache.queue_render_pipeline(fxaa_descriptor),
            blit_pipeline_id: cache.queue_render_pipeline(blit_descriptor),
        }
    }
}
