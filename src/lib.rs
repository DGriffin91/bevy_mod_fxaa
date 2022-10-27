use bevy::{
    asset::load_internal_asset,
    core_pipeline::core_2d,
    core_pipeline::core_3d,
    ecs::query::QueryItem,
    prelude::*,
    reflect::TypeUuid,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_graph::RenderGraph,
        RenderApp, RenderStage,
    },
};
use node::FXAANode;
use pipeline::FXAAPipelineBindGroup;

use crate::pipeline::prepare_fxaa_texture;

mod node;
mod pipeline;

#[derive(Clone)]
pub enum Quality {
    Low,
    Medium,
    High,
    Ultra,
}

impl Quality {
    fn get_str(&self) -> &str {
        match self {
            Quality::Low => "LOW",
            Quality::Medium => "MEDIUM",
            Quality::High => "HIGH",
            Quality::Ultra => "ULTRA",
        }
    }
}

#[derive(Component, Clone)]
pub struct FXAA {
    pub enabled: bool,

    //   0.250 - low quality
    //   0.166 - medium quality
    //   0.125 - high quality
    // The minimum amount of local contrast required to apply algorithm.
    pub edge_threshold: Quality,

    //   0.0833 - low quality, (the start of visible unfiltered edges)
    //   0.0625 - medium quality
    //   0.0312 - high quality, (visible limit)
    // Trims the algorithm from processing darks.
    pub edge_threshold_min: Quality,
}

impl Default for FXAA {
    fn default() -> Self {
        FXAA {
            enabled: true,
            edge_threshold: Quality::High,
            edge_threshold_min: Quality::High,
        }
    }
}

impl FXAA {
    pub fn get_settings(&self) -> Vec<String> {
        vec![
            format!("EDGE_THRESH_{}", self.edge_threshold.get_str()),
            format!("EDGE_THRESH_MIN_{}", self.edge_threshold_min.get_str()),
        ]
    }
}

impl ExtractComponent for FXAA {
    type Query = &'static Self;
    type Filter = With<Camera>;

    fn extract_component(item: QueryItem<Self::Query>) -> Self {
        item.clone()
    }
}

const LDR_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 3212361765414793412);

const FXAA_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2982361765441723543);

const BLIT_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2082981732578979433);

pub const FXAA_NODE_3D: &str = "fxaa_node_3d";
pub const FXAA_NODE_2D: &str = "fxaa_node_2d";

pub struct FXAAPlugin;
impl Plugin for FXAAPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Msaa { samples: 1 }); // Disable MSAA be default

        load_internal_asset!(app, LDR_SHADER_HANDLE, "to_ldr.wgsl", Shader::from_wgsl);
        load_internal_asset!(app, FXAA_SHADER_HANDLE, "fxaa.wgsl", Shader::from_wgsl);
        load_internal_asset!(app, BLIT_SHADER_HANDLE, "blit.wgsl", Shader::from_wgsl);

        app.add_plugin(ExtractComponentPlugin::<FXAA>::default());

        let render_app = match app.get_sub_app_mut(RenderApp) {
            Ok(render_app) => render_app,
            Err(_) => return,
        };
        render_app
            .init_resource::<FXAAPipelineBindGroup>()
            .add_system_to_stage(RenderStage::Prepare, prepare_fxaa_texture);

        {
            let fxaa_node = FXAANode::new(&mut render_app.world);
            let mut binding = render_app.world.resource_mut::<RenderGraph>();
            let graph = binding.get_sub_graph_mut(core_3d::graph::NAME).unwrap();

            graph.add_node(FXAA_NODE_3D, fxaa_node);

            graph
                .add_slot_edge(
                    graph.input_node().unwrap().id,
                    core_3d::graph::input::VIEW_ENTITY,
                    FXAA_NODE_3D,
                    FXAANode::IN_VIEW,
                )
                .unwrap();

            graph
                .add_node_edge(core_3d::graph::node::MAIN_PASS, FXAA_NODE_3D)
                .unwrap();

            graph
                .add_node_edge(FXAA_NODE_3D, core_3d::graph::node::TONEMAPPING)
                .unwrap();
        }
        {
            let fxaa_node = FXAANode::new(&mut render_app.world);
            let mut binding = render_app.world.resource_mut::<RenderGraph>();
            let graph = binding.get_sub_graph_mut(core_2d::graph::NAME).unwrap();

            graph.add_node(FXAA_NODE_2D, fxaa_node);

            graph
                .add_slot_edge(
                    graph.input_node().unwrap().id,
                    core_2d::graph::input::VIEW_ENTITY,
                    FXAA_NODE_2D,
                    FXAANode::IN_VIEW,
                )
                .unwrap();

            graph
                .add_node_edge(core_2d::graph::node::MAIN_PASS, FXAA_NODE_2D)
                .unwrap();

            graph
                .add_node_edge(FXAA_NODE_2D, core_2d::graph::node::TONEMAPPING)
                .unwrap();
        }
    }
}
