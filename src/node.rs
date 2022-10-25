use std::sync::Mutex;

use bevy::prelude::*;
use bevy::render::render_graph::Node;
use bevy::render::render_graph::{NodeRunError, RenderGraphContext, SlotInfo, SlotType};
use bevy::render::render_resource::{
    BindGroupDescriptor, BindGroupEntry, BindingResource, FilterMode, LoadOp, Operations,
    PipelineCache, RenderPassColorAttachment, RenderPassDescriptor, SamplerDescriptor,
    TextureViewId,
};
use bevy::render::renderer::RenderContext;
use bevy::render::view::{ExtractedView, ViewMainTexture};
use bevy::{
    prelude::QueryState,
    render::{render_resource::BindGroup, view::ViewTarget},
};

use crate::pipeline::FXAAPipeline;
use crate::FXAA;
pub struct FXAANode {
    query: QueryState<(&'static ViewTarget, Option<&'static FXAA>), With<ExtractedView>>,
    cached_texture_bind_group: Mutex<Option<(TextureViewId, BindGroup)>>,
}

impl FXAANode {
    pub const IN_VIEW: &'static str = "view";

    pub fn new(world: &mut World) -> Self {
        Self {
            query: QueryState::new(world),
            cached_texture_bind_group: Mutex::new(None),
        }
    }
}

impl Node for FXAANode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![SlotInfo::new(FXAANode::IN_VIEW, SlotType::Entity)]
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
        let fxaa_pipeline = world.resource::<FXAAPipeline>();

        let (target, fxaa) = match self.query.get_manual(world, view_entity) {
            Ok(result) => result,
            Err(_) => return Ok(()),
        };

        let texture = match &target.main_texture {
            ViewMainTexture::Hdr { ldr_texture, .. } => ldr_texture,
            ViewMainTexture::Sdr { texture, .. } => texture,
        };

        let fxaa_enabled = fxaa.map_or(false, |t| t.enabled);

        let pipeline_id = if fxaa_enabled {
            fxaa_pipeline.fxaa_pipeline_id
        } else {
            fxaa_pipeline.blit_pipeline_id
        };

        let pipeline = match pipeline_cache.get_render_pipeline(pipeline_id) {
            Some(pipeline) => pipeline,
            None => return Ok(()),
        };

        let main_texture = target.main_texture.texture();

        let mut cached_bind_group = self.cached_texture_bind_group.lock().unwrap();
        let bind_group = match &mut *cached_bind_group {
            Some((id, bind_group)) if main_texture.id() == *id => bind_group,
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
                                    resource: BindingResource::TextureView(main_texture),
                                },
                                BindGroupEntry {
                                    binding: 1,
                                    resource: BindingResource::Sampler(&sampler),
                                },
                            ],
                        });

                let (_, bind_group) = cached_bind_group.insert((main_texture.id(), bind_group));
                bind_group
            }
        };

        let pass_descriptor = RenderPassDescriptor {
            label: Some("fxaa_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: texture,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Default::default()), // TODO shouldn't need to be cleared
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        };

        let mut render_pass = render_context
            .command_encoder
            .begin_render_pass(&pass_descriptor);

        render_pass.set_pipeline(pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}
