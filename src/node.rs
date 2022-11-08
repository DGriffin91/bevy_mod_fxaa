use std::sync::Mutex;

use bevy::prelude::*;

use bevy::render::render_graph::Node;
use bevy::render::render_graph::{NodeRunError, RenderGraphContext, SlotInfo, SlotType};
use bevy::render::render_resource::{
    BindGroupDescriptor, BindGroupEntry, BindingResource, FilterMode, Operations, PipelineCache,
    RenderPassColorAttachment, RenderPassDescriptor, SamplerDescriptor, TextureViewId,
};
use bevy::render::renderer::RenderContext;
use bevy::render::view::ExtractedView;
use bevy::{
    prelude::QueryState,
    render::{render_resource::BindGroup, view::ViewTarget},
};

use crate::{CameraFxaaPipelines, Fxaa, FxaaPipeline, ToLdrPipeline};

pub struct FxaaNode {
    query: QueryState<
        (
            &'static ViewTarget,
            &'static CameraFxaaPipelines,
            &'static Fxaa,
        ),
        With<ExtractedView>,
    >,
    cached_texture_bind_group: Mutex<Option<(TextureViewId, BindGroup)>>,
}

impl FxaaNode {
    pub const IN_VIEW: &'static str = "view";

    pub fn new(world: &mut World) -> Self {
        Self {
            query: QueryState::new(world),
            cached_texture_bind_group: Mutex::new(None),
        }
    }
}

impl Node for FxaaNode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![SlotInfo::new(FxaaNode::IN_VIEW, SlotType::Entity)]
    }

    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
    }

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let view_entity = graph.get_input_entity(Self::IN_VIEW)?;
        let pipeline_cache = world.resource::<PipelineCache>();
        let fxaa_pipeline = world.resource::<FxaaPipeline>();
        let to_ldr_pipeline = world.resource::<ToLdrPipeline>();

        let (target, pipeline, fxaa) = match self.query.get_manual(world, view_entity) {
            Ok(result) => result,
            Err(_) => return Ok(()),
        };

        if !fxaa.enabled {
            return Ok(());
        };

        let fxaa_render_pipeline = pipeline_cache
            .get_render_pipeline(pipeline.fxaa_pipeline_id)
            .unwrap();

        if target.is_hdr() {
            let to_ldr_render_pipeline = pipeline_cache
                .get_render_pipeline(pipeline.to_ldr_pipeline_id)
                .unwrap();
            let post_process = target.post_process_write();
            let source = post_process.source;
            let destination = post_process.destination;
            let mut cached_bind_group = self.cached_texture_bind_group.lock().unwrap();
            let bind_group = match &mut *cached_bind_group {
                Some((id, bind_group)) if source.id() == *id => bind_group,
                cached_bind_group => {
                    let sampler = render_context
                        .render_device
                        .create_sampler(&SamplerDescriptor {
                            mipmap_filter: FilterMode::Linear,
                            mag_filter: FilterMode::Linear,
                            min_filter: FilterMode::Linear,
                            ..default()
                        });

                    let bind_group =
                        render_context
                            .render_device
                            .create_bind_group(&BindGroupDescriptor {
                                label: None,
                                layout: &to_ldr_pipeline.texture_bind_group,
                                entries: &[
                                    BindGroupEntry {
                                        binding: 0,
                                        resource: BindingResource::TextureView(source),
                                    },
                                    BindGroupEntry {
                                        binding: 1,
                                        resource: BindingResource::Sampler(&sampler),
                                    },
                                ],
                            });

                    let (_, bind_group) = cached_bind_group.insert((source.id(), bind_group));
                    bind_group
                }
            };

            let pass_descriptor = RenderPassDescriptor {
                label: Some("fxaa_to_ldr_pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: destination,
                    resolve_target: None,
                    ops: Operations::default(),
                })],
                depth_stencil_attachment: None,
            };

            let mut render_pass = render_context
                .command_encoder
                .begin_render_pass(&pass_descriptor);

            render_pass.set_pipeline(to_ldr_render_pipeline);
            render_pass.set_bind_group(0, bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        let post_process = target.post_process_write();
        let source = post_process.source;
        let destination = post_process.destination;
        let mut cached_bind_group = self.cached_texture_bind_group.lock().unwrap();
        let bind_group = match &mut *cached_bind_group {
            Some((id, bind_group)) if source.id() == *id => bind_group,
            cached_bind_group => {
                let sampler = render_context
                    .render_device
                    .create_sampler(&SamplerDescriptor {
                        mipmap_filter: FilterMode::Linear,
                        mag_filter: FilterMode::Linear,
                        min_filter: FilterMode::Linear,
                        ..default()
                    });

                let bind_group =
                    render_context
                        .render_device
                        .create_bind_group(&BindGroupDescriptor {
                            label: None,
                            layout: &fxaa_pipeline.texture_bind_group,
                            entries: &[
                                BindGroupEntry {
                                    binding: 0,
                                    resource: BindingResource::TextureView(source),
                                },
                                BindGroupEntry {
                                    binding: 1,
                                    resource: BindingResource::Sampler(&sampler),
                                },
                            ],
                        });

                let (_, bind_group) = cached_bind_group.insert((source.id(), bind_group));
                bind_group
            }
        };

        let pass_descriptor = RenderPassDescriptor {
            label: Some("fxaa_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: destination,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
        };

        let mut render_pass = render_context
            .command_encoder
            .begin_render_pass(&pass_descriptor);

        render_pass.set_pipeline(fxaa_render_pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}
