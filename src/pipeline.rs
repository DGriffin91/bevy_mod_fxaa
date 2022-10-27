use std::borrow::Cow;

use bevy::{
    core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    prelude::*,
    render::{
        camera::ExtractedCamera,
        render_resource::{
            BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
            CachedRenderPipelineId, ColorTargetState, ColorWrites, Extent3d, FragmentState,
            MultisampleState, PipelineCache, PrimitiveState, RenderPipelineDescriptor,
            SamplerBindingType, ShaderStages, TextureDescriptor, TextureDimension, TextureFormat,
            TextureSampleType, TextureUsages, TextureViewDimension,
        },
        renderer::RenderDevice,
        texture::{BevyDefault, CachedTexture, TextureCache},
        view::{ExtractedView, ViewTarget},
    },
    utils::HashMap,
};

use crate::{BLIT_SHADER_HANDLE, FXAA, FXAA_SHADER_HANDLE, LDR_SHADER_HANDLE};

#[derive(Resource)]
pub struct FXAAPipeline {
    pub texture_bind_group: BindGroupLayout,
    pub fxaa_ldr_pipeline_id: CachedRenderPipelineId,
    pub fxaa_hdr_pipeline_id: CachedRenderPipelineId,
    pub to_ldr_pipeline_id: CachedRenderPipelineId,
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

        let fxaa_ldr_descriptor = fullscreen_vertex_pipeline_descriptor(
            "fxaa ldr pipeline",
            &fxaa_texture_bind_group,
            FXAA_SHADER_HANDLE,
            vec![],
            "fs_main",
            TextureFormat::bevy_default(),
        );

        let fxaa_hdr_descriptor = fullscreen_vertex_pipeline_descriptor(
            "fxaa hdr pipeline",
            &fxaa_texture_bind_group,
            FXAA_SHADER_HANDLE,
            vec!["TONEMAP".to_string()],
            "fs_main",
            ViewTarget::TEXTURE_FORMAT_HDR,
        );

        let to_ldr_descriptor = fullscreen_vertex_pipeline_descriptor(
            "to ldr pipeline",
            &fxaa_texture_bind_group,
            LDR_SHADER_HANDLE,
            vec![],
            "fs_main",
            ViewTarget::TEXTURE_FORMAT_HDR,
        );

        let blit_descriptor = fullscreen_vertex_pipeline_descriptor(
            "blit pipeline",
            &fxaa_texture_bind_group,
            BLIT_SHADER_HANDLE,
            vec![],
            "fs_main",
            TextureFormat::bevy_default(),
        );

        let mut cache = render_world.resource_mut::<PipelineCache>();

        FXAAPipeline {
            texture_bind_group: fxaa_texture_bind_group,
            fxaa_ldr_pipeline_id: cache.queue_render_pipeline(fxaa_ldr_descriptor),
            fxaa_hdr_pipeline_id: cache.queue_render_pipeline(fxaa_hdr_descriptor),
            to_ldr_pipeline_id: cache.queue_render_pipeline(to_ldr_descriptor),
            blit_pipeline_id: cache.queue_render_pipeline(blit_descriptor),
        }
    }
}

fn fullscreen_vertex_pipeline_descriptor(
    label: &'static str,
    bind_group_layout: &BindGroupLayout,
    shader: HandleUntyped,
    shader_defs: Vec<String>,
    entry_point: &'static str,
    format: TextureFormat,
) -> RenderPipelineDescriptor {
    RenderPipelineDescriptor {
        label: Some(label.into()),
        layout: Some(vec![bind_group_layout.clone()]),
        vertex: fullscreen_shader_vertex_state(),
        fragment: Some(FragmentState {
            shader: shader.typed(),
            shader_defs,
            entry_point: Cow::Borrowed(entry_point),
            targets: vec![Some(ColorTargetState {
                format: format,
                blend: None,
                write_mask: ColorWrites::ALL,
            })],
        }),
        primitive: PrimitiveState::default(),
        depth_stencil: None,
        multisample: MultisampleState::default(),
    }
}

#[derive(Component)]
pub struct FXAATexture {
    pub output: CachedTexture,
}

pub fn prepare_fxaa_texture(
    mut commands: Commands,
    mut texture_cache: ResMut<TextureCache>,
    render_device: Res<RenderDevice>,
    views: Query<(Entity, &ExtractedCamera, &ExtractedView), With<FXAA>>,
) {
    let mut output_textures = HashMap::default();

    for (entity, camera, view) in &views {
        if let Some(physical_target_size) = camera.physical_target_size {
            let mut texture_descriptor = TextureDescriptor {
                label: None,
                size: Extent3d {
                    depth_or_array_layers: 1,
                    width: physical_target_size.x,
                    height: physical_target_size.y,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: if view.hdr {
                    ViewTarget::TEXTURE_FORMAT_HDR
                } else {
                    TextureFormat::bevy_default()
                },
                usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            };

            texture_descriptor.label = Some("fxaa_view_target_texture");
            let output = output_textures
                .entry(camera.target.clone())
                .or_insert_with(|| texture_cache.get(&render_device, texture_descriptor))
                .clone();

            commands.entity(entity).insert(FXAATexture { output });
        }
    }
}
